use musubi::config::Config;
use std::env;
use std::fs;
use tempfile::TempDir;

/// Build a TempDir and a not-yet-created path inside it for use as MUSUBI_CONFIG.
/// The path doesn't exist by default, so tests that want env-only behavior get it
/// for free; tests that want a file write to this path before calling Config::load.
fn empty_config_path() -> (TempDir, std::path::PathBuf) {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("config.toml");
    (dir, path)
}

fn with_test_env<F: FnOnce()>(
    extra: Vec<(&'static str, Option<&str>)>,
    config_path: &std::path::Path,
    f: F,
) {
    let mut vars: Vec<(&'static str, Option<String>)> = vec![
        ("ANTHROPIC_API_KEY", None),
        ("OPENAI_API_KEY", None),
        ("MUSUBI_LINKS_DIR", None),
        ("MUSUBI_NOW_DIR", None),
        ("MUSUBI_CONFIG", Some(config_path.to_string_lossy().into_owned())),
    ];
    for (k, v) in extra {
        vars.retain(|(existing_k, _)| *existing_k != k);
        vars.push((k, v.map(|s| s.to_string())));
    }
    temp_env::with_vars(vars, f);
}

// --- env-only behavior (carried over from from_env) ---

#[test]
fn test_config_reads_anthropic_key() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![("ANTHROPIC_API_KEY", Some("test-key-123"))],
        &path,
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.anthropic_key, Some("test-key-123".to_string()));
        },
    );
}

#[test]
fn test_config_reads_openai_key() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![("OPENAI_API_KEY", Some("test-openai-key"))],
        &path,
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.openai_key, Some("test-openai-key".to_string()));
        },
    );
}

#[test]
fn test_has_llm_key_with_anthropic() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![("ANTHROPIC_API_KEY", Some("test-key"))],
        &path,
        || {
            let config = Config::load().unwrap();
            assert!(config.has_llm_key());
        },
    );
}

#[test]
fn test_has_llm_key_with_openai() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![("OPENAI_API_KEY", Some("test-key"))],
        &path,
        || {
            let config = Config::load().unwrap();
            assert!(config.has_llm_key());
        },
    );
}

#[test]
fn test_has_llm_key_returns_false_when_no_keys() {
    let (_d, path) = empty_config_path();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        assert!(!config.has_llm_key());
    });
}

#[test]
fn test_custom_links_dir() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![("MUSUBI_LINKS_DIR", Some("/custom/path/to/links"))],
        &path,
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.links_dir.to_str().unwrap(), "/custom/path/to/links");
        },
    );
}

#[test]
fn test_config_defaults_to_home_links() {
    let (_d, path) = empty_config_path();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        let home = env::var("HOME").unwrap();
        let expected = std::path::PathBuf::from(home).join("links");
        assert_eq!(config.links_dir, expected);
    });
}

#[test]
fn test_custom_now_dir() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![("MUSUBI_NOW_DIR", Some("/custom/path/to/now"))],
        &path,
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.now_dir.to_str().unwrap(), "/custom/path/to/now");
        },
    );
}

#[test]
fn test_config_defaults_to_home_now() {
    let (_d, path) = empty_config_path();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        let home = env::var("HOME").unwrap();
        let expected = std::path::PathBuf::from(home).join("now");
        assert_eq!(config.now_dir, expected);
    });
}

#[test]
fn test_config_does_not_require_home_when_both_dirs_set() {
    let (_d, path) = empty_config_path();
    with_test_env(
        vec![
            ("MUSUBI_LINKS_DIR", Some("/custom/links")),
            ("MUSUBI_NOW_DIR", Some("/custom/now")),
            ("HOME", None),
        ],
        &path,
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.links_dir.to_str().unwrap(), "/custom/links");
            assert_eq!(config.now_dir.to_str().unwrap(), "/custom/now");
        },
    );
}

// --- file-based behavior ---

#[test]
fn test_loads_anthropic_key_from_file() {
    let (_d, path) = empty_config_path();
    fs::write(&path, r#"anthropic_api_key = "from-file""#).unwrap();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        assert_eq!(config.anthropic_key.as_deref(), Some("from-file"));
    });
}

#[test]
fn test_loads_openai_key_from_file() {
    let (_d, path) = empty_config_path();
    fs::write(&path, r#"openai_api_key = "from-file""#).unwrap();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        assert_eq!(config.openai_key.as_deref(), Some("from-file"));
    });
}

#[test]
fn test_env_overrides_file_per_field() {
    let (_d, path) = empty_config_path();
    fs::write(
        &path,
        r#"
            anthropic_api_key = "file-anthropic"
            openai_api_key = "file-openai"
        "#,
    )
    .unwrap();
    with_test_env(
        vec![("ANTHROPIC_API_KEY", Some("env-anthropic"))],
        &path,
        || {
            let config = Config::load().unwrap();
            // env wins for anthropic
            assert_eq!(config.anthropic_key.as_deref(), Some("env-anthropic"));
            // file fills in openai because env is unset
            assert_eq!(config.openai_key.as_deref(), Some("file-openai"));
        },
    );
}

