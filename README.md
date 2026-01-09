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

- Automatic tracking parameter removal (utm\_\*, fbclid, etc.)
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
