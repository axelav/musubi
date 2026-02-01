use musubi::now::create_now_file;
use std::fs;
use tempfile::TempDir;

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
