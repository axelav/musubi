# Archive Mode Design

**Date:** 2026-01-08
**Status:** Approved for implementation

## Overview

Archive mode extends Musubi to save self-contained HTML versions of web pages alongside the existing markdown summaries. This enables offline reading even if the original page goes down.

## Goals

- Preserve pages for offline reading
- Maintain current summary/tagging features
- Keep archives self-contained (no external dependencies)
- Start simple: HTML + inline CSS only (no images/fonts initially)

## User Interface

### Command Line

```bash
# Regular mode (current behavior)
musubi https://example.com/article

# Archive mode (saves both .md and .html)
musubi --archive https://example.com/article
musubi -a https://example.com/article
```

Both `--archive` and `-a` flags trigger archive mode.

### File Output

```
~/links/
  2026-01-08 Example Article.md    # Summary, tags, metadata (existing)
  2026-01-08 Example Article.html  # Archived page content (new)
```

Files share the same base name for easy association.

## Architecture

### Execution Flow

```
1. Fetch page HTML (existing)
2. Extract metadata (existing)
3. Generate LLM summary & tags (existing)
4. Save markdown file (existing)
5. Process HTML for archival (new):
   - Parse HTML
   - Find external <link rel="stylesheet"> references
   - Fetch each CSS file
   - Inline CSS into <style> tags
   - Strip <script> tags and event handlers
   - Log failures, add HTML comments for missing CSS
6. Save companion .html file (new)
```

### Graceful Degradation

Archive mode follows the same philosophy as LLM failure handling:
- If archive processing fails → markdown summary still saves
- If CSS fetch fails → HTML saves anyway with warnings
- Best effort approach: save what we can, note what failed

## HTML Processing Details

### CSS Inlining

1. Parse HTML with `scraper` crate (already a dependency)
2. Find all `<link rel="stylesheet" href="...">` elements
3. For each external stylesheet:
   - Resolve relative URLs against page's base URL
   - Fetch CSS content using `reqwest` with timeout (10s default)
   - **Success:** Replace `<link>` with `<style>/* from: URL */\n{CSS content}</style>`
   - **Failure:**
     - Log warning to stderr: `Failed to fetch CSS: {URL}: {error}`
     - Add HTML comment: `<!-- Failed to inline stylesheet: {URL} ({error}) -->`
     - Remove the `<link>` tag

### Script Removal

- Remove all `<script>` tags (inline and external)
- Remove event handler attributes (`onclick`, `onload`, `onerror`, etc.)
- Keep page static and safe to open

### URL Resolution

- Use `url::Url::join()` for relative CSS paths
- Handle edge cases:
  - Protocol-relative URLs: `//example.com/style.css`
  - Absolute paths: `/static/style.css`
  - Already absolute URLs: `https://cdn.example.com/style.css`

## Configuration

### Defaults

```rust
pub struct ArchiveConfig {
    pub css_timeout: Duration,      // Default: 10s per stylesheet
    pub css_max_size: usize,        // Default: 5MB per file
}
```

These prevent hanging on slow/huge stylesheets and avoid bloating archives.

### Future Extension

Could make these configurable via CLI flags or config file if needed.

## Error Handling

### Fetch Failures

**CSS fetch fails (timeout, 403, network error):**
- Log to stderr
- Add HTML comment noting the failure
- Continue processing other stylesheets

**HTML processing fails entirely:**
- Log error to stderr
- Markdown summary already saved
- Don't save corrupted HTML

### Edge Cases

1. **No external CSS** - Page has only inline styles or no styles
   - Save HTML as-is after stripping scripts

2. **Duplicate filename** - HTML file already exists
   - Use same numbering strategy as markdown: `Title (2).html`
   - Keep HTML and markdown numbers synchronized

3. **Huge CSS files** - Stylesheets exceed max size
   - Treat as fetch failure
   - Log: `CSS too large: {URL} ({size} bytes, max {max})`

4. **CORS/403 errors** - CDN blocks requests
   - Treat as fetch failure
   - Log specific HTTP status

5. **Malformed CSS** - Invalid CSS content
   - Inline it anyway
   - Browser will handle/ignore bad CSS

