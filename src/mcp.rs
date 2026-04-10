use std::sync::Arc;

use rmcp::{
    ErrorData, Json, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Defuddle, DefuddleError, DefuddleResult};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ParseHtmlRequest {
    pub html: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FetchAndParseUrlRequest {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractMetadataResponse {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    pub schema_org: Option<serde_json::Value>,
}

impl ExtractMetadataResponse {
    fn from_html(html: &str, url: &str) -> Result<Self, DefuddleError> {
        let parsed_url = Url::parse(url).map_err(|e| DefuddleError::InvalidUrl(e.to_string()))?;
        let doc = dom_query::Document::from(html);
        let metadata = crate::metadata::extract(&doc, &parsed_url);
        let schema_org = crate::metadata::extract_schema_org(&doc);

        Ok(Self {
            title: metadata.title,
            author: metadata.author,
            published: metadata.published,
            site: metadata.site,
            description: metadata.description,
            image: metadata.image,
            language: metadata.language,
            schema_org,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExtractMarkdownResponse {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub content_markdown: String,
    pub word_count: usize,
}

impl From<DefuddleResult> for ExtractMarkdownResponse {
    fn from(value: DefuddleResult) -> Self {
        Self {
            title: value.title,
            author: value.author,
            published: value.published,
            site: value.site,
            description: value.description,
            language: value.language,
            content_markdown: value.content_markdown,
            word_count: value.word_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DefuddleMcpServer {
    tool_router: ToolRouter<Self>,
}

impl Default for DefuddleMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl DefuddleMcpServer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[must_use]
    pub fn streamable_http_service(
        config: StreamableHttpServerConfig,
    ) -> StreamableHttpService<Self, LocalSessionManager> {
        StreamableHttpService::new(
            || Ok(Self::new()),
            Arc::new(LocalSessionManager::default()),
            config,
        )
    }
}

#[tool_router(router = tool_router)]
impl DefuddleMcpServer {
    #[tool(
        name = "parse_html",
        description = "Parse raw HTML for a source URL and return the full Defuddle extraction result."
    )]
    pub async fn parse_html(
        &self,
        Parameters(ParseHtmlRequest { html, url }): Parameters<ParseHtmlRequest>,
    ) -> Result<Json<DefuddleResult>, ErrorData> {
        let parsed = Defuddle::parse(&html, &url).map_err(defuddle_error_to_mcp)?;
        Ok(Json(parsed))
    }

    #[tool(
        name = "fetch_and_parse_url",
        description = "Fetch a URL over HTTP and return the full Defuddle extraction result."
    )]
    pub async fn fetch_and_parse_url(
        &self,
        Parameters(FetchAndParseUrlRequest { url }): Parameters<FetchAndParseUrlRequest>,
    ) -> Result<Json<DefuddleResult>, ErrorData> {
        let parsed = Defuddle::fetch_and_parse(&url)
            .await
            .map_err(defuddle_error_to_mcp)?;
        Ok(Json(parsed))
    }

    #[tool(
        name = "extract_metadata",
        description = "Parse raw HTML for a source URL and return only extracted metadata fields."
    )]
    pub async fn extract_metadata(
        &self,
        Parameters(ParseHtmlRequest { html, url }): Parameters<ParseHtmlRequest>,
    ) -> Result<Json<ExtractMetadataResponse>, ErrorData> {
        let parsed =
            ExtractMetadataResponse::from_html(&html, &url).map_err(defuddle_error_to_mcp)?;
        Ok(Json(parsed))
    }

    #[tool(
        name = "extract_markdown",
        description = "Parse raw HTML for a source URL and return markdown plus lightweight metadata."
    )]
    pub async fn extract_markdown(
        &self,
        Parameters(ParseHtmlRequest { html, url }): Parameters<ParseHtmlRequest>,
    ) -> Result<Json<ExtractMarkdownResponse>, ErrorData> {
        let parsed = Defuddle::parse(&html, &url).map_err(defuddle_error_to_mcp)?;
        Ok(Json(parsed.into()))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for DefuddleMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Use Defuddle tools to extract clean content and metadata from HTML or URLs."
                .to_owned(),
        )
    }
}

fn defuddle_error_to_mcp(error: DefuddleError) -> ErrorData {
    match error {
        DefuddleError::InvalidUrl(message) => ErrorData::invalid_params(message, None),
        DefuddleError::Fetch(message) | DefuddleError::Parse(message) => {
            ErrorData::internal_error(message, None)
        }
    }
}
