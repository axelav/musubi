use musubi::fetch::clean_url;

#[test]
fn test_clean_url_removes_utm_params() {
    let input = "https://example.com/page?utm_source=twitter&utm_campaign=test&id=123";
    let cleaned = clean_url(input).unwrap();
    assert_eq!(cleaned, "https://example.com/page?id=123");
}

#[test]
fn test_clean_url_removes_fbclid() {
    let input = "https://example.com/page?fbclid=abc123&foo=bar";
    let cleaned = clean_url(input).unwrap();
    assert_eq!(cleaned, "https://example.com/page?foo=bar");
}

#[test]
fn test_clean_url_preserves_functional_params() {
    let input = "https://example.com/search?q=rust&page=2";
    let cleaned = clean_url(input).unwrap();
    assert_eq!(cleaned, "https://example.com/search?q=rust&page=2");
}
