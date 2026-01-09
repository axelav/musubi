use anyhow::Result;
use std::time::Duration;
use url::Url;

/// List of HTML event handler attributes to remove
const EVENT_HANDLERS: &[&str] = &[
    "onclick",
    "ondblclick",
    "onmousedown",
    "onmouseup",
    "onmouseover",
    "onmousemove",
    "onmouseout",
    "onmouseenter",
    "onmouseleave",
    "onload",
    "onunload",
    "onchange",
    "onsubmit",
    "onreset",
    "onselect",
    "onblur",
    "onfocus",
    "onkeydown",
    "onkeypress",
    "onkeyup",
    "onerror",
    "onresize",
    "onscroll",
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
                // Determine quote type and pattern length
                let (quote, pattern_len) = if result[pos..].starts_with(&pattern_double) {
                    ('"', pattern_double.len())
                } else {
                    ('\'', pattern_single.len())
                };

                // Calculate search_start as pos + pattern_len
                let search_start = pos + pattern_len;

                // Find closing quote from search_start
                if let Some(closing_quote_offset) = result[search_start..].find(quote) {
                    // Calculate end properly: search_start + offset + 1 (to include the closing quote)
                    let end = search_start + closing_quote_offset + 1;
                    result.replace_range(pos..end, "");
                } else {
                    // No closing quote found - break to prevent infinite loop
                    break;
                }
            } else {
                // Neither position found - break to prevent infinite loop
                break;
            }
        }
    }

    result
}

// FIXME: This function is not used currently

/// Find all external CSS links in HTML
fn find_css_links(html: &str) -> Vec<String> {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let link_selector = Selector::parse("link[rel='stylesheet']").unwrap();

    document
        .select(&link_selector)
        .filter_map(|element| element.value().attr("href").map(|s| s.to_string()))
        .collect()
}

/// Fetch CSS content from URL with timeout and size limits
/// Returns Ok(css_content) on success, Err with description on failure
fn fetch_css(url: &Url, config: &ArchiveConfig) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Musubi/0.1)")
        .timeout(config.css_timeout)
        .build()?;

    let response = client.get(url.as_str()).send()?;

    // Check size before reading full body
    if let Some(content_length) = response.content_length() {
        if content_length as usize > config.css_max_size {
            anyhow::bail!(
                "CSS too large: {} bytes (max {})",
                content_length,
                config.css_max_size
            );
        }
    }

    let css = response.text()?;
    // Normalize line endings (CRLF -> LF) to prevent ^M characters on Unix
    let css = css.replace("\r\n", "\n");

    // Check actual size
    if css.len() > config.css_max_size {
        anyhow::bail!(
            "CSS too large: {} bytes (max {})",
            css.len(),
            config.css_max_size
        );
    }

    Ok(css)
}

/// Inline external CSS stylesheets into HTML
/// Returns modified HTML and list of failures (url, error_message)
fn inline_stylesheets(
    html: &str,
    base_url: &Url,
    config: &ArchiveConfig,
) -> (String, Vec<(String, String)>) {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);
    let link_selector = Selector::parse("link[rel='stylesheet']").unwrap();

    let mut result = html.to_string();
    let mut failures = Vec::new();

    // Collect all stylesheet links
    let links: Vec<_> = document
        .select(&link_selector)
        .filter_map(|element| {
            element
                .value()
                .attr("href")
                .map(|href| (href.to_string(), element.html()))
        })
        .collect();

    // Process each link
    for (href, original_tag) in links {
        // Resolve URL
        let css_url = match base_url.join(&href) {
            Ok(url) => url,
            Err(e) => {
                failures.push((href.clone(), format!("Invalid URL: {}", e)));
                continue;
            }
        };

        // Fetch CSS
        match fetch_css(&css_url, config) {
            Ok(css_content) => {
                // Create inline style tag
                let inline_tag = format!(
                    "<style>/* from: {} */\n{}</style>",
                    css_url.as_str(),
                    css_content
                );

                // Replace link tag with inline style
                result = result.replace(&original_tag, &inline_tag);
            }
            Err(e) => {
                // Record failure and remove link tag
                let error_msg = format!("{}", e);
                failures.push((css_url.to_string(), error_msg.clone()));

                // Add HTML comment and remove link
                let comment = format!(
                    "<!-- Failed to inline stylesheet: {} ({}) -->",
                    css_url.as_str(),
                    error_msg
                );
                result = result.replace(&original_tag, &comment);
            }
        }
    }

    (result, failures)
}

/// Process HTML for archival: inline CSS, strip scripts
/// Returns processed HTML string ready to save
pub fn archive_page(html: &str, base_url: &Url, config: &ArchiveConfig) -> Result<String> {
    // Normalize line endings first (CRLF -> LF)
    let normalized = html.replace("\r\n", "\n");

    // Strip scripts and handlers
    let mut processed = strip_scripts_and_handlers(&normalized);

    // Inline CSS
    let (inlined, failures) = inline_stylesheets(&processed, base_url, config);
    processed = inlined;

    // Log failures to stderr
    for (url, error) in failures {
        eprintln!("Warning: Failed to fetch CSS: {}: {}", url, error);
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_css_links_basic() {
        let html = r#"<html><head>
            <link rel="stylesheet" href="style.css">
            <link rel="stylesheet" href="/css/theme.css">
            <link rel="icon" href="favicon.ico">
        </head></html>"#;

        let links = find_css_links(html);
        assert_eq!(links.len(), 2);
        assert!(links.contains(&"style.css".to_string()));
        assert!(links.contains(&"/css/theme.css".to_string()));
    }

    #[test]
    fn test_resolve_css_url_relative() {
        let base = Url::parse("https://example.com/blog/post").unwrap();
        let relative = "styles.css";
        let resolved = base.join(relative).unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/blog/styles.css");
    }

    #[test]
    fn test_resolve_css_url_absolute_path() {
        let base = Url::parse("https://example.com/blog/post").unwrap();
        let absolute = "/css/style.css";
        let resolved = base.join(absolute).unwrap();
        assert_eq!(resolved.as_str(), "https://example.com/css/style.css");
    }

    #[test]
    fn test_resolve_css_url_protocol_relative() {
        let base = Url::parse("https://example.com/page").unwrap();
        let proto_rel = "//cdn.example.com/style.css";
        let resolved = base.join(proto_rel).unwrap();
        assert_eq!(resolved.as_str(), "https://cdn.example.com/style.css");
    }

    #[test]
    fn test_resolve_css_url_already_absolute() {
        let base = Url::parse("https://example.com/page").unwrap();
        let absolute = "https://cdn.example.com/style.css";
        let resolved = base.join(absolute).unwrap();
        assert_eq!(resolved.as_str(), "https://cdn.example.com/style.css");
    }
}
