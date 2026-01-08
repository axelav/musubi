# Musubi Design Document

**Date:** 2026-01-08
**Status:** Approved

## Overview

Musubi is a CLI tool that fetches web pages, extracts metadata, uses an LLM to generate summaries and tags, and saves everything as markdown files. Named after the Japanese word for "connection/link" (結び), it follows the naming pattern of existing tools (koan, kaizen).

## Core Functionality

1. Accept a URL as input
2. Clean tracking parameters from the URL
3. Fetch the web page
4. Extract metadata (title, description, etc.)
5. Use an LLM to generate a 2-3 sentence summary and relevant tags
6. Save to a markdown file with frontmatter

## Project Structure

```
musubi/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point, argument parsing
│   ├── lib.rs               # Library root, re-exports
│   ├── config.rs            # Configuration (env vars, defaults)
│   ├── fetch.rs             # HTTP fetching, URL cleaning
│   ├── parse.rs             # HTML parsing, metadata extraction
│   ├── summarize.rs         # LLM integration (Anthropic/OpenAI)
│   ├── writer.rs            # Markdown file generation
│   └── error.rs             # Error types
└── tests/
    └── integration_tests.rs
```

## Module Responsibilities

### config.rs
- Read environment variables: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, `MUSUBI_LINKS_DIR`
- Provide defaults (`~/links` for directory)
- Validate configuration

### fetch.rs
- Clean tracking parameters from URLs
- Fetch HTML content using `reqwest`
- Tracking parameters to strip:
  - Google Analytics: `utm_*`
  - Facebook: `fbclid`
  - Google Ads: `gclid`, `gclsrc`
  - Other common trackers: `mc_cid`, `mc_eid`, `_hsenc`, `_hsmi`, `ref`, `source`

### parse.rs
- Extract title from HTML
- Extract description (meta description, Open Graph)
- Extract other metadata
- Focus on main content, strip HTML tags

### summarize.rs
- Trait-based LLM provider interface
- Anthropic implementation (primary)
- OpenAI implementation (future)
- Provider selection: try Anthropic key first, then OpenAI
- Graceful degradation: if LLM fails, save file without summary

### writer.rs
- Generate markdown with frontmatter
- Handle file naming and conflicts
- Create directory if it doesn't exist

## Data Flow

```
URL (String)
  → fetch::clean_and_fetch()
  → FetchedPage { url: String, html: String }
  → parse::extract_metadata()
  → PageMetadata { title, description, ... }
  → summarize::generate()
  → Summary { text: String, tags: Vec<String> }
  → writer::write_link()
  → Result<PathBuf>
```

## Key Data Structures

```rust
pub struct FetchedPage {
    pub original_url: String,
    pub cleaned_url: String,
    pub html: String,
}

pub struct PageMetadata {
    pub title: String,
    pub description: Option<String>,
    pub fetch_date: DateTime<Utc>,
}

pub struct Summary {
    pub text: String,      // 2-3 sentences
    pub tags: Vec<String>, // e.g., ["music", "software"]
}
```

## LLM Integration

### Provider Abstraction

```rust
pub trait LlmProvider {
    fn generate_summary(&self, title: &str, content: &str) -> Result<Summary>;
}
```

### Implementation
- Use `reqwest` directly to call Anthropic Messages API
- Model: `claude-3-5-sonnet-20241022`
- Truncate content to ~4000 tokens to control costs
- Request JSON response with `summary` and `tags` fields

### Prompt Strategy
```
Given this webpage:
Title: {title}
Content: {truncated_html_text}

Generate:
1. A 2-3 sentence summary of the main content
2. 3-5 relevant topic tags (single words, lowercase)

Format: JSON with "summary" and "tags" fields
```

### Error Handling
If LLM fails (API error, missing key, etc.):
- Save markdown file with all metadata
- Omit summary section
- Print warning: "Warning: Could not generate summary, saved without it"

## File Format

```markdown
---
title: Understanding Rust Ownership
date: 2025-01-08T18:32:15.123Z
url: https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
---

## Understanding Rust Ownership

<cleaned_url>

<2-3 sentence summary from LLM>

---

[[2025-01-08]] #links #rust #programming #memory
```

## File Naming

- Format: `YYYY-MM-DD <sanitized-title>.md`
- Example: `2025-01-08 Understanding Rust Ownership.md`
- Sanitization:
  - Replace special characters (`/`, `\`, `:`, etc.) with spaces or hyphens
  - Collapse multiple spaces to single space
  - Trim leading/trailing whitespace
  - Limit to ~100 characters
- Conflict handling: append `-2`, `-3`, etc. if file exists
- Fallback: if title extraction fails, use domain name

## CLI Interface

```bash
# Basic usage
musubi https://example.com/article

# Output
✓ Fetched: Understanding Rust Ownership
✓ Generated summary
✓ Saved: ~/links/2025-01-08 Understanding Rust Ownership.md

# If LLM fails
✓ Fetched: Understanding Rust Ownership
⚠ Could not generate summary (API key missing or error)
✓ Saved: ~/links/2025-01-08 Understanding Rust Ownership.md
```

### Arguments

```rust
#[derive(Parser)]
#[command(name = "musubi")]
#[command(about = "Save and summarize web links to markdown")]
struct Cli {
    /// URL to save
    url: String,

    /// Override links directory (default: $MUSUBI_LINKS_DIR or ~/links)
    #[arg(short, long)]
    dir: Option<PathBuf>,
}
```

## Dependencies

- `clap` (derive) - CLI argument parsing
- `reqwest` (blocking) - HTTP client & LLM API calls
- `scraper` - HTML parsing with CSS selectors
- `url` - URL parsing and manipulation
- `chrono` - Date/time handling
- `anyhow` - Error handling
- `serde`, `serde_json` - JSON serialization for LLM API

## Testing Strategy

- Unit tests for URL cleaning (tracking params removed correctly)
- Unit tests for title sanitization (special characters handled)
- Unit tests for markdown formatting
- Mock HTTP responses for parser tests
- Integration test with mock LLM (no real API calls)
- Manual end-to-end testing with real URLs

## Error Handling

Using `anyhow` for error propagation with context:

- Network errors → "Failed to fetch URL: {url}"
- Parse errors → "Failed to extract page metadata"
- LLM errors → Warning printed, file saved without summary
- File write errors → "Failed to write markdown file"

## Configuration

### Environment Variables

- `ANTHROPIC_API_KEY` - Anthropic API key (checked first)
- `OPENAI_API_KEY` - OpenAI API key (future, checked second)
- `MUSUBI_LINKS_DIR` - Directory for saved links (default: `~/links`)

### Provider Selection

1. Check for `ANTHROPIC_API_KEY`
2. If not found, check for `OPENAI_API_KEY`
3. If neither found, return error with helpful message

## Future Enhancements

- OpenAI provider implementation
- Support for additional metadata sources
- Custom tag overrides via CLI flags
- Different output formats
- Search/query saved links
