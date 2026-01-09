use anyhow::Result;
use std::time::Duration;
use url::Url;

/// Configuration for HTML archival processing
#[derive(Debug, Clone)]
pub struct ArchiveConfig {
    /// Timeout for fetching each CSS file
    pub css_timeout: Duration,
    /// Maximum size for each CSS file in bytes
    pub css_max_size: usize,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            css_timeout: Duration::from_secs(10),
            css_max_size: 5 * 1024 * 1024, // 5MB
        }
    }
}

/// Process HTML for archival: inline CSS, strip scripts
/// Returns processed HTML string ready to save
pub fn archive_page(
    html: &str,
    _base_url: &Url,
    _config: &ArchiveConfig,
) -> Result<String> {
    // TODO: implement in next tasks
    Ok(html.to_string())
}
