use musubi::writer::sanitize_filename;

#[test]
fn test_sanitize_filename_removes_special_chars() {
    let input = "Hello/World: A Test?";
    let sanitized = sanitize_filename(input);
    assert_eq!(sanitized, "Hello-World- A Test-");
}

#[test]
fn test_sanitize_filename_collapses_spaces() {
    let input = "Hello    World";
    let sanitized = sanitize_filename(input);
    assert_eq!(sanitized, "Hello World");
}

#[test]
fn test_sanitize_filename_truncates_long_names() {
    let input = "a".repeat(150);
    let sanitized = sanitize_filename(&input);
    assert!(sanitized.len() <= 100);
}
