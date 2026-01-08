use std::fs;
use tempfile::TempDir;
use musubi::fetch;
use musubi::parse;
use musubi::writer;

#[test]
fn test_end_to_end_without_llm() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();

    // Simple HTML for testing
    let test_html = r#"
        <html>
            <head>
                <title>Test Page</title>
                <meta name="description" content="A test page">
            </head>
            <body>
                <article>This is test content for the page.</article>
            </body>
        </html>
    "#;

    // Parse metadata
    let metadata = parse::extract_metadata(test_html, "https://example.com/test").unwrap();
    assert_eq!(metadata.title, "Test Page");

    // Write file without summary
    let file_path = writer::write_link_file(
        temp_dir.path(),
        &metadata.title,
        &metadata.url,
        &metadata.fetch_date,
        None,
        &[],
    ).unwrap();

    // Verify file was created
    assert!(file_path.exists());

    // Verify content
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("title: Test Page"));
    assert!(content.contains("https://example.com/test"));
    assert!(content.contains("## Test Page"));
    assert!(content.contains("#links"));
}

#[test]
fn test_url_cleaning_integration() {
    let url = "https://example.com/page?utm_source=test&id=123";
    let cleaned = fetch::clean_url(url).unwrap();
    assert!(!cleaned.contains("utm_source"));
    assert!(cleaned.contains("id=123"));
}
