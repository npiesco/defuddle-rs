use std::sync::Arc;

use dom_query::Document;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{Defuddle, DefuddleError, DefuddleResult};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum PythonDefuddleError {
    #[error("invalid URL: {message}")]
    InvalidUrl { message: String },

    #[error("HTTP fetch failed: {message}")]
    Fetch { message: String },

    #[error("parse error: {message}")]
    Parse { message: String },

    #[error("runtime error: {message}")]
    Runtime { message: String },
}

impl From<DefuddleError> for PythonDefuddleError {
    fn from(value: DefuddleError) -> Self {
        match value {
            DefuddleError::InvalidUrl(message) => Self::InvalidUrl { message },
            DefuddleError::Fetch(message) => Self::Fetch { message },
            DefuddleError::Parse(message) => Self::Parse { message },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct PythonDefuddleResult {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    pub content_html: String,
    pub content_markdown: String,
    pub word_count: u64,
    pub schema_org_json: Option<String>,
}

impl From<DefuddleResult> for PythonDefuddleResult {
    fn from(value: DefuddleResult) -> Self {
        Self {
            title: value.title,
            author: value.author,
            published: value.published,
            site: value.site,
            description: value.description,
            image: value.image,
            language: value.language,
            content_html: value.content_html,
            content_markdown: value.content_markdown,
            word_count: value.word_count as u64,
            schema_org_json: value
                .schema_org
                .and_then(|schema| serde_json::to_string(&schema).ok()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct PythonMetadataResult {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub language: Option<String>,
    pub schema_org_json: Option<String>,
}

impl PythonMetadataResult {
    fn from_html(html: &str, url: &str) -> Result<Self, PythonDefuddleError> {
        let parsed_url = Url::parse(url).map_err(|error| PythonDefuddleError::InvalidUrl {
            message: error.to_string(),
        })?;
        let doc = Document::from(html);
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
            schema_org_json: schema_org.and_then(|value| serde_json::to_string(&value).ok()),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct PythonMarkdownResult {
    pub title: String,
    pub author: Option<String>,
    pub published: Option<String>,
    pub site: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub content_markdown: String,
    pub word_count: u64,
}

impl From<DefuddleResult> for PythonMarkdownResult {
    fn from(value: DefuddleResult) -> Self {
        Self {
            title: value.title,
            author: value.author,
            published: value.published,
            site: value.site,
            description: value.description,
            language: value.language,
            content_markdown: value.content_markdown,
            word_count: value.word_count as u64,
        }
    }
}

#[derive(Debug, Default, uniffi::Object)]
pub struct DefuddleParser;

impl DefuddleParser {
    pub fn new_for_test() -> Self {
        Self
    }
}

#[uniffi::export]
impl DefuddleParser {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self)
    }

    pub fn parse_html(
        &self,
        html: String,
        url: String,
    ) -> Result<PythonDefuddleResult, PythonDefuddleError> {
        let parsed = Defuddle::parse(&html, &url)?;
        Ok(parsed.into())
    }

    pub fn fetch_and_parse_url(
        &self,
        url: String,
    ) -> Result<PythonDefuddleResult, PythonDefuddleError> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| PythonDefuddleError::Runtime {
                message: error.to_string(),
            })?;
        let parsed = runtime.block_on(Defuddle::fetch_and_parse(&url))?;
        Ok(parsed.into())
    }

    pub fn extract_metadata(
        &self,
        html: String,
        url: String,
    ) -> Result<PythonMetadataResult, PythonDefuddleError> {
        PythonMetadataResult::from_html(&html, &url)
    }

    pub fn extract_markdown(
        &self,
        html: String,
        url: String,
    ) -> Result<PythonMarkdownResult, PythonDefuddleError> {
        let parsed = Defuddle::parse(&html, &url)?;
        Ok(parsed.into())
    }
}
