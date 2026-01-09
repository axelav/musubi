use musubi::archive::{archive_page, ArchiveConfig};
use url::Url;

#[test]
fn test_archive_page_end_to_end() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <style>body { margin: 0; }</style>
    <script>alert('removed');</script>
</head>
<body onclick="removed()">
    <h1>Test Content</h1>
    <p>This is a test paragraph.</p>
</body>
</html>"#;

    let base_url = Url::parse("https://example.com/page").unwrap();
    let config = ArchiveConfig::default();

    let result = archive_page(html, &base_url, &config).unwrap();

    // Verify scripts removed
    assert!(!result.contains("<script"));
    assert!(!result.contains("alert"));

    // Verify event handlers removed
    assert!(!result.contains("onclick"));

    // Verify content preserved
    assert!(result.contains("Test Content"));
    assert!(result.contains("This is a test paragraph"));

    // Verify inline styles preserved
    assert!(result.contains("body { margin: 0; }"));
}

#[test]
fn test_archive_with_no_external_resources() {
    let html = r#"<html><body><p>Simple page</p></body></html>"#;
    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();

    let result = archive_page(html, &base_url, &config).unwrap();
    assert!(result.contains("Simple page"));
}
