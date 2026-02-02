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

# Optional: Custom directories (defaults to ~/links and ~/now)
export MUSUBI_LINKS_DIR="$HOME/my-links"
export MUSUBI_NOW_DIR="$HOME/my-notes"
```

## Usage

### Save a link

```bash
# Basic usage
musubi link https://example.com/article

# Save with archive mode (HTML + markdown)
musubi link --archive https://example.com/article
musubi link -a https://example.com/article

# Override links directory
musubi link <https://example.com/article> --dir ./my-links

# Use a custom prompt for the LLM summary
musubi link <https://example.com/article> --prompt "Summarize this for a developer"

# Combine flags
musubi link -a <https://example.com/article> --dir ./my-links
```

### Create a timestamped note

```bash
# Create a new note (opens in $EDITOR)
musubi now

# Create a note with a title
musubi now "My note title"

# Create without opening editor
musubi now --no-edit
musubi now "Quick thought" --no-edit

# Override notes directory
musubi now --dir ./my-notes
```

## Output Format

### Link files

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

### Now files

Timestamped notes are saved as `YYYY-MM-DD HHMMSS Title.md`:

```markdown
---
title: My note title
date: 2025-01-08T18:32:15-05:00
---
```

## Features

- Automatic tracking parameter removal (utm\_\*, fbclid, etc.)
- LLM-generated summaries using Claude or ChatGPT
- Automatic tag generation
- Custom prompts for tailored summaries
- Archive mode for offline reading (saves self-contained HTML)
- Timestamped notes with `now` subcommand
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