#[test]
fn test_loads_dirs_from_file() {
    let (_d, path) = empty_config_path();
    fs::write(
        &path,
        r#"
            links_dir = "/files/links"
            now_dir = "/files/now"
        "#,
    )
    .unwrap();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        assert_eq!(config.links_dir.to_str().unwrap(), "/files/links");
        assert_eq!(config.now_dir.to_str().unwrap(), "/files/now");
    });
}

#[test]
fn test_tilde_expands_in_file_paths() {
    let (_d, path) = empty_config_path();
    fs::write(
        &path,
        r#"
            links_dir = "~/from-tilde-links"
            now_dir = "~/from-tilde-now"
        "#,
    )
    .unwrap();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        let home = env::var("HOME").unwrap();
        assert_eq!(
            config.links_dir,
            std::path::PathBuf::from(&home).join("from-tilde-links")
        );
        assert_eq!(
            config.now_dir,
            std::path::PathBuf::from(&home).join("from-tilde-now")
        );
    });
}

#[test]
fn test_missing_file_is_not_an_error() {
    let (_d, path) = empty_config_path();
    // path intentionally not created
    assert!(!path.exists());
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        assert!(config.anthropic_key.is_none());
        assert!(config.openai_key.is_none());
    });
}

#[test]
fn test_malformed_file_errors() {
    let (_d, path) = empty_config_path();
    fs::write(&path, "not valid = = toml [[[").unwrap();
    with_test_env(vec![], &path, || {
        let err = Config::load().unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.to_lowercase().contains("config")
                || msg.to_lowercase().contains("toml")
                || msg.to_lowercase().contains("parse"),
            "unexpected error message: {msg}"
        );
    });
}

#[test]
fn test_empty_file_is_fine() {
    let (_d, path) = empty_config_path();
    fs::write(&path, "").unwrap();
    with_test_env(vec![], &path, || {
        let config = Config::load().unwrap();
        assert!(config.anthropic_key.is_none());
    });
}

// --- default config path resolution (regression: must be ~/.config/musubi/config.toml,
//     not Apple's Application Support, on macOS) ---

#[test]
fn test_default_config_path_uses_xdg_under_home() {
    // Point HOME at a tempdir, place the config at $HOME/.config/musubi/config.toml,
    // unset MUSUBI_CONFIG and XDG_CONFIG_HOME entirely. The file MUST be picked up.
    let home = TempDir::new().unwrap();
    let cfg_dir = home.path().join(".config").join("musubi");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("config.toml"),
        r#"anthropic_api_key = "from-default-path""#,
    )
    .unwrap();

    temp_env::with_vars(
        [
            ("ANTHROPIC_API_KEY", None::<String>),
            ("OPENAI_API_KEY", None),
            ("MUSUBI_LINKS_DIR", Some("/tmp/links".to_string())),
            ("MUSUBI_NOW_DIR", Some("/tmp/now".to_string())),
            ("MUSUBI_CONFIG", None),
            ("XDG_CONFIG_HOME", None),
            ("HOME", Some(home.path().to_string_lossy().into_owned())),
        ],
        || {
            let config = Config::load().unwrap();
            assert_eq!(
                config.anthropic_key.as_deref(),
                Some("from-default-path"),
                "default path resolution must land on $HOME/.config/musubi/config.toml"
            );
        },
    );
}

#[test]
fn test_xdg_config_home_is_respected() {
    let xdg = TempDir::new().unwrap();
    let cfg_dir = xdg.path().join("musubi");
    fs::create_dir_all(&cfg_dir).unwrap();
    fs::write(
        cfg_dir.join("config.toml"),
        r#"anthropic_api_key = "from-xdg""#,
    )
    .unwrap();

    temp_env::with_vars(
        [
            ("ANTHROPIC_API_KEY", None::<String>),
            ("OPENAI_API_KEY", None),
            ("MUSUBI_LINKS_DIR", Some("/tmp/links".to_string())),
            ("MUSUBI_NOW_DIR", Some("/tmp/now".to_string())),
            ("MUSUBI_CONFIG", None),
            ("XDG_CONFIG_HOME", Some(xdg.path().to_string_lossy().into_owned())),
        ],
        || {
            let config = Config::load().unwrap();
            assert_eq!(config.anthropic_key.as_deref(), Some("from-xdg"));
        },
    );
}

#[test]
fn test_unknown_keys_in_file_are_rejected() {
    // Catch typos like `anthropic_key` instead of `anthropic_api_key`.
    let (_d, path) = empty_config_path();
    fs::write(&path, r#"anthropic_key = "oops""#).unwrap();
    with_test_env(vec![], &path, || {
        assert!(Config::load().is_err());
    });
}
