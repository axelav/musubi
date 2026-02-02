use musubi::now::create_now_file;
use std::fs;
use tempfile::TempDir;

// Helper function to extract YAML front matter title value
fn extract_yaml_title(content: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() < 3 || lines[0] != "---" {
        return None;
    }
    
    for line in &lines[1..] {
        if line.starts_with("title:") {
            let title_part = line.strip_prefix("title:").unwrap().trim();
            return Some(title_part.to_string());
        }
        if *line == "---" {
            break;
        }
    }
    None
}

#[test]
fn test_create_now_file_with_title() {
    let temp_dir = TempDir::new().unwrap();
    let (path, _) = create_now_file(temp_dir.path(), Some("test note"), false).unwrap();

    assert!(path.exists());
    let filename = path.file_name().unwrap().to_str().unwrap();
    assert!(filename.ends_with("test note.md"));

    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("title: test note"));
    assert!(content.contains("## test note"));
}

#[test]
fn test_create_now_file_without_title() {
    let temp_dir = TempDir::new().unwrap();
    let (path, _) = create_now_file(temp_dir.path(), None, false).unwrap();

    assert!(path.exists());
    // Should use time-based title (HH-MM-SS format)
    let content = fs::read_to_string(&path).unwrap();
    assert!(content.contains("title:"));
    assert!(content.contains("date:"));
}

#[test]
fn test_create_now_file_collision_handling() {
    let temp_dir = TempDir::new().unwrap();

    let (path1, _) = create_now_file(temp_dir.path(), Some("duplicate"), false).unwrap();
    let (path2, _) = create_now_file(temp_dir.path(), Some("duplicate"), false).unwrap();

    assert!(path1.exists());
    assert!(path2.exists());
    assert_ne!(path1, path2);

    let filename2 = path2.file_name().unwrap().to_str().unwrap();
    assert!(filename2.contains("-2.md"));
}

#[test]
fn test_create_now_file_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir.path().join("nested").join("path");

    let (path, _) = create_now_file(&nested_dir, Some("test"), false).unwrap();

    assert!(path.exists());
    assert!(nested_dir.exists());
}

#[test]
fn test_yaml_escaping_colon_in_title() {
    let temp_dir = TempDir::new().unwrap();
    let title = "Note: Important Meeting";
    let (path, _) = create_now_file(temp_dir.path(), Some(title), false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let yaml_title = extract_yaml_title(&content).unwrap();
    
    // Title with colon should be quoted
    assert!(yaml_title.starts_with('"'));
    assert!(yaml_title.ends_with('"'));
    assert!(yaml_title.contains("Note: Important Meeting"));
}

#[test]
fn test_yaml_escaping_hash_in_title() {
    let temp_dir = TempDir::new().unwrap();
    let title = "Issue #123 Fix";
    let (path, _) = create_now_file(temp_dir.path(), Some(title), false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let yaml_title = extract_yaml_title(&content).unwrap();
    
    // Title with hash should be quoted
    assert!(yaml_title.starts_with('"'));
    assert!(yaml_title.contains("Issue #123 Fix"));
}

#[test]
fn test_yaml_escaping_quotes_in_title() {
    let temp_dir = TempDir::new().unwrap();
    let title = r#"My "quoted" title"#;
    let (path, _) = create_now_file(temp_dir.path(), Some(title), false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let yaml_title = extract_yaml_title(&content).unwrap();
    
    // Title with quotes should escape them
    assert!(yaml_title.contains(r#"\""#));
}

#[test]
fn test_yaml_no_escaping_simple_title() {
    let temp_dir = TempDir::new().unwrap();
    let title = "Simple Title";
    let (path, _) = create_now_file(temp_dir.path(), Some(title), false).unwrap();

    let content = fs::read_to_string(&path).unwrap();
    let yaml_title = extract_yaml_title(&content).unwrap();
    
    // Simple title should not be quoted
    assert_eq!(yaml_title, "Simple Title");
}
