use defuddle_rs::python_api::{DefuddleParser, PythonDefuddleError};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

#[test]
fn parser_parse_html_returns_full_result_shape() {
    let parser = DefuddleParser::new_for_test();
    let html = fixture("example.html");

    let result = parser
        .parse_html(html, "https://example.com".to_owned())
        .unwrap();

    assert_eq!(result.title, "Example Domain");
    assert_eq!(result.site.as_deref(), Some("example.com"));
    assert_eq!(result.language.as_deref(), Some("en"));
    assert!(result.content_html.contains("<p>"));
    assert!(
        result
            .content_markdown
            .contains("This domain is for use in documentation examples")
    );
    assert_eq!(result.word_count, 17);
    assert_eq!(result.schema_org_json, None);
}

#[test]
fn parser_extract_metadata_matches_mcp_surface() {
    let parser = DefuddleParser::new_for_test();
    let html = fixture("example.html");

    let result = parser
        .extract_metadata(html, "https://example.com".to_owned())
        .unwrap();

    assert_eq!(result.title, "Example Domain");
    assert_eq!(result.site.as_deref(), Some("example.com"));
    assert_eq!(result.language.as_deref(), Some("en"));
    assert!(result.description.is_none());
    assert!(result.schema_org_json.is_none());
}

#[test]
fn parser_extract_markdown_matches_browser_subset() {
    let parser = DefuddleParser::new_for_test();
    let html = fixture("example.html");

    let result = parser
        .extract_markdown(html, "https://example.com".to_owned())
        .unwrap();

    assert_eq!(result.title, "Example Domain");
    assert_eq!(result.site.as_deref(), Some("example.com"));
    assert_eq!(result.word_count, 17);
    assert!(
        result
            .content_markdown
            .contains("This domain is for use in documentation examples")
    );
    assert!(!result.content_markdown.contains("<p>"));
}

#[test]
fn parser_parse_html_reports_invalid_url_as_python_error() {
    let parser = DefuddleParser::new_for_test();
    let html = fixture("example.html");

    let error = parser.parse_html(html, "notaurl".to_owned()).unwrap_err();

    assert!(matches!(error, PythonDefuddleError::InvalidUrl { .. }));
}
