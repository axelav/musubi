use musubi::config::Config;
use std::env;

#[test]
fn test_config_reads_anthropic_key() {
    env::set_var("ANTHROPIC_API_KEY", "test-key-123");
    let config = Config::from_env().unwrap();
    assert_eq!(config.anthropic_key, Some("test-key-123".to_string()));
    env::remove_var("ANTHROPIC_API_KEY");
}

#[test]
fn test_config_reads_openai_key() {
    env::set_var("OPENAI_API_KEY", "test-openai-key");
    let config = Config::from_env().unwrap();
    assert_eq!(config.openai_key, Some("test-openai-key".to_string()));
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_has_llm_key_with_anthropic() {
    env::set_var("ANTHROPIC_API_KEY", "test-key");
    env::remove_var("OPENAI_API_KEY");
    let config = Config::from_env().unwrap();
    assert!(config.has_llm_key());
    env::remove_var("ANTHROPIC_API_KEY");
}

#[test]
fn test_has_llm_key_with_openai() {
    env::remove_var("ANTHROPIC_API_KEY");
    env::set_var("OPENAI_API_KEY", "test-key");
    let config = Config::from_env().unwrap();
    assert!(config.has_llm_key());
    env::remove_var("OPENAI_API_KEY");
}

#[test]
fn test_has_llm_key_returns_false_when_no_keys() {
    env::remove_var("ANTHROPIC_API_KEY");
    env::remove_var("OPENAI_API_KEY");
    let config = Config::from_env().unwrap();
    assert!(!config.has_llm_key());
}

#[test]
fn test_custom_links_dir() {
    env::set_var("MUSUBI_LINKS_DIR", "/custom/path/to/links");
    let config = Config::from_env().unwrap();
    assert_eq!(config.links_dir.to_str().unwrap(), "/custom/path/to/links");
    env::remove_var("MUSUBI_LINKS_DIR");
}

#[test]
fn test_config_defaults_to_home_links() {
    env::remove_var("MUSUBI_LINKS_DIR");
    let config = Config::from_env().unwrap();
    let home = env::var("HOME").unwrap();
    let expected = std::path::PathBuf::from(home).join("links");
    assert_eq!(config.links_dir, expected);
}
