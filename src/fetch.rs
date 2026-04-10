//! HTTP fetch — retrieve web pages via reqwest.

use std::error::Error;

use crate::DefuddleError;

/// Fetch the HTML content of a URL.
pub async fn get(url: &str) -> Result<String, DefuddleError> {
    let rustls_client = reqwest::Client::builder()
        .use_rustls_tls()
        .tls_built_in_native_certs(true)
        .tls_built_in_webpki_certs(true)
        .user_agent(
            "Mozilla/5.0 (compatible; defuddle-rs/0.1; +https://github.com/npiesco/defuddle-rs)",
        )
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| DefuddleError::Fetch(format_reqwest_error("build rustls client", url, &e)))?;

    match fetch_with_client(&rustls_client, url).await {
        Ok(html) => Ok(html),
        Err(error) if should_retry_with_native_tls(&error) => {
            let native_tls_client = reqwest::Client::builder()
                .use_native_tls()
                .user_agent(
                    "Mozilla/5.0 (compatible; defuddle-rs/0.1; +https://github.com/npiesco/defuddle-rs)",
                )
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|native_error| {
                    DefuddleError::Fetch(format!(
                        "{}; fallback native-tls client also failed to build: {}",
                        format_reqwest_error("send request", url, &error),
                        format_reqwest_error("build native-tls client", url, &native_error)
                    ))
                })?;

            match fetch_with_client(&native_tls_client, url).await {
                Ok(html) => Ok(html),
                Err(native_error) if should_retry_insecurely(&native_error) => {
                    tracing::warn!(
                        url,
                        "TLS certificate verification failed with both rustls and native-tls; retrying insecurely"
                    );

                    let insecure_client = reqwest::Client::builder()
                        .use_native_tls()
                        .danger_accept_invalid_certs(true)
                        .user_agent(
                            "Mozilla/5.0 (compatible; defuddle-rs/0.1; +https://github.com/npiesco/defuddle-rs)",
                        )
                        .timeout(std::time::Duration::from_secs(30))
                        .build()
                        .map_err(|insecure_build_error| {
                            DefuddleError::Fetch(format!(
                                "{}; fallback via native-tls also failed: {}; insecure fallback client failed to build: {}",
                                format_reqwest_error("send request", url, &error),
                                format_reqwest_error("send request", url, &native_error),
                                format_reqwest_error("build insecure client", url, &insecure_build_error)
                            ))
                        })?;

                    fetch_with_client(&insecure_client, url)
                        .await
                        .map_err(|insecure_error| {
                            DefuddleError::Fetch(format!(
                                "{}; fallback via native-tls also failed: {}; insecure fallback also failed: {}",
                                format_reqwest_error("send request", url, &error),
                                format_reqwest_error("send request", url, &native_error),
                                format_reqwest_error("send request", url, &insecure_error)
                            ))
                        })
                }
                Err(native_error) => Err(DefuddleError::Fetch(format!(
                    "{}; fallback via native-tls also failed: {}",
                    format_reqwest_error("send request", url, &error),
                    format_reqwest_error("send request", url, &native_error)
                ))),
            }
        }
        Err(error) => Err(DefuddleError::Fetch(format_reqwest_error(
            "send request",
            url,
            &error,
        ))),
    }
}

async fn fetch_with_client(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    let response = client.get(url).send().await?.error_for_status()?;
    response.text().await
}

fn should_retry_with_native_tls(error: &reqwest::Error) -> bool {
    should_retry_insecurely(error)
}

fn should_retry_insecurely(error: &reqwest::Error) -> bool {
    let message = format_reqwest_error("send request", "", error).to_ascii_lowercase();
    error.is_connect()
        && (message.contains("certificate")
            || message.contains("unknownissuer")
            || message.contains("unknown issuer")
            || message.contains("tls"))
}

fn format_reqwest_error(action: &str, url: &str, error: &reqwest::Error) -> String {
    let mut message = if url.is_empty() {
        format!("{action} failed: {error}")
    } else {
        format!("{action} for url ({url}) failed: {error}")
    };
    let mut source = error.source();

    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }

    message
}
