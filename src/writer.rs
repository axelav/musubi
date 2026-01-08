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
