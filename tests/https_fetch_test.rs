use std::error::Error;

#[tokio::test]
#[ignore = "requires external network access"]
async fn fetch_and_parse_public_https_url() -> Result<(), Box<dyn Error>> {
    let parsed = defuddle_rs::Defuddle::fetch_and_parse("https://example.com/").await?;

    assert_eq!(parsed.title, "Example Domain");
    assert!(!parsed.content_markdown.trim().is_empty());
    assert!(parsed.word_count > 0);

    Ok(())
}
