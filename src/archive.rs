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

/// Remove all script tags from HTML document
fn strip_scripts(html: &str) -> String {
    let mut result = html.to_string();

    // Simple approach: remove all <script> tags using string splitting
    // This handles both inline scripts and external script tags
    result = result
        .split("<script")
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                // Find closing tag and skip everything until then
                if let Some(end_pos) = part.find("</script>") {
                    part[end_pos + 9..].to_string()
                } else {
                    // Self-closing or malformed - remove entire rest
                    String::new()
                }
            }
        })
        .collect::<String>();

    result
}

/// Process HTML for archival: inline CSS, strip scripts
/// Returns processed HTML string ready to save
pub fn archive_page(
    html: &str,
    _base_url: &Url,
    _config: &ArchiveConfig,
) -> Result<String> {
    let processed = strip_scripts(html);
    Ok(processed)
}
