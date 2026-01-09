# Archive Mode Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add archive mode to save self-contained HTML versions alongside markdown summaries for offline reading.

**Architecture:** Create new `src/archive.rs` module that processes HTML by inlining external CSS and stripping scripts. Integrate into main flow after markdown save with graceful degradation. Add `-a`/`--archive` CLI flags.

**Tech Stack:** Rust, scraper (HTML parsing), reqwest (CSS fetching), url (URL resolution)

---

## Task 1: Add Archive Module Structure

**Files:**
- Create: `src/archive.rs`
- Modify: `src/lib.rs:1-8`

**Step 1: Write failing test for ArchiveConfig default values**

Create `tests/archive_tests.rs`:

```rust
use musubi::archive::ArchiveConfig;
use std::time::Duration;

#[test]
fn test_archive_config_defaults() {
    let config = ArchiveConfig::default();
    assert_eq!(config.css_timeout, Duration::from_secs(10));
    assert_eq!(config.css_max_size, 5 * 1024 * 1024);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_archive_config_defaults`
Expected: FAIL with "no `archive` in the root"

**Step 3: Create archive module with ArchiveConfig**

Create `src/archive.rs`:

```rust
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
    base_url: &Url,
    config: &ArchiveConfig,
) -> Result<String> {
    // TODO: implement in next tasks
    Ok(html.to_string())
}
```

**Step 4: Add archive module to lib.rs**

Modify `src/lib.rs`:

```rust
pub mod archive;
pub mod config;
pub mod error;
pub mod fetch;
pub mod parse;
pub mod summarize;
pub mod writer;

pub use error::MusubiError;
```

**Step 5: Run test to verify it passes**

Run: `cargo test test_archive_config_defaults`
Expected: PASS

**Step 6: Commit**

```bash
git add src/archive.rs src/lib.rs tests/archive_tests.rs
git commit -m "feat(archive): add archive module structure with ArchiveConfig"
```

---

## Task 2: Implement Script Stripping

**Files:**
- Modify: `src/archive.rs:1-end`
- Modify: `tests/archive_tests.rs:1-end`

**Step 1: Write failing test for script tag removal**

Add to `tests/archive_tests.rs`:

```rust
use musubi::archive::archive_page;
use url::Url;

#[test]
fn test_strips_script_tags() {
    let html = r#"<html><body>
        <p>Content</p>
        <script>alert('hello');</script>
        <script src="external.js"></script>
        <p>More content</p>
    </body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.contains("<script"));
    assert!(result.contains("Content"));
    assert!(result.contains("More content"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_strips_script_tags`
Expected: FAIL with assertion - scripts still present

**Step 3: Implement script stripping**

Add to `src/archive.rs`:

```rust
use scraper::{Html, Selector};

/// Remove all script tags from HTML document
fn strip_scripts(html: &str) -> String {
    let document = Html::parse_document(html);
    let script_selector = Selector::parse("script").unwrap();

    let mut result = html.to_string();

    // Find all script tags and remove them
    let scripts: Vec<_> = document.select(&script_selector).collect();
    for script in scripts.iter().rev() {
        let html_str = script.html();
        result = result.replace(&format!("<script{}", &html_str[7..]), "");
    }

    // Simple approach: remove all <script> tags using regex-like replacement
    // This is a placeholder - we'll improve with proper HTML manipulation
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
pub fn archive_page(
    html: &str,
    base_url: &Url,
    config: &ArchiveConfig,
) -> Result<String> {
    let processed = strip_scripts(html);
    Ok(processed)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_strips_script_tags`
Expected: PASS

**Step 5: Commit**

```bash
git add src/archive.rs tests/archive_tests.rs
git commit -m "feat(archive): implement script tag stripping"
```

---

## Task 3: Implement Event Handler Stripping

**Files:**
- Modify: `src/archive.rs:1-end`
- Modify: `tests/archive_tests.rs:1-end`

**Step 1: Write failing test for event handler removal**

Add to `tests/archive_tests.rs`:

