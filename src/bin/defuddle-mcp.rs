use std::error::Error;

use axum::Router;
use defuddle_rs::mcp::DefuddleMcpServer;
use rmcp::{
    ServiceExt,
    transport::{stdio, streamable_http_server::StreamableHttpServerConfig},
};

const DEFAULT_BIND: &str = "127.0.0.1:8080";
const DEFAULT_PATH: &str = "/mcp";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("stdio") => run_stdio().await?,
        Some("http") => run_http(args).await?,
        Some("--help") | Some("-h") | Some("help") => print_help(),
        Some(other) => {
            eprintln!("unknown mode: {other}\n");
            print_help();
            std::process::exit(2);
        }
    }
    Ok(())
}

async fn run_stdio() -> Result<(), Box<dyn Error>> {
    let server = DefuddleMcpServer::new();
    let running = server.serve(stdio()).await?;
    running.waiting().await?;
    Ok(())
}

async fn run_http(args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    let options = HttpOptions::parse(args)?;
    let path = normalize_path(&options.path);

    let config = StreamableHttpServerConfig::default()
        .with_stateful_mode(!options.stateless)
        .with_json_response(options.json_response);
    let service = DefuddleMcpServer::streamable_http_service(config);
    let router = Router::new().nest_service(&path, service);
    let listener = tokio::net::TcpListener::bind(&options.bind).await?;

    eprintln!(
        "defuddle-mcp serving streamable HTTP on http://{}{}",
        options.bind, path
    );

    axum::serve(listener, router).await?;
    Ok(())
}

#[derive(Debug, Clone)]
struct HttpOptions {
    bind: String,
    path: String,
    stateless: bool,
    json_response: bool,
}

impl Default for HttpOptions {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND.to_string(),
            path: DEFAULT_PATH.to_string(),
            stateless: false,
            json_response: false,
        }
    }
}

impl HttpOptions {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, Box<dyn Error>> {
        let mut options = Self::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--bind" => options.bind = expect_value(&mut args, "--bind")?,
                "--path" => options.path = expect_value(&mut args, "--path")?,
                "--stateless" => options.stateless = true,
                "--json-response" => options.json_response = true,
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => {
                    return Err(format!("unknown http option: {other}").into());
                }
            }
        }

        if options.json_response && !options.stateless {
            return Err("--json-response requires --stateless".into());
        }

        Ok(options)
    }
}

fn expect_value(
    args: &mut impl Iterator<Item = String>,
    flag: &str,
) -> Result<String, Box<dyn Error>> {
    args.next()
        .ok_or_else(|| format!("missing value for {flag}").into())
}

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn print_help() {
    eprintln!(
        "defuddle-mcp\n\n\
Usage:\n  defuddle-mcp [stdio]\n  defuddle-mcp http [--bind ADDR] [--path PATH] [--stateless --json-response]\n\n\
Modes:\n  stdio   Run the MCP server over stdio (default)\n  http    Run the MCP server over streamable HTTP\n\n\
HTTP options:\n  --bind ADDR         Listen address (default: {DEFAULT_BIND})\n  --path PATH         HTTP mount path (default: {DEFAULT_PATH})\n  --stateless         Disable sessionful HTTP mode\n  --json-response     Return application/json in stateless mode\n"
    );
}
