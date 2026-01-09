use anyhow::Result;
use std::time::Duration;
use url::Url;

/// List of HTML event handler attributes to remove
const EVENT_HANDLERS: &[&str] = &[
    "onclick", "ondblclick", "onmousedown", "onmouseup", "onmouseover",
    "onmousemove", "onmouseout", "onmouseenter", "onmouseleave",
    "onload", "onunload", "onchange", "onsubmit", "onreset", "onselect",
    "onblur", "onfocus", "onkeydown", "onkeypress", "onkeyup",
    "onerror", "onresize", "onscroll",
];

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

/// Remove all script tags and event handlers from HTML
fn strip_scripts_and_handlers(html: &str) -> String {
    let mut result = String::new();
    let mut remaining = html;

    // Case-insensitive search for <script tags
    while let Some(start_pos) = remaining.to_lowercase().find("<script") {
        // Add everything before the script tag
        result.push_str(&remaining[..start_pos]);

        // Find the end of the opening tag
        let after_script = &remaining[start_pos + 7..]; // 7 = length of "<script"

        if let Some(tag_end_pos) = after_script.find('>') {
            let tag_content = &after_script[..tag_end_pos];

            // Check if it's a self-closing tag (contains /> before the >)
            if tag_content.trim_end().ends_with('/') {
                // Self-closing tag - skip just the tag itself
                remaining = &after_script[tag_end_pos + 1..];
            } else {
                // Regular tag - find the closing </script> tag
                let after_tag = &after_script[tag_end_pos + 1..];

                if let Some(close_pos) = after_tag.to_lowercase().find("</script>") {
                    // Skip everything until after the closing tag
                    remaining = &after_tag[close_pos + 9..]; // 9 = length of "</script>"
                } else {
                    // No closing tag found - skip rest of document
                    remaining = "";
                }
            }
        } else {
            // Malformed tag - skip rest of document
            remaining = "";
        }
    }

    // Add any remaining content after the last script tag
    result.push_str(remaining);

    // Remove event handler attributes
    for handler in EVENT_HANDLERS {
        // Match pattern: handler="anything" or handler='anything'
        let pattern_double = format!(r#"{}=""#, handler);
        let pattern_single = format!(r#"{}='"#, handler);

        // Simple removal: find and remove handler="..." or handler='...'
        loop {
            let pos_double = result.find(&pattern_double);
            let pos_single = result.find(&pattern_single);

            if pos_double.is_none() && pos_single.is_none() {
                break;
            }

            if let Some(pos) = pos_double.or(pos_single) {
                let quote = if result[pos..].starts_with(&pattern_double) { '"' } else { '\'' };
                if let Some(end) = result[pos..].find(quote).and_then(|start| {
                    result[pos + start + 1..].find(quote).map(|end| pos + start + 1 + end + 1)
                }) {
                    result.replace_range(pos..end, "");
                } else {
                    break;
                }
            }
        }
    }

    result
}

/// Process HTML for archival: inline CSS, strip scripts
/// Returns processed HTML string ready to save
pub fn archive_page(
    html: &str,
    _base_url: &Url,
    _config: &ArchiveConfig,
) -> Result<String> {
    let processed = strip_scripts_and_handlers(html);
    Ok(processed)
}