```rust
#[test]
fn test_strips_event_handlers() {
    let html = r#"<html><body>
        <button onclick="alert('click')">Click me</button>
        <div onload="init()" onmouseover="hover()">Content</div>
        <a href="#" onmousedown="track()">Link</a>
    </body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(!result.contains("onclick"));
    assert!(!result.contains("onload"));
    assert!(!result.contains("onmouseover"));
    assert!(!result.contains("onmousedown"));
    assert!(result.contains("Click me"));
    assert!(result.contains("Content"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_strips_event_handlers`
Expected: FAIL with assertion - event handlers still present

**Step 3: Implement event handler stripping**

Modify `src/archive.rs`, update `strip_scripts` to `strip_scripts_and_handlers`:

```rust
/// List of HTML event handler attributes to remove
const EVENT_HANDLERS: &[&str] = &[
    "onclick", "ondblclick", "onmousedown", "onmouseup", "onmouseover",
    "onmousemove", "onmouseout", "onmouseenter", "onmouseleave",
    "onload", "onunload", "onchange", "onsubmit", "onreset", "onselect",
    "onblur", "onfocus", "onkeydown", "onkeypress", "onkeyup",
    "onerror", "onresize", "onscroll",
];

/// Remove all script tags and event handlers from HTML
fn strip_scripts_and_handlers(html: &str) -> String {
    let mut result = html.to_string();

    // Remove script tags
    result = result
        .split("<script")
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_string()
            } else {
                if let Some(end_pos) = part.find("</script>") {
                    part[end_pos + 9..].to_string()
                } else {
                    String::new()
                }
            }
        })
        .collect::<String>();

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

pub fn archive_page(
    html: &str,
    base_url: &Url,
    config: &ArchiveConfig,
) -> Result<String> {
    let processed = strip_scripts_and_handlers(html);
    Ok(processed)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_strips_event_handlers`
Expected: PASS

**Step 5: Commit**

```bash
git add src/archive.rs tests/archive_tests.rs
git commit -m "feat(archive): implement event handler stripping"
```

---

## Task 4: Implement CSS Link Detection

**Files:**
- Modify: `src/archive.rs:1-end`
- Modify: `tests/archive_tests.rs:1-end`

**Step 1: Write failing test for finding CSS links**

Add to `tests/archive_tests.rs`:

```rust
#[test]
fn test_finds_css_links() {
    let html = r#"<html><head>
        <link rel="stylesheet" href="styles.css">
        <link rel="stylesheet" href="https://cdn.example.com/theme.css">
        <link rel="icon" href="favicon.ico">
        <style>body { color: red; }</style>
    </head><body></body></html>"#;

    let base_url = Url::parse("https://example.com/page").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    // Should inline CSS (we'll verify the full behavior later)
    // For now, just verify function doesn't crash
    assert!(result.len() > 0);
}
```

**Step 2: Run test to verify it passes (placeholder)**

Run: `cargo test test_finds_css_links`
Expected: PASS (just checking structure for now)

**Step 3: Implement CSS link detection helper**

Add to `src/archive.rs`:

```rust
use scraper::ElementRef;

/// Find all external CSS links in HTML
fn find_css_links(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("link[rel='stylesheet']").unwrap();

    document
        .select(&link_selector)
        .filter_map(|element| {
            element.value().attr("href").map(|s| s.to_string())
        })
        .collect()
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
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_find_css_links_basic`
Expected: PASS

**Step 5: Commit**

```bash
git add src/archive.rs tests/archive_tests.rs
git commit -m "feat(archive): implement CSS link detection"
```

---

## Task 5: Implement URL Resolution

**Files:**
- Modify: `src/archive.rs:1-end`

**Step 1: Write failing test for URL resolution**

Add to `src/archive.rs` tests section:

```rust
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
```

**Step 2: Run tests to verify they pass**

Run: `cargo test test_resolve_css_url`
Expected: PASS (url::Url::join handles all these cases)

**Step 3: Commit**

```bash
git add src/archive.rs
git commit -m "test(archive): add URL resolution tests"
```

---

## Task 6: Implement CSS Fetching

**Files:**
- Modify: `src/archive.rs:1-end`

**Step 1: Write test for CSS fetch helper (will mock later)**

Add to `src/archive.rs`:

```rust
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
```

**Step 2: No test needed (uses network, will test integration)**

