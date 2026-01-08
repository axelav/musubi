use musubi::parse::{extract_metadata, PageMetadata};

#[test]
fn test_extract_title_from_html() {
    let html = r#"
        <html>
            <head><title>Test Page Title</title></head>
            <body>Content</body>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(metadata.title, "Test Page Title");
}

#[test]
fn test_extract_description_from_meta() {
    let html = r#"
        <html>
            <head>
                <title>Test</title>
                <meta name="description" content="This is a test description">
            </head>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(
        metadata.description,
        Some("This is a test description".to_string())
    );
}

#[test]
fn test_extract_og_description() {
    let html = r#"
        <html>
            <head>
                <title>Test</title>
                <meta property="og:description" content="Open Graph description">
            </head>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(
        metadata.description,
        Some("Open Graph description".to_string())
    );
}
