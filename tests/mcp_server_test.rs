use std::collections::BTreeSet;
use std::error::Error;

use rmcp::{
    ClientHandler, ServiceExt,
    model::{CallToolRequestParams, ClientInfo},
};
use serde_json::json;

use defuddle_rs::DefuddleResult;

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

fn example_html() -> String {
    let path = format!("{}/tests/fixtures/example.html", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("fixture should load")
}

async fn spawn_fixture_server() -> Result<String, Box<dyn Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let html = example_html();

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

    Ok(format!("http://{address}/example"))
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
async fn mcp_lists_expected_defuddle_tools() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let tools = client.peer().list_all_tools().await?;
    let names = tools
        .iter()
        .map(|tool| tool.name.to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        names,
        BTreeSet::from([
            "extract_markdown".to_string(),
            "extract_metadata".to_string(),
            "fetch_and_parse_url".to_string(),
            "fetch_url_markdown".to_string(),
            "parse_html".to_string(),
        ])
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn parse_html_tool_returns_full_defuddle_result() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let html = example_html();

    let result = client
        .call_tool(
            CallToolRequestParams::new("parse_html").with_arguments(
                json!({
                    "html": html,
                    "url": "https://example.com"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));
    let structured = result
        .structured_content
        .clone()
        .expect("parse_html should return structured content");
    let parsed: DefuddleResult = serde_json::from_value(structured)?;
    assert_eq!(parsed.title, "Example Domain");
    assert!(parsed.content_markdown.contains("documentation examples"));
    let text_json: serde_json::Value = serde_json::from_str(first_text(&result))?;
    assert_eq!(text_json, serde_json::to_value(&parsed)?);

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn extract_metadata_tool_returns_metadata_only() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let html = example_html();

    let result = client
        .call_tool(
            CallToolRequestParams::new("extract_metadata").with_arguments(
                json!({
                    "html": html,
                    "url": "https://example.com"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));
    let structured = result
        .structured_content
        .clone()
        .expect("extract_metadata should return structured content");

    assert_eq!(structured["title"], "Example Domain");
    assert_eq!(structured["language"], "en");
    assert_eq!(structured["site"], "example.com");
    assert!(structured.get("content_html").is_none());
    assert!(structured.get("content_markdown").is_none());
    assert!(structured.get("word_count").is_none());

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn extract_markdown_tool_returns_markdown_without_html() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let html = example_html();

    let result = client
        .call_tool(
            CallToolRequestParams::new("extract_markdown").with_arguments(
                json!({
                    "html": html,
                    "url": "https://example.com"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));
    let structured = result
        .structured_content
        .clone()
        .expect("extract_markdown should return structured content");

    assert_eq!(structured["title"], "Example Domain");
    assert_eq!(structured["word_count"], 17);
    assert!(
        structured["content_markdown"]
            .as_str()
            .expect("markdown should be a string")
            .contains("documentation examples")
    );
    assert!(structured.get("content_html").is_none());

    client.cancel().await?;
    Ok(())
}

#[tokio::test]
async fn fetch_and_parse_url_tool_fetches_and_parses_remote_html() -> Result<(), Box<dyn Error>> {
    let client = spawn_client_and_server().await?;
    let url = spawn_fixture_server().await?;

    let result = client
        .call_tool(
            CallToolRequestParams::new("fetch_and_parse_url")
                .with_arguments(json!({ "url": url }).as_object().unwrap().clone()),
        )
        .await?;

    assert_eq!(result.is_error, Some(false));
    let structured = result
        .structured_content
        .clone()
        .expect("fetch_and_parse_url should return structured content");
    let parsed: DefuddleResult = serde_json::from_value(structured)?;
    assert_eq!(parsed.title, "Example Domain");

    client.cancel().await?;
    Ok(())
}