This function uses real network calls, so we'll test it in integration tests later.

**Step 3: Commit**

```bash
git add src/archive.rs
git commit -m "feat(archive): implement CSS fetching with timeout and size limits"
```

---

## Task 7: Implement CSS Inlining Core Logic

**Files:**
- Modify: `src/archive.rs:1-end`
- Modify: `tests/archive_tests.rs:1-end`

**Step 1: Write test for CSS inlining (mock HTML)**

Add to `tests/archive_tests.rs`:

```rust
#[test]
fn test_inline_css_preserves_inline_styles() {
    let html = r#"<html><head>
        <style>body { color: blue; }</style>
    </head><body></body></html>"#;

    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();
    let result = archive_page(html, &base_url, &config).unwrap();

    assert!(result.contains("body { color: blue; }"));
    assert!(result.contains("<style>"));
}
```

**Step 2: Run test to verify it passes**

Run: `cargo test test_inline_css_preserves_inline_styles`
Expected: PASS (current code doesn't break inline styles)

**Step 3: Implement CSS inlining logic**

Add to `src/archive.rs`:

```rust
/// Inline external CSS stylesheets into HTML
/// Returns modified HTML and list of failures (url, error_message)
fn inline_stylesheets(
    html: &str,
    base_url: &Url,
    config: &ArchiveConfig,
) -> (String, Vec<(String, String)>) {
    let document = Html::parse_document(html);
    let link_selector = Selector::parse("link[rel='stylesheet']").unwrap();

    let mut result = html.to_string();
    let mut failures = Vec::new();

    // Collect all stylesheet links
    let links: Vec<_> = document
        .select(&link_selector)
        .filter_map(|element| {
            element.value().attr("href").map(|href| {
                (href.to_string(), element.html())
            })
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
```

**Step 4: Update archive_page to use inlining**

Modify `archive_page` in `src/archive.rs`:

```rust
pub fn archive_page(
    html: &str,
    base_url: &Url,
    config: &ArchiveConfig,
) -> Result<String> {
    // Strip scripts and handlers
    let mut processed = strip_scripts_and_handlers(html);

    // Inline CSS
    let (inlined, failures) = inline_stylesheets(&processed, base_url, config);
    processed = inlined;

    // Log failures to stderr
    for (url, error) in failures {
        eprintln!("Warning: Failed to fetch CSS: {}: {}", url, error);
    }

    Ok(processed)
}
```

**Step 5: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 6: Commit**

```bash
git add src/archive.rs tests/archive_tests.rs
git commit -m "feat(archive): implement CSS inlining with failure handling"
```

---

## Task 8: Add CLI Flag for Archive Mode

**Files:**
- Modify: `src/main.rs:10-20`

**Step 1: Add --archive flag to CLI**

Modify `src/main.rs` Cli struct:

```rust
#[derive(Parser)]
#[command(name = "musubi")]
#[command(about = "Save and summarize web links to markdown", long_about = None)]
struct Cli {
    /// URL to save
    url: String,

    /// Override links directory (default: $MUSUBI_LINKS_DIR or ~/links)
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Save archived HTML version alongside markdown summary
    #[arg(short = 'a', long = "archive")]
    archive: bool,
}
```

**Step 2: Build to verify it compiles**

Run: `cargo build`
Expected: SUCCESS

**Step 3: Test help output**

Run: `cargo run -- --help`
Expected: Should show `-a, --archive` option

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(cli): add --archive/-a flag for HTML archival"
```

---

## Task 9: Integrate Archive Mode into Main Flow

**Files:**
- Modify: `src/main.rs:70-85`
- Modify: `src/writer.rs:92-end`

**Step 1: Add helper to generate HTML filename**

Add to `src/writer.rs`:

```rust
/// Generate HTML filename from markdown path
/// Example: "2026-01-08 Title.md" -> "2026-01-08 Title.html"
pub fn get_html_path(md_path: &Path) -> PathBuf {
    md_path.with_extension("html")
}

#[cfg(test)]
mod tests {
    use super::*;

    // ... existing tests ...

    #[test]
    fn test_get_html_path() {
        let md_path = PathBuf::from("/links/2026-01-08 Title.md");
        let html_path = get_html_path(&md_path);
        assert_eq!(html_path, PathBuf::from("/links/2026-01-08 Title.html"));
    }
}
```

**Step 2: Run test**

Run: `cargo test test_get_html_path`
Expected: PASS

**Step 3: Add archive processing to main**

Modify `src/main.rs`, add after markdown writing (around line 82):

```rust
use musubi::archive;
use std::fs;

// ... existing code ...

    println!("✓ Saved: {}", file_path.display());

    // Archive HTML if requested
    if cli.archive {
        match archive::archive_page(
            &page.html,
            &url::Url::parse(&page.cleaned_url)?,
            &archive::ArchiveConfig::default(),
        ) {
            Ok(archived_html) => {
                let html_path = writer::get_html_path(&file_path);
                match fs::write(&html_path, archived_html) {
                    Ok(_) => println!("✓ Archived: {}", html_path.display()),
                    Err(e) => eprintln!("⚠ Failed to write archive: {}", e),
                }
            }
            Err(e) => {
                eprintln!("⚠ Failed to archive page: {}", e);
                // Markdown already saved, continue
            }
        }
    }

    Ok(())
}
```

**Step 4: Build and verify**

Run: `cargo build`
Expected: SUCCESS

**Step 5: Commit**

```bash
git add src/main.rs src/writer.rs
git commit -m "feat(archive): integrate archive mode into main flow"
```

---

## Task 10: Add Integration Test

**Files:**
- Create: `tests/archive_integration_tests.rs`

**Step 1: Write integration test (minimal, no network)**

Create `tests/archive_integration_tests.rs`:

```rust
use musubi::archive::{archive_page, ArchiveConfig};
use url::Url;

