use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct LinkFrontMatter {
    title: String,
    date: String,
    url: String,
}

pub fn sanitize_filename(title: &str) -> String {
    let sanitized = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => c,
        })
        .collect::<String>();

    let collapsed = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");

    let trimmed = collapsed.trim();

    if trimmed.len() > 100 {
        trimmed[..100].to_string()
    } else {
        trimmed.to_string()
    }
}

fn generate_filename(date: &DateTime<Utc>, title: &str) -> String {
    let local_date = date.with_timezone(&Local);
    let date_str = local_date.format("%Y-%m-%d").to_string();
    let sanitized_title = sanitize_filename(title);
    format!("{} {}.md", date_str, sanitized_title)
}

pub(crate) fn find_available_filename(dir: &Path, base_filename: &str) -> PathBuf {
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
    fs::create_dir_all(dir).context(format!("Failed to create directory: {}", dir.display()))?;

    // Generate filename
    let base_filename = generate_filename(date, title);
    let file_path = find_available_filename(dir, &base_filename);

    // Format date for content
    let iso_date = date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let local_date = date.with_timezone(&Local);
    let wiki_date = local_date.format("%Y-%m-%d").to_string();

    // Build YAML front matter using serde_yml for proper escaping
    let front_matter = LinkFrontMatter {
        title: title.to_string(),
        date: iso_date.clone(),
        url: url.to_string(),
    };
    
    let yaml_str = serde_yml::to_string(&front_matter)
        .context("Failed to serialize front matter to YAML")?;

    // Build content
    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&yaml_str);
    content.push_str("---\n\n");
    content.push_str(&format!("## {}\n\n", title));
    content.push_str(&format!("{}\n\n", url));

    if let Some(summary_text) = summary {
        let blockquote = summary_text
            .lines()
            .map(|line| format!("> {}", line))
            .collect::<Vec<_>>()
            .join("\n");
        content.push_str(&format!("{}\n\n", blockquote));
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

/// Generate HTML filename from markdown path
/// Example: "2026-01-08 Title.md" -> "2026-01-08 Title.html"
pub fn get_html_path(md_path: &Path) -> PathBuf {
    md_path.with_extension("html")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename() {
        let date = Utc::now();
        let filename = generate_filename(&date, "Test Title");
        let local_date = date.with_timezone(&Local);
        let date_str = local_date.format("%Y-%m-%d").to_string();
        assert_eq!(filename, format!("{} Test Title.md", date_str));
    }

    #[test]
    fn test_get_html_path() {
        let md_path = PathBuf::from("/links/2026-01-08 Title.md");
        let html_path = get_html_path(&md_path);
        assert_eq!(html_path, PathBuf::from("/links/2026-01-08 Title.html"));
    }

    #[test]
    fn test_summary_formatted_as_blockquote() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let date = Utc::now();
        let summary = "This is a test summary";

        let file_path = write_link_file(
            temp_dir.path(),
            "Test Title",
            "https://example.com",
            &date,
            Some(summary),
            &[],
        )
        .unwrap();

        let content = fs::read_to_string(&file_path).unwrap();

        // Summary should be formatted as blockquote
        assert!(content.contains("> This is a test summary"));
    }
}
