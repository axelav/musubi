# Musubi Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a CLI tool that fetches web pages, generates LLM summaries with tags, and saves them as markdown files.

**Architecture:** Modular library design with separate concerns (fetch, parse, summarize, write) orchestrated by a thin CLI binary. Each module is independently testable with clear interfaces.

**Tech Stack:** Rust, clap (CLI), reqwest (HTTP/LLM API), scraper (HTML), url (parsing), chrono (dates), anyhow (errors), serde/serde_json (JSON)

---

## Task 1: Project Setup & Dependencies

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/error.rs`

**Step 1: Add dependencies to Cargo.toml**

```toml
[package]
name = "musubi"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "musubi"
path = "src/main.rs"

[lib]
name = "musubi"
path = "src/lib.rs"

[dependencies]
clap = { version = "4.5", features = ["derive"] }
reqwest = { version = "0.12", features = ["blocking", "json"] }
scraper = "0.20"
url = "2.5"
chrono = "0.4"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
tempfile = "3.13"
```

**Step 2: Create basic library structure**

Create `src/lib.rs`:
```rust
pub mod config;
pub mod error;
pub mod fetch;
pub mod parse;
pub mod summarize;
pub mod writer;

pub use error::MusubiError;
```

**Step 3: Create error module**

Create `src/error.rs`:
```rust
use std::fmt;

#[derive(Debug)]
pub enum MusubiError {
    Network(String),
    Parse(String),
    Write(String),
    Config(String),
}

