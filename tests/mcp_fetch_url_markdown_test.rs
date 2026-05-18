use std::error::Error;

use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientInfo},
};
use serde_json::json;
use tempfile::tempdir;

#[derive(Debug, Clone, Default)]
struct DummyClient;

impl ClientHandler for DummyClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

async fn spawn_client_and_server()
-> Result<rmcp::service::RunningService<rmcp::RoleClient, DummyClient>, Box<dyn Error>> {
    let (server_transport, client_transport) = tokio::io::duplex(16 * 1024);

    tokio::spawn(async move {
        let server = defuddle_rs::mcp::DefuddleMcpServer::new();
        let running = server
            .serve(server_transport)
            .await
            .expect("server should start");
        running.waiting().await.expect("server should stay healthy");
    });

    Ok(DummyClient.serve(client_transport).await?)
}

async fn spawn_fixture_server(fixture_name: &'static str) -> Result<String, Box<dyn Error>> {
    let path = format!(
        "{}/tests/fixtures/{fixture_name}",
        env!("CARGO_MANIFEST_DIR")
    );
    let html = std::fs::read_to_string(path).expect("fixture should load");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;

    tokio::spawn(async move {
        loop {
            let Ok((mut socket, _)) = listener.accept().await else {
                break;
            };
            let body = html.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};

                let mut buffer = [0_u8; 4096];
                let _ = socket.read(&mut buffer).await;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.shutdown().await;
            });
        }
    });

    Ok(format!("http://{address}/page"))
}

fn first_text(result: &rmcp::model::CallToolResult) -> &str {
    result
        .content
        .first()
        .and_then(|content| content.raw.as_text())
        .map(|text| text.text.as_str())
        .expect("tool result should include text content")
}

#[tokio::test]
async fn fetch_url_markdown_returns_plain_text_not_json() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let url = spawn_fixture_server("fasterthanlime.html").await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("fetch_url_markdown")
                .with_arguments(json!({ "url": url }).as_object().unwrap().clone()),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));

    // Must NOT return structured_content — plain text only avoids the CLI's /tmp save
    assert!(
        result.structured_content.is_none(),
        "fetch_url_markdown must return plain text, not Json<T>"
    );

    let text = first_text(&result);
    assert!(!text.is_empty(), "content_markdown must not be empty");

    // Must be plain markdown, not a JSON object literal
    assert!(
        serde_json::from_str::<serde_json::Value>(text)
            .map(|v| !v.is_object())
            .unwrap_or(true),
        "text content must not be a JSON object"
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn fetch_url_markdown_fits_inline_for_typical_article() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    // rust_blog.html is a typical article-length page (~19 KB HTML)
    let url = spawn_fixture_server("rust_blog.html").await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("fetch_url_markdown")
                .with_arguments(json!({ "url": url }).as_object().unwrap().clone()),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));
    let text = first_text(&result);
    assert!(
        text.len() < 25_000,
        "rust_blog markdown is {} bytes, exceeds 25 KB inline threshold",
        text.len()
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn fetch_and_save_markdown_writes_file_and_returns_confirmation() -> Result<(), Box<dyn Error>>
{
    let client = spawn_client_and_server().await?;
    let url = spawn_fixture_server("fasterthanlime.html").await?;
    let dir = tempdir()?;
    let output_path = dir.path().join("out.md");

    let result = client
        .call_tool(
            CallToolRequestParams::new("fetch_and_save_markdown").with_arguments(
                json!({
                    "url": url,
                    "output_path": output_path.to_str().unwrap()
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));

    let written = std::fs::read_to_string(&output_path)?;
    assert!(!written.is_empty(), "written file must not be empty");

    let confirmation = first_text(&result);
    assert!(
        confirmation.len() < 200,
        "confirmation should be short, not the full markdown ({} bytes)",
        confirmation.len()
    );
    assert!(
        confirmation.contains("Saved"),
        "confirmation should say Saved, got: {confirmation}"
    );

    client.cancel().await?;
    Ok(())
}