6. **HTML parse failure** - Malformed HTML
   - Save raw HTML with warning comment at top
   - Scripts still stripped if possible

## Code Structure

### New Module: `src/archive.rs`

```rust
pub struct ArchiveConfig {
    pub css_timeout: Duration,
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
pub fn archive_page(
    html: &str,
    base_url: &Url,
    config: &ArchiveConfig,
) -> Result<String> {
    // Returns processed HTML string
}

fn inline_stylesheets(
    doc: &mut Html,
    base_url: &Url,
    config: &ArchiveConfig
) -> Vec<String> {
    // Returns list of failures for logging
}

fn strip_scripts(doc: &mut Html) { }
fn strip_event_handlers(doc: &mut Html) { }
```

### Integration

In `src/lib.rs` or main processing flow:

```rust
// After saving markdown
if args.archive {
    match archive::archive_page(&html, &url, &Default::default()) {
        Ok(archived_html) => {
            let html_path = get_html_path(&md_path); // Same base name, .html extension
            save_html_file(&archived_html, &html_path)?;
        }
        Err(e) => {
            eprintln!("Warning: Archive failed: {}", e);
            // Markdown already saved, continue
        }
    }
}
```

### CLI Changes

In `src/main.rs`:

```rust
#[derive(Parser)]
struct Args {
    // ... existing fields ...

    /// Save archived HTML version alongside markdown summary
    #[arg(short = 'a', long = "archive")]
    archive: bool,
}
```

## Testing Strategy

### Unit Tests (`tests/archive.rs`)

- CSS inlining with mock HTML/CSS
- Script removal (inline `<script>` and external)
- Event handler stripping (onclick, onload, etc.)
- URL resolution:
  - Relative paths: `styles/main.css`
  - Absolute paths: `/css/style.css`
  - Protocol-relative: `//cdn.example.com/style.css`
  - Already absolute: `https://example.com/style.css`
- Error cases:
  - Malformed HTML
  - Failed CSS fetches
  - Timeout handling
  - Size limit enforcement

### Integration Tests

- End-to-end: `musubi --archive URL` creates both `.md` and `.html`
- Verify HTML is self-contained (no network requests to render)
- Graceful degradation: archive fails, markdown still saves
- Filename synchronization for duplicates

### Manual Testing Checklist

**Test with real-world sites:**
- [ ] Wikipedia articles (clean HTML, external CSS)
- [ ] News sites (complex layouts, multiple stylesheets)
- [ ] Blog posts (various CMS systems)
- [ ] Documentation sites (often heavy CSS)

**Verification:**
- [ ] Archived HTML opens correctly in browser
- [ ] Inline styles render properly
- [ ] No console errors about missing resources
- [ ] Page is readable offline

## Storage Implications

- HTML files will be larger than markdown (especially with inlined CSS)
- Typical sizes:
  - Markdown summary: 1-5 KB
  - Archived HTML: 50-500 KB (depends on CSS size)
- Users should be aware archives consume more disk space
- Document in README and help text

## Future Extensions

This design makes it easy to add more archive features incrementally:

### Images (future)
- Add `--archive-images` flag
- Fetch referenced images
- Base64 encode and inline as data URIs
- Or save as separate files in subdirectory

### Web Fonts (future)
- Similar approach to CSS inlining
- Fetch font files, inline as data URIs in CSS

### Full Subdirectory Mode (future)
- `YYYY-MM-DD Title/` folder structure
- `summary.md`, `archive.html`, `assets/` subdirectory
- More organized for complex archives

## Success Criteria

- [ ] `musubi -a URL` creates both `.md` and `.html` files
- [ ] Archived HTML is self-contained (opens offline)
- [ ] CSS is properly inlined
- [ ] Scripts are removed
- [ ] Failures are logged with helpful messages
- [ ] Markdown summary still saves even if archive fails
- [ ] Tests cover core functionality and edge cases
- [ ] Documentation updated (README, help text)

## Notes

- Start simple: HTML + inline CSS only
- Prioritize reliability over perfection
- Easy to extend with images/fonts later
- Follow existing patterns (graceful degradation, filename handling)
