use std::error::Error;

use axum::Router;
use rmcp::{
    ClientHandler, ServiceExt,
    model::ClientInfo,
    transport::{StreamableHttpClientTransport, StreamableHttpServerConfig},
};

#[derive(Debug, Clone, Default)]
struct DummyClient;

impl ClientHandler for DummyClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}

#[tokio::test]
async fn streamable_http_server_lists_defuddle_tools() -> Result<(), Box<dyn Error>> {
    let service = defuddle_rs::mcp::DefuddleMcpServer::streamable_http_service(
        StreamableHttpServerConfig::default()
            .with_stateful_mode(false)
            .with_json_response(true)
            .with_sse_keep_alive(None),
    );
    let router = Router::new().nest_service("/mcp", service);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let handle = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("http server should run");
    });

    let transport = StreamableHttpClientTransport::from_uri(format!("http://{addr}/mcp"));
    let client = DummyClient.serve(transport).await?;
    let tools = client.peer().list_all_tools().await?;

    assert!(tools.iter().any(|tool| tool.name == "parse_html"));
    assert!(tools.iter().any(|tool| tool.name == "extract_markdown"));

    client.cancel().await?;
    handle.abort();
    let _ = handle.await;
    Ok(())
}