#[test]
fn test_archive_page_end_to_end() {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <style>body { margin: 0; }</style>
    <script>alert('removed');</script>
</head>
<body onclick="removed()">
    <h1>Test Content</h1>
    <p>This is a test paragraph.</p>
</body>
</html>"#;

    let base_url = Url::parse("https://example.com/page").unwrap();
    let config = ArchiveConfig::default();

    let result = archive_page(html, &base_url, &config).unwrap();

    // Verify scripts removed
    assert!(!result.contains("<script"));
    assert!(!result.contains("alert"));

    // Verify event handlers removed
    assert!(!result.contains("onclick"));

    // Verify content preserved
    assert!(result.contains("Test Content"));
    assert!(result.contains("This is a test paragraph"));

    // Verify inline styles preserved
    assert!(result.contains("body { margin: 0; }"));
}

#[test]
fn test_archive_with_no_external_resources() {
    let html = r#"<html><body><p>Simple page</p></body></html>"#;
    let base_url = Url::parse("https://example.com").unwrap();
    let config = ArchiveConfig::default();

    let result = archive_page(html, &base_url, &config).unwrap();
    assert!(result.contains("Simple page"));
}
```

**Step 2: Run tests**

Run: `cargo test archive_integration`
Expected: All tests pass

**Step 3: Commit**

```bash
git add tests/archive_integration_tests.rs
git commit -m "test(archive): add integration tests"
```

---

## Task 11: Add Manual Testing Reminder to README

**Files:**
- Modify: `README.md:28-35`

**Step 1: Add archive mode to usage section**

Modify `README.md` usage section:

```markdown
## Usage

```bash
# Basic usage
musubi https://example.com/article

# Save with archive mode (HTML + markdown)
musubi --archive https://example.com/article
musubi -a https://example.com/article

# Override links directory
musubi https://example.com/article --dir ./my-links

# Combine flags
musubi -a https://example.com/article --dir ./my-links
```
```

**Step 2: Add archive mode to features**

Modify `README.md` features section:

```markdown
## Features

- Automatic tracking parameter removal (utm_*, fbclid, etc.)
- LLM-generated summaries using Claude or ChatGPT
- Automatic tag generation
- Archive mode for offline reading (saves self-contained HTML)
- Graceful degradation (saves without summary if LLM fails)
- Duplicate filename handling
```

**Step 3: Add archive mode to output format section**

Add after output format section in `README.md`:

```markdown
### Archive Mode Output

With `-a` or `--archive` flag, saves both markdown and HTML:

