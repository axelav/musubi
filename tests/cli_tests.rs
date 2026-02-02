use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_cli_link_subcommand_basic() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("link")
        .arg("https://example.com")
        .arg("--dir")
        .arg(temp_dir.path())
        .env("MUSUBI_LINKS_DIR", temp_dir.path())
        .env("MUSUBI_NOW_DIR", temp_dir.path());
    
    // Note: This will fail in CI without network, but validates the CLI structure
    // The command should at least parse arguments correctly
    let output = cmd.output().unwrap();
    
    // Either succeeds or fails with network error, not argument parsing error
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Should not be a clap error about missing subcommand or invalid args
        assert!(!stderr.contains("error: unrecognized subcommand"));
        assert!(!stderr.contains("error: unexpected argument"));
    }
}

#[test]
fn test_cli_now_subcommand_no_edit() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("now")
        .arg("--no-edit")
        .arg("--dir")
        .arg(temp_dir.path())
        .env("MUSUBI_LINKS_DIR", temp_dir.path())
        .env("MUSUBI_NOW_DIR", temp_dir.path());
    
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("✓ Created:"));
    
    // Verify a file was created
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert_eq!(entries.len(), 1, "Should create exactly one file");
    assert!(entries[0].path().extension().unwrap() == "md");
}

#[test]
fn test_cli_now_subcommand_with_title_no_edit() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("now")
        .arg("test note title")
        .arg("--no-edit")
        .arg("--dir")
        .arg(temp_dir.path())
        .env("MUSUBI_LINKS_DIR", temp_dir.path())
        .env("MUSUBI_NOW_DIR", temp_dir.path());
    
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("✓ Created:"));
    
    // Verify the file contains the title
    let entries: Vec<_> = fs::read_dir(temp_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert_eq!(entries.len(), 1);
    let content = fs::read_to_string(entries[0].path()).unwrap();
    assert!(content.contains("title: test note title"));
}

#[test]
fn test_cli_requires_subcommand() {
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    
    // Running without any subcommand should fail
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_link_requires_url() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("link")
        .arg("--dir")
        .arg(temp_dir.path())
        .env("MUSUBI_LINKS_DIR", temp_dir.path())
        .env("MUSUBI_NOW_DIR", temp_dir.path());
    
    // Should fail because URL is required
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_cli_invalid_subcommand() {
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("invalid-command");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_cli_now_dir_override() {
    let temp_dir = TempDir::new().unwrap();
    let custom_dir = temp_dir.path().join("custom");
    
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("now")
        .arg("--no-edit")
        .arg("--dir")
        .arg(&custom_dir)
        .env("MUSUBI_LINKS_DIR", temp_dir.path())
        .env("MUSUBI_NOW_DIR", temp_dir.path());
    
    cmd.assert()
        .success();
    
    // Verify file was created in custom directory
    assert!(custom_dir.exists());
    let entries: Vec<_> = fs::read_dir(&custom_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    
    assert_eq!(entries.len(), 1);
}

#[test]
fn test_cli_help_shows_subcommands() {
    let mut cmd = Command::cargo_bin("musubi").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("link"))
        .stdout(predicate::str::contains("now"));
}
