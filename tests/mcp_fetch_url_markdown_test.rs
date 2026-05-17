use std::error::Error;

use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientInfo},
};
use serde_json::json;

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

#[tokio::test]
async fn fetch_url_markdown_returns_markdown_shape() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let url = spawn_fixture_server("fasterthanlime.html").await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("fetch_url_markdown")
                .with_arguments(json!({ "url": url }).as_object().unwrap().clone()),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));

    let structured = result
        .structured_content
        .clone()
        .expect("fetch_url_markdown should return structured content");

    // Must not contain full-HTML or schema_org bloat fields
    assert!(
        structured.get("content_html").is_none(),
        "fetch_url_markdown must not include content_html"
    );
    assert!(
        structured.get("schema_org").is_none(),
        "fetch_url_markdown must not include schema_org"
    );

    // Must contain non-empty markdown content
    let markdown = structured["content_markdown"]
        .as_str()
        .expect("content_markdown should be a string");
    assert!(!markdown.is_empty(), "content_markdown must not be empty");

    // Entire JSON envelope must fit within Copilot's 50 KB tool-output cap
    let envelope_bytes = serde_json::to_vec(&structured)?.len();
    assert!(
        envelope_bytes < 50_000,
        "response envelope is {envelope_bytes} bytes, exceeds 50 KB cap"
    );

    client.cancel().await?;
    Ok(())
}
