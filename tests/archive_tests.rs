use musubi::archive::ArchiveConfig;
use musubi::archive::archive_page;
use std::time::Duration;
use url::Url;

#[test]
fn test_archive_config_defaults() {
    let config = ArchiveConfig::default();
    assert_eq!(config.css_timeout, Duration::from_secs(10));
    assert_eq!(config.css_max_size, 5 * 1024 * 1024);
}

#[test]
fn test_strips_script_tags() {
    let html = r#"<html><body>
        <p>Content</p>
        <script>alert('hello');</script>
        <script src="external.js"></script>
        <p>More content</p>
    </body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.contains("<script"));
    assert!(result.contains("Content"));
    assert!(result.contains("More content"));
}
