use musubi::archive::ArchiveConfig;
use std::time::Duration;

#[test]
fn test_archive_config_defaults() {
    let config = ArchiveConfig::default();
    assert_eq!(config.css_timeout, Duration::from_secs(10));
    assert_eq!(config.css_max_size, 5 * 1024 * 1024);
}
