use musubi::config::Config;
use std::env;

#[test]
fn test_config_reads_anthropic_key() {
    temp_env::with_vars([("ANTHROPIC_API_KEY", Some("test-key-123"))], || {
        let config = Config::from_env().unwrap();
        assert_eq!(config.anthropic_key, Some("test-key-123".to_string()));
    });
}

#[test]
fn test_config_reads_openai_key() {
    temp_env::with_vars([("OPENAI_API_KEY", Some("test-openai-key"))], || {
        let config = Config::from_env().unwrap();
        assert_eq!(config.openai_key, Some("test-openai-key".to_string()));
    });
}

#[test]
fn test_has_llm_key_with_anthropic() {
    temp_env::with_vars(
        [
            ("ANTHROPIC_API_KEY", Some("test-key")),
            ("OPENAI_API_KEY", None),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert!(config.has_llm_key());
        },
    );
}

#[test]
fn test_has_llm_key_with_openai() {
    temp_env::with_vars(
        [
            ("ANTHROPIC_API_KEY", None),
            ("OPENAI_API_KEY", Some("test-key")),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert!(config.has_llm_key());
        },
    );
}

#[test]
fn test_has_llm_key_returns_false_when_no_keys() {
    temp_env::with_vars(
        [
            ("ANTHROPIC_API_KEY", None::<&str>),
            ("OPENAI_API_KEY", None::<&str>),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert!(!config.has_llm_key());
        },
    );
}

#[test]
fn test_custom_links_dir() {
    temp_env::with_vars(
        [("MUSUBI_LINKS_DIR", Some("/custom/path/to/links"))],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.links_dir.to_str().unwrap(), "/custom/path/to/links");
        },
    );
}

#[test]
fn test_config_defaults_to_home_links() {
    temp_env::with_vars([("MUSUBI_LINKS_DIR", None::<&str>)], || {
        let config = Config::from_env().unwrap();
        let home = env::var("HOME").unwrap();
        let expected = std::path::PathBuf::from(home).join("links");
        assert_eq!(config.links_dir, expected);
    });
}

#[test]
fn test_custom_now_dir() {
    temp_env::with_vars([("MUSUBI_NOW_DIR", Some("/custom/path/to/now"))], || {
        let config = Config::from_env().unwrap();
        assert_eq!(config.now_dir.to_str().unwrap(), "/custom/path/to/now");
    });
}

#[test]
fn test_config_defaults_to_home_now() {
    temp_env::with_vars([("MUSUBI_NOW_DIR", None::<&str>)], || {
        let config = Config::from_env().unwrap();
        let home = env::var("HOME").unwrap();
        let expected = std::path::PathBuf::from(home).join("now");
        assert_eq!(config.now_dir, expected);
    });
}

#[test]
fn test_config_does_not_require_home_when_both_dirs_set() {
    temp_env::with_vars(
        [
            ("MUSUBI_LINKS_DIR", Some("/custom/links")),
            ("MUSUBI_NOW_DIR", Some("/custom/now")),
            ("HOME", None),
        ],
        || {
            let config = Config::from_env().unwrap();
            assert_eq!(config.links_dir.to_str().unwrap(), "/custom/links");
            assert_eq!(config.now_dir.to_str().unwrap(), "/custom/now");
        },
    );
}
