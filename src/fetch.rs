//! HTTP fetch — retrieve web pages via reqwest.

use crate::DefuddleError;

/// Fetch the HTML content of a URL.
pub async fn get(url: &str) -> Result<String, DefuddleError> {
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (compatible; defuddle-rs/0.1; +https://github.com/npiesco/defuddle-rs)",
        )
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| DefuddleError::Fetch(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| DefuddleError::Fetch(e.to_string()))?;

    if !response.status().is_success() {
        return Err(DefuddleError::Fetch(format!(
            "HTTP {}: {}",
            response.status(),
            url
        )));
    }

    response
        .text()
        .await
        .map_err(|e| DefuddleError::Fetch(e.to_string()))
}
