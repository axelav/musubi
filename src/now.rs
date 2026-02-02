use anyhow::{Context, Result};
use chrono::{Local, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::writer::{find_available_filename, sanitize_filename};

/// Escape a string for use in YAML
/// Wraps the string in double quotes if it contains special characters
fn yaml_escape(s: &str) -> String {
    // Check if the string needs escaping
    let needs_escape = s.is_empty()
        || s.starts_with(' ')
        || s.ends_with(' ')
        || s.contains(':')
        || s.contains('#')
        || s.contains('\n')
        || s.contains('\r')
        || s.contains('"')
        || s.contains('\'')
        || s.contains('\\')
        || s.starts_with('-')
        || s.starts_with('[')
        || s.starts_with('{')
        || s.starts_with('&')
        || s.starts_with('*')
        || s.starts_with('!')
        || s.starts_with('|')
        || s.starts_with('>')
        || s.starts_with('%')
        || s.starts_with('@')
        || s.starts_with('`');

    if !needs_escape {
        return s.to_string();
    }

    // Escape the string by wrapping in double quotes and escaping internal quotes
    // Note: backslashes must be escaped first to avoid double-escaping
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

/// Create a now file and optionally open in editor
/// Returns the created file path and whether editor was launched
pub fn create_now_file(
    dir: &Path,
    title: Option<&str>,
    open_editor: bool,
) -> Result<(PathBuf, bool)> {
    // Create directory if needed
    fs::create_dir_all(dir).context(format!("Failed to create directory: {}", dir.display()))?;

    // Generate title (use time if none provided)
    let now = Utc::now();
    let local_now = now.with_timezone(&Local);

    let display_title = match title {
        Some(t) if !t.trim().is_empty() => t.trim().to_string(),
        _ => local_now.format("%H-%M-%S").to_string(),
    };

    // Generate filename
    let date_str = local_now.format("%Y-%m-%d").to_string();
    let sanitized_title = sanitize_filename(&display_title);
    let base_filename = format!("{} {}.md", date_str, sanitized_title);
    let file_path = find_available_filename(dir, &base_filename);

    // Generate content
    let iso_date = now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    let escaped_title = yaml_escape(&display_title);
    let content = format!(
        "---\ntitle: {}\ndate: {}\n---\n\n## {}\n\n",
        escaped_title, iso_date, display_title
    );

    // Write file
    fs::write(&file_path, &content)
        .context(format!("Failed to write file: {}", file_path.display()))?;

    // Handle editor
    if !open_editor {
        return Ok((file_path, false));
    }

    let editor_launched = match std::env::var("EDITOR") {
        Ok(editor) => {
            // Parse EDITOR to handle arguments safely (e.g., "code --wait")
            let parts = shlex::split(&editor).ok_or_else(|| {
                anyhow::anyhow!("Failed to parse EDITOR environment variable: {}", editor)
            })?;

            if parts.is_empty() {
                anyhow::bail!("EDITOR environment variable is empty after parsing");
            }

            let program = &parts[0];
            let args: Vec<&str> = parts[1..].iter().map(|s| s.as_str()).collect();

            // Build command without shell to avoid injection
            let mut cmd = Command::new(program);
            cmd.args(args);
            cmd.arg(&file_path);

            let status = cmd
                .status()
                .context(format!("Failed to launch editor: {}", editor))?;

            if !status.success() {
                if let Some(code) = status.code() {
                    anyhow::bail!("Editor exited with non-zero status code: {}", code);
                } else {
                    anyhow::bail!("Editor was terminated by signal");
                }
            }
            true
        }
        Err(_) => false,
    };

    Ok((file_path, editor_launched))
}
