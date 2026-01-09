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

#[test]
fn test_strips_self_closing_script_tags() {
    let html = r#"<html><body>
        <p>Before</p>
        <script src="test.js"/>
        <p>After</p>
    </body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.contains("<script"));
    assert!(result.contains("Before"));
    assert!(result.contains("After"), "Content after self-closing script tag should be preserved");
}

#[test]
fn test_strips_uppercase_script_tags() {
    let html = r#"<html><body>
        <p>Content</p>
        <SCRIPT>alert('hello');</SCRIPT>
        <p>More content</p>
    </body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.to_lowercase().contains("<script"));
    assert!(result.contains("Content"));
    assert!(result.contains("More content"));
}

#[test]
fn test_strips_mixed_case_script_tags() {
    let html = r#"<html><body>
        <p>Before</p>
        <ScRiPt>alert('test');</sCrIpT>
        <p>Middle</p>
        <Script src="test.js"></Script>
        <p>After</p>
    </body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.to_lowercase().contains("<script"));
    assert!(result.contains("Before"));
    assert!(result.contains("Middle"));
    assert!(result.contains("After"));
}

#[test]
fn test_strips_event_handlers() {
    let html = r##"<html><body>
        <button onclick="alert('click')">Click me</button>
        <div onload="init()" onmouseover="hover()">Content</div>
        <a href="#" onmousedown="track()">Link</a>
    </body></html>"##;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.contains("onclick"));
    assert!(!result.contains("onload"));
    assert!(!result.contains("onmouseover"));
    assert!(!result.contains("onmousedown"));
    assert!(result.contains("Click me"));
    assert!(result.contains("Content"));
}

#[test]
fn test_strips_single_quoted_event_handlers() {
    let html = r##"<html><body>
        <button onclick='alert("click")'>Click me</button>
        <div onload='init()' onmouseover='hover()'>Content</div>
        <a href="#" onmousedown='track()'>Link</a>
    </body></html>"##;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.contains("onclick"));
    assert!(!result.contains("onload"));
    assert!(!result.contains("onmouseover"));
    assert!(!result.contains("onmousedown"));
    assert!(result.contains("Click me"));
    assert!(result.contains("Content"));
}
