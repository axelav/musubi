use musubi::parse::{collapse_consecutive, extract_metadata, normalize_title};

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

#[test]
fn test_collapse_consecutive_spaces() {
    assert_eq!(collapse_consecutive("hello  world"), "hello world");
    assert_eq!(collapse_consecutive("multiple   spaces"), "multiple spaces");
}

#[test]
fn test_collapse_consecutive_hyphens() {
    assert_eq!(collapse_consecutive("test--hyphen"), "test-hyphen");
    assert_eq!(collapse_consecutive("many---hyphens"), "many-hyphens");
}

#[test]
fn test_collapse_mixed() {
    assert_eq!(collapse_consecutive("test  --  mixed"), "test - mixed");
}

#[test]
fn test_normalize_basic_ascii() {
    assert_eq!(normalize_title("Hello World"), "Hello World");
    assert_eq!(normalize_title("Test-123"), "Test-123");
}

#[test]
fn test_normalize_unicode_characters() {
    assert_eq!(normalize_title("Café & Bistró"), "Cafe & Bistro");
    assert_eq!(normalize_title("naïve"), "naive");
}

#[test]
fn test_normalize_special_punctuation() {
    assert_eq!(normalize_title("Bob's Pizza — The Best!"), "Bob's Pizza - The Best");
    assert_eq!(normalize_title("My «Awesome» Title"), "My Awesome Title");
}

#[test]
fn test_normalize_allowed_punctuation() {
    assert_eq!(normalize_title("Test (2024)"), "Test (2024)");
    assert_eq!(normalize_title("Hello, World!"), "Hello, World");
    assert_eq!(normalize_title("Item 1.5"), "Item 1.5");
}

#[test]
fn test_normalize_consecutive_spaces() {
    assert_eq!(normalize_title("Multiple   Spaces"), "Multiple Spaces");
}

#[test]
fn test_normalize_consecutive_hyphens() {
    assert_eq!(normalize_title("Test---Hyphens"), "Test-Hyphens");
}

#[test]
fn test_normalize_empty_and_whitespace() {
    assert_eq!(normalize_title(""), "Untitled");
    assert_eq!(normalize_title("   "), "Untitled");
    assert_eq!(normalize_title("———"), "Untitled");
}

#[test]
fn test_normalize_trim_whitespace() {
    assert_eq!(normalize_title("  Hello World  "), "Hello World");
}

#[test]
fn test_extract_metadata_normalizes_title() {
    let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Café & Bistró — The Best!</title>
            <meta name="description" content="Test description">
        </head>
        <body></body>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(metadata.title, "Cafe & Bistro - The Best");
}
