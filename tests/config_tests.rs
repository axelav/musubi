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
fn test_config_defaults_to_home_links() {
    env::remove_var("MUSUBI_LINKS_DIR");
    let config = Config::from_env().unwrap();
    let _home = env::var("HOME").unwrap();
    assert!(config.links_dir.to_str().unwrap().contains("links"));
}