```
~/links/
  2025-01-08 Example Article.md    # Summary with tags
  2025-01-08 Example Article.html  # Self-contained archive
```

The HTML file includes:
- Inlined external CSS (no network requests needed)
- Scripts removed (static, safe page)
- Event handlers stripped
- Original content preserved
```

**Step 4: Commit**

```bash
git add README.md
git commit -m "docs: add archive mode documentation to README"
```

---

## Task 12: Manual Testing Checklist

**Manual Tests to Run:**

Run these tests with real websites to verify archive mode works correctly:

### Test 1: Wikipedia Article

```bash
cargo run -- -a "https://en.wikipedia.org/wiki/Rust_(programming_language)"
```

**Verify:**
- [ ] Both .md and .html files created
- [ ] HTML opens in browser without network
- [ ] Page layout looks correct
- [ ] No JavaScript errors in console
- [ ] Inline Wikipedia styles work

### Test 2: News Article

```bash
cargo run -- -a "https://www.theguardian.com/technology"
```

**Verify:**
- [ ] Both files created
- [ ] HTML readable offline
- [ ] External CSS warnings logged to stderr
- [ ] Content still readable despite CSS failures

### Test 3: Blog Post

```bash
cargo run -- -a "https://blog.rust-lang.org/"
```

**Verify:**
- [ ] Both files created
- [ ] HTML layout preserved
- [ ] Images may be broken (expected - not implemented)
- [ ] Text content readable

### Test 4: Documentation Site

```bash
cargo run -- -a "https://docs.rs/"
```

**Verify:**
- [ ] Both files created
- [ ] Code syntax highlighting works (if CSS inline succeeds)
- [ ] Navigation styles work

### Test 5: Simple Page

```bash
cargo run -- -a "https://example.com"
```

**Verify:**
- [ ] Both files created
- [ ] HTML is minimal and clean
- [ ] No errors or warnings

### Test 6: Without Archive Flag

```bash
cargo run -- "https://example.com"
```

**Verify:**
- [ ] Only .md file created
- [ ] No .html file
- [ ] Existing behavior unchanged

**Document Results:**

After manual testing, create a brief report:

```bash
echo "# Manual Testing Results" > docs/plans/2026-01-08-archive-testing-results.md
echo "" >> docs/plans/2026-01-08-archive-testing-results.md
echo "Date: $(date)" >> docs/plans/2026-01-08-archive-testing-results.md
echo "" >> docs/plans/2026-01-08-archive-testing-results.md
echo "## Test Results" >> docs/plans/2026-01-08-archive-testing-results.md
echo "" >> docs/plans/2026-01-08-archive-testing-results.md
echo "- [ ] Wikipedia - " >> docs/plans/2026-01-08-archive-testing-results.md
echo "- [ ] News site - " >> docs/plans/2026-01-08-archive-testing-results.md
echo "- [ ] Blog post - " >> docs/plans/2026-01-08-archive-testing-results.md
echo "- [ ] Documentation - " >> docs/plans/2026-01-08-archive-testing-results.md
echo "- [ ] Simple page - " >> docs/plans/2026-01-08-archive-testing-results.md
echo "- [ ] Without flag - " >> docs/plans/2026-01-08-archive-testing-results.md
```

Fill in results and commit:

```bash
git add docs/plans/2026-01-08-archive-testing-results.md
git commit -m "docs: add manual testing results"
```

---

## Summary

**Total Tasks:** 12
**Estimated Time:** 2-3 hours for implementation + testing
**Key Commits:** ~12-15 commits following TDD approach

**Architecture Decisions:**
- New `archive` module keeps concerns separated
- Graceful degradation matches existing LLM failure pattern
- CLI integration is minimal and non-breaking
- HTML/markdown files share same base name for easy association

**Testing Strategy:**
- Unit tests for core functions (config, stripping, detection)
- Integration tests for end-to-end flow
- Manual tests with real websites (Wikipedia, news, blogs, docs)

**Success Criteria:**
- All automated tests pass
- `musubi -a URL` creates both .md and .html
- HTML files open offline with preserved layout
- Failures log helpful warnings
- Existing behavior (without `-a`) unchanged