impl fmt::Display for MusubiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MusubiError::Network(msg) => write!(f, "Network error: {}", msg),
            MusubiError::Parse(msg) => write!(f, "Parse error: {}", msg),
            MusubiError::Write(msg) => write!(f, "Write error: {}", msg),
            MusubiError::Config(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for MusubiError {}
```

**Step 4: Build to verify dependencies**

Run: `cargo build`
Expected: Successful compilation

**Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/error.rs
git commit -m "feat: add project dependencies and error types"
```

---

## Task 2: Configuration Module

**Files:**
- Create: `src/config.rs`
- Create: `tests/config_tests.rs`

**Step 1: Write the failing test**

Create `tests/config_tests.rs`:
```rust
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
    let home = env::var("HOME").unwrap();
    assert!(config.links_dir.to_str().unwrap().contains("links"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test config_tests`
Expected: FAIL with "module config not found"

**Step 3: Write minimal implementation**

Create `src/config.rs`:
```rust
use std::env;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub anthropic_key: Option<String>,
    pub openai_key: Option<String>,
    pub links_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let anthropic_key = env::var("ANTHROPIC_API_KEY").ok();
        let openai_key = env::var("OPENAI_API_KEY").ok();

        let links_dir = if let Ok(dir) = env::var("MUSUBI_LINKS_DIR") {
            PathBuf::from(dir)
        } else {
            let home = env::var("HOME")
                .context("HOME environment variable not set")?;
            PathBuf::from(home).join("links")
        };

        Ok(Config {
            anthropic_key,
            openai_key,
            links_dir,
        })
    }

    pub fn has_llm_key(&self) -> bool {
        self.anthropic_key.is_some() || self.openai_key.is_some()
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test config_tests`
Expected: PASS (2 tests)

**Step 5: Commit**

```bash
git add src/config.rs tests/config_tests.rs
git commit -m "feat: add configuration module with env var support"
```

---

## Task 3: URL Cleaning

**Files:**
- Create: `src/fetch.rs` (partial)
- Create: `tests/fetch_tests.rs`

**Step 1: Write the failing test**

Create `tests/fetch_tests.rs`:
```rust
use musubi::fetch::clean_url;

#[test]
fn test_clean_url_removes_utm_params() {
    let input = "https://example.com/page?utm_source=twitter&utm_campaign=test&id=123";
    let cleaned = clean_url(input).unwrap();
    assert_eq!(cleaned, "https://example.com/page?id=123");
}

#[test]
fn test_clean_url_removes_fbclid() {
    let input = "https://example.com/page?fbclid=abc123&foo=bar";
    let cleaned = clean_url(input).unwrap();
    assert_eq!(cleaned, "https://example.com/page?foo=bar");
}

#[test]
fn test_clean_url_preserves_functional_params() {
    let input = "https://example.com/search?q=rust&page=2";
    let cleaned = clean_url(input).unwrap();
    assert_eq!(cleaned, "https://example.com/search?q=rust&page=2");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test fetch_tests`
Expected: FAIL with "function clean_url not found"

**Step 3: Write minimal implementation**

Create `src/fetch.rs`:
```rust
use anyhow::{Context, Result};
use url::Url;

const TRACKING_PARAMS: &[&str] = &[
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "fbclid", "gclid", "gclsrc",
    "mc_cid", "mc_eid",
    "_hsenc", "_hsmi",
    "ref", "source",
];

pub fn clean_url(url_str: &str) -> Result<String> {
    let mut url = Url::parse(url_str)
        .context("Failed to parse URL")?;

    let filtered_pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| !TRACKING_PARAMS.contains(&key.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    if filtered_pairs.is_empty() {
        url.set_query(None);
    } else {
        let query_string = filtered_pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        url.set_query(Some(&query_string));
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_url_no_params() {
        let input = "https://example.com/page";
        let cleaned = clean_url(input).unwrap();
        assert_eq!(cleaned, "https://example.com/page");
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test fetch_tests`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/fetch.rs tests/fetch_tests.rs
git commit -m "feat: add URL cleaning to remove tracking parameters"
```

---

## Task 4: HTTP Fetching

**Files:**
- Modify: `src/fetch.rs`
- Modify: `tests/fetch_tests.rs`

**Step 1: Write the failing test**

Add to `tests/fetch_tests.rs`:
```rust
use musubi::fetch::FetchedPage;

#[test]
fn test_fetched_page_structure() {
    // This is a unit test for the struct
    let page = FetchedPage {
        original_url: "https://example.com?utm_source=test".to_string(),
        cleaned_url: "https://example.com".to_string(),
        html: "<html><body>Test</body></html>".to_string(),
    };

    assert_eq!(page.original_url, "https://example.com?utm_source=test");
    assert_eq!(page.cleaned_url, "https://example.com");
    assert!(page.html.contains("Test"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_fetched_page_structure`
Expected: FAIL with "struct FetchedPage not found"

**Step 3: Write minimal implementation**

Add to `src/fetch.rs`:
```rust
#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub original_url: String,
    pub cleaned_url: String,
    pub html: String,
}

pub fn fetch_page(url: &str) -> Result<FetchedPage> {
    let cleaned_url = clean_url(url)?;

    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; Musubi/0.1)")
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(&cleaned_url)
        .send()
        .context(format!("Failed to fetch URL: {}", cleaned_url))?;

    let html = response
        .text()
        .context("Failed to read response body")?;

    Ok(FetchedPage {
        original_url: url.to_string(),
        cleaned_url,
        html,
    })
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_fetched_page_structure`
Expected: PASS

**Step 5: Commit**

```bash
git add src/fetch.rs tests/fetch_tests.rs
git commit -m "feat: add HTTP fetching with user agent"
```

---

## Task 5: HTML Parsing

**Files:**
- Create: `src/parse.rs`
- Create: `tests/parse_tests.rs`

**Step 1: Write the failing test**

Create `tests/parse_tests.rs`:
```rust
use musubi::parse::{extract_metadata, PageMetadata};

#[test]
fn test_extract_title_from_html() {
    let html = r#"
        <html>
            <head><title>Test Page Title</title></head>
            <body>Content</body>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(metadata.title, "Test Page Title");
}

#[test]
fn test_extract_description_from_meta() {
    let html = r#"
        <html>
            <head>
                <title>Test</title>
                <meta name="description" content="This is a test description">
            </head>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(metadata.description, Some("This is a test description".to_string()));
}

#[test]
fn test_extract_og_description() {
    let html = r#"
        <html>
            <head>
                <title>Test</title>
                <meta property="og:description" content="Open Graph description">
            </head>
        </html>
    "#;

    let metadata = extract_metadata(html, "https://example.com").unwrap();
    assert_eq!(metadata.description, Some("Open Graph description".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test parse_tests`
Expected: FAIL with "module parse not found"

**Step 3: Write minimal implementation**

Create `src/parse.rs`:
```rust
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct PageMetadata {
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub fetch_date: DateTime<Utc>,
}

pub fn extract_metadata(html: &str, url: &str) -> Result<PageMetadata> {
    let document = Html::parse_document(html);

    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "Untitled".to_string())
        .trim()
        .to_string();

    // Extract description (try meta description, then og:description)
    let meta_desc_selector = Selector::parse("meta[name='description']").unwrap();
    let og_desc_selector = Selector::parse("meta[property='og:description']").unwrap();

    let description = document
        .select(&meta_desc_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .or_else(|| {
            document
                .select(&og_desc_selector)
                .next()
                .and_then(|el| el.value().attr("content"))
        })
        .map(|s| s.to_string());

    Ok(PageMetadata {
        title,
        description,
        url: url.to_string(),
        fetch_date: Utc::now(),
    })
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test parse_tests`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/parse.rs tests/parse_tests.rs
git commit -m "feat: add HTML metadata extraction with title and description"
```

---

## Task 6: LLM Summarization (Trait & Anthropic)

**Files:**
- Create: `src/summarize.rs`
- Create: `tests/summarize_tests.rs`

**Step 1: Write the failing test**

Create `tests/summarize_tests.rs`:
```rust
use musubi::summarize::Summary;

#[test]
fn test_summary_structure() {
    let summary = Summary {
        text: "This is a test summary. It has multiple sentences.".to_string(),
        tags: vec!["test".to_string(), "rust".to_string()],
    };

    assert!(!summary.text.is_empty());
    assert_eq!(summary.tags.len(), 2);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_summary_structure`
Expected: FAIL with "module summarize not found"

**Step 3: Write minimal implementation**

Create `src/summarize.rs`:
```rust
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub text: String,
    pub tags: Vec<String>,
}

pub trait LlmProvider {
    fn generate_summary(&self, title: &str, content: &str) -> Result<Summary>;
}

pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

impl LlmProvider for AnthropicProvider {
    fn generate_summary(&self, title: &str, content: &str) -> Result<Summary> {
        let truncated_content = truncate_content(content, 4000);

        let prompt = format!(
            r#"Given this webpage:
Title: {}
Content: {}

Generate a JSON response with:
1. A "summary" field containing 2-3 sentences summarizing the main content
2. A "tags" field containing an array of 3-5 relevant topic tags (single words, lowercase)

Respond ONLY with valid JSON in this format:
{{"summary": "...", "tags": ["tag1", "tag2", "tag3"]}}
"#,
            title, truncated_content
        );

        let client = reqwest::blocking::Client::new();
        let request_body = AnthropicRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!(
                "Anthropic API error ({}): {}",
                status,
                error_text
            ));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .context("Failed to parse Anthropic response")?;

        let text = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| anyhow!("No content in Anthropic response"))?;

        // Parse the JSON from the LLM response
        let summary: Summary = serde_json::from_str(&text)
            .context("Failed to parse summary JSON from LLM")?;

        Ok(summary)
    }
}

fn truncate_content(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        content.to_string()
    } else {
        format!("{}...", &content[..max_chars])
    }
}

pub fn create_provider(anthropic_key: Option<String>, _openai_key: Option<String>) -> Result<Box<dyn LlmProvider>> {
    if let Some(key) = anthropic_key {
        Ok(Box::new(AnthropicProvider::new(key)))
    } else {
        Err(anyhow!(
            "No LLM API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable."
        ))
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_summary_structure`
Expected: PASS

**Step 5: Commit**

```bash
git add src/summarize.rs tests/summarize_tests.rs
git commit -m "feat: add LLM summarization with Anthropic provider"
```

---

## Task 7: Markdown File Writer

**Files:**
- Create: `src/writer.rs`
- Create: `tests/writer_tests.rs`

**Step 1: Write the failing test**

Create `tests/writer_tests.rs`:
```rust
use musubi::writer::sanitize_filename;

#[test]
fn test_sanitize_filename_removes_special_chars() {
    let input = "Hello/World: A Test?";
    let sanitized = sanitize_filename(input);
    assert_eq!(sanitized, "Hello-World- A Test-");
}

#[test]
fn test_sanitize_filename_collapses_spaces() {
    let input = "Hello    World";
    let sanitized = sanitize_filename(input);
    assert_eq!(sanitized, "Hello World");
}

#[test]
fn test_sanitize_filename_truncates_long_names() {
    let input = "a".repeat(150);
    let sanitized = sanitize_filename(&input);
    assert!(sanitized.len() <= 100);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test writer_tests`
Expected: FAIL with "module writer not found"

**Step 3: Write minimal implementation**

Create `src/writer.rs`:
```rust
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};

pub fn sanitize_filename(title: &str) -> String {
    let sanitized = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => c,
        })
        .collect::<String>();

    let collapsed = sanitized
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let trimmed = collapsed.trim();

    if trimmed.len() > 100 {
        trimmed[..100].to_string()
    } else {
        trimmed.to_string()
    }
}

fn generate_filename(date: &DateTime<Utc>, title: &str) -> String {
    let date_str = date.format("%Y-%m-%d").to_string();
    let sanitized_title = sanitize_filename(title);
    format!("{} {}.md", date_str, sanitized_title)
}

fn find_available_filename(dir: &Path, base_filename: &str) -> PathBuf {
    let mut path = dir.join(base_filename);
    let mut counter = 2;

    while path.exists() {
        let base_name = base_filename.trim_end_matches(".md");
        let new_filename = format!("{}-{}.md", base_name, counter);
        path = dir.join(new_filename);
        counter += 1;
    }

    path
}

pub fn write_link_file(
    dir: &Path,
    title: &str,
    url: &str,
    date: &DateTime<Utc>,
    summary: Option<&str>,
    tags: &[String],
) -> Result<PathBuf> {
    // Create directory if it doesn't exist
    fs::create_dir_all(dir)
        .context(format!("Failed to create directory: {}", dir.display()))?;

    // Generate filename
    let base_filename = generate_filename(date, title);
    let file_path = find_available_filename(dir, &base_filename);

    // Format date for content
    let iso_date = date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let wiki_date = date.format("%Y-%m-%d").to_string();

    // Build content
    let mut content = String::new();
    content.push_str(&format!("---\n"));
    content.push_str(&format!("title: {}\n", title));
    content.push_str(&format!("date: {}\n", iso_date));
    content.push_str(&format!("url: {}\n", url));
    content.push_str(&format!("---\n\n"));
    content.push_str(&format!("## {}\n\n", title));
    content.push_str(&format!("{}\n\n", url));

    if let Some(summary_text) = summary {
        content.push_str(&format!("{}\n\n", summary_text));
    }

    content.push_str("---\n\n");
    content.push_str(&format!("[[{}]] #links", wiki_date));

    for tag in tags {
        content.push_str(&format!(" #{}", tag));
    }
    content.push('\n');

    // Write file
    fs::write(&file_path, content)
        .context(format!("Failed to write file: {}", file_path.display()))?;

    Ok(file_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename() {
        let date = Utc::now();
        let filename = generate_filename(&date, "Test Title");
        let date_str = date.format("%Y-%m-%d").to_string();
        assert_eq!(filename, format!("{} Test Title.md", date_str));
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test writer_tests`
Expected: PASS (3 tests)

**Step 5: Commit**

```bash
git add src/writer.rs tests/writer_tests.rs
git commit -m "feat: add markdown file writer with filename sanitization"
```

---

## Task 8: CLI Integration

**Files:**
- Modify: `src/main.rs`

**Step 1: Write CLI structure**

Replace contents of `src/main.rs`:
```rust
use anyhow::{Context, Result};
use clap::Parser;
use musubi::config::Config;
use musubi::fetch;
use musubi::parse;
use musubi::summarize;
use musubi::writer;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "musubi")]
#[command(about = "Save and summarize web links to markdown", long_about = None)]
struct Cli {
    /// URL to save
    url: String,

    /// Override links directory (default: $MUSUBI_LINKS_DIR or ~/links)
    #[arg(short, long)]
    dir: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut config = Config::from_env()
        .context("Failed to load configuration")?;

    // Override directory if provided
    if let Some(dir) = cli.dir {
        config.links_dir = dir;
    }

    // Fetch page
    println!("⏳ Fetching: {}", cli.url);
    let page = fetch::fetch_page(&cli.url)
        .context("Failed to fetch page")?;

    // Parse metadata
    let metadata = parse::extract_metadata(&page.html, &page.cleaned_url)
        .context("Failed to extract metadata")?;

    println!("✓ Fetched: {}", metadata.title);

    // Generate summary (optional, graceful degradation)
    let (summary_text, tags) = if config.has_llm_key() {
        match summarize::create_provider(config.anthropic_key, config.openai_key) {
            Ok(provider) => {
                // Extract text content from HTML for summarization
                let text_content = extract_text_content(&page.html);

                match provider.generate_summary(&metadata.title, &text_content) {
                    Ok(summary) => {
                        println!("✓ Generated summary");
                        (Some(summary.text), summary.tags)
                    }
                    Err(e) => {
                        eprintln!("⚠ Could not generate summary: {}", e);
                        (None, vec![])
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠ Could not create LLM provider: {}", e);
                (None, vec![])
            }
        }
    } else {
        eprintln!("⚠ No LLM API key found, saving without summary");
        (None, vec![])
    };

    // Write markdown file
    let file_path = writer::write_link_file(
        &config.links_dir,
        &metadata.title,
        &page.cleaned_url,
        &metadata.fetch_date,
        summary_text.as_deref(),
        &tags,
    )
    .context("Failed to write link file")?;

    println!("✓ Saved: {}", file_path.display());

    Ok(())
}

fn extract_text_content(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Try to get main content (common tags)
    let content_selectors = vec![
        "article",
        "main",
        "[role='main']",
        ".content",
        "#content",
        "body",
    ];

    for selector_str in content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    return text;
                }
            }
        }
    }

    // Fallback: get all text
    document.root_element().text().collect::<Vec<_>>().join(" ")
}
```

**Step 2: Build and test manually**

Run: `cargo build --release`
Expected: Successful compilation

**Step 3: Test with invalid URL (should fail gracefully)**

Run: `cargo run -- "not-a-url"`
Expected: Error message about invalid URL

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate CLI with all modules"
```

---

## Task 9: Integration Test

**Files:**
- Create: `tests/integration_tests.rs`

**Step 1: Write integration test**

Create `tests/integration_tests.rs`:
```rust
use std::env;
use std::fs;
use tempfile::TempDir;
use musubi::config::Config;
use musubi::fetch;
use musubi::parse;
use musubi::writer;

#[test]
fn test_end_to_end_without_llm() {
    // Create temp directory
    let temp_dir = TempDir::new().unwrap();

    // Simple HTML for testing
    let test_html = r#"
        <html>
            <head>
                <title>Test Page</title>
                <meta name="description" content="A test page">
            </head>
            <body>
                <article>This is test content for the page.</article>
            </body>
        </html>
    "#;

    // Parse metadata
    let metadata = parse::extract_metadata(test_html, "https://example.com/test").unwrap();
    assert_eq!(metadata.title, "Test Page");

    // Write file without summary
    let file_path = writer::write_link_file(
        temp_dir.path(),
        &metadata.title,
        &metadata.url,
        &metadata.fetch_date,
        None,
        &[],
    ).unwrap();

    // Verify file was created
    assert!(file_path.exists());

    // Verify content
    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("title: Test Page"));
    assert!(content.contains("https://example.com/test"));
    assert!(content.contains("## Test Page"));
    assert!(content.contains("#links"));
}

#[test]
fn test_url_cleaning_integration() {
    let url = "https://example.com/page?utm_source=test&id=123";
    let cleaned = fetch::clean_url(url).unwrap();
    assert!(!cleaned.contains("utm_source"));
    assert!(cleaned.contains("id=123"));
}
```

**Step 2: Run integration tests**

Run: `cargo test integration_tests`
Expected: PASS (2 tests)

**Step 3: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add tests/integration_tests.rs
git commit -m "test: add integration tests for end-to-end flow"
```

---

## Task 10: Documentation & README

**Files:**
- Create: `README.md`

**Step 1: Write README**

Create `README.md`:
```markdown
# Musubi (結び)

A CLI tool that fetches web pages, extracts metadata, uses an LLM to generate summaries and tags, and saves everything as markdown files.

## Installation

```bash
cargo install --path .
```

## Configuration

Set environment variables:

```bash
# Required: At least one LLM API key
export ANTHROPIC_API_KEY="your-key-here"
# or
export OPENAI_API_KEY="your-key-here"

# Optional: Custom links directory (defaults to ~/links)
export MUSUBI_LINKS_DIR="$HOME/my-links"
```

## Usage

```bash
# Basic usage
musubi https://example.com/article

# Override links directory
musubi https://example.com/article --dir ./my-links
```

## Output Format

Files are saved as `YYYY-MM-DD Title.md` with frontmatter:

```markdown
---
title: Page Title
date: 2025-01-08T18:32:15.123Z
url: https://example.com/article
---

## Page Title

https://example.com/article

LLM-generated summary of the page content in 2-3 sentences.

---

[[2025-01-08]] #links #tag1 #tag2 #tag3
```

## Features

- Automatic tracking parameter removal (utm_*, fbclid, etc.)
- LLM-generated summaries using Claude or ChatGPT
- Automatic tag generation
- Graceful degradation (saves without summary if LLM fails)
- Duplicate filename handling

## Development

```bash
# Run tests
cargo test

# Build
cargo build --release

# Install locally
cargo install --path .
```

## License

MIT
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README with installation and usage instructions"
```

---

## Task 11: Final Testing & Polish

**Files:**
- Modify: `Cargo.toml` (metadata)

**Step 1: Add package metadata**

Update `Cargo.toml`:
```toml
[package]
name = "musubi"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Save and summarize web links to markdown files"
license = "MIT"
repository = "https://github.com/yourusername/musubi"
keywords = ["cli", "markdown", "llm", "bookmarks"]
categories = ["command-line-utilities"]
```

**Step 2: Run full test suite**

Run: `cargo test --all`
Expected: All tests pass

**Step 3: Test with real URL (requires API key)**

Run: `ANTHROPIC_API_KEY=your-key cargo run -- "https://doc.rust-lang.org/book/"`
Expected: Successfully creates markdown file with summary

**Step 4: Build release binary**

Run: `cargo build --release`
Expected: Binary at `target/release/musubi`

**Step 5: Final commit**

```bash
git add Cargo.toml
git commit -m "chore: add package metadata for publication"
```

---

## Verification Checklist

Before considering the implementation complete:

- [ ] All unit tests pass (`cargo test`)
- [ ] Integration tests pass
- [ ] CLI help works (`cargo run -- --help`)
- [ ] Handles invalid URLs gracefully
- [ ] Handles missing API keys gracefully (saves without summary)
- [ ] Creates directory if it doesn't exist
- [ ] Handles duplicate filenames
- [ ] Cleans tracking parameters from URLs
- [ ] Extracts title and description from HTML
- [ ] Generates LLM summary (when API key present)
- [ ] Saves properly formatted markdown with frontmatter
- [ ] README documentation is clear
- [ ] All commits have descriptive messages

## Notes

- The OpenAI provider implementation is intentionally left for future enhancement
- Manual testing with real URLs requires a valid ANTHROPIC_API_KEY
- Consider adding `examples/` directory with sample outputs if desired
- Consider adding CI/CD configuration (GitHub Actions) for automated testing
