use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Config {
    pub anthropic_key: Option<String>,
    pub openai_key: Option<String>,
    pub links_dir: PathBuf,
    pub now_dir: PathBuf,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileConfig {
    anthropic_api_key: Option<String>,
    openai_api_key: Option<String>,
    links_dir: Option<String>,
    now_dir: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let file = load_file_config()?;

        let anthropic_key = env::var("ANTHROPIC_API_KEY")
            .ok()
            .or(file.anthropic_api_key);
        let openai_key = env::var("OPENAI_API_KEY").ok().or(file.openai_api_key);

        let links_env = env::var("MUSUBI_LINKS_DIR").ok().map(PathBuf::from);
        let now_env = env::var("MUSUBI_NOW_DIR").ok().map(PathBuf::from);

        let links_file = file.links_dir.as_deref().map(expand_tilde);
        let now_file = file.now_dir.as_deref().map(expand_tilde);

        let links_resolved = links_env.or(links_file);
        let now_resolved = now_env.or(now_file);

        let (links_dir, now_dir) = match (links_resolved, now_resolved) {
            (Some(l), Some(n)) => (l, n),
            (links_opt, now_opt) => {
                let home = env::var("HOME").context("HOME environment variable not set")?;
                let home_path = PathBuf::from(home);
                let links = links_opt.unwrap_or_else(|| home_path.join("links"));
                let now = now_opt.unwrap_or_else(|| home_path.join("now"));
                (links, now)
            }
        };

        Ok(Config {
            anthropic_key,
            openai_key,
            links_dir,
            now_dir,
        })
    }

    pub fn has_llm_key(&self) -> bool {
        self.anthropic_key.is_some() || self.openai_key.is_some()
    }
}

fn load_file_config() -> Result<FileConfig> {
    let path = match config_file_path() {
        Some(p) => p,
        None => return Ok(FileConfig::default()),
    };

    let contents = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(FileConfig::default()),
        Err(e) => {
            return Err(anyhow::Error::new(e)
                .context(format!("failed to read config file at {}", path.display())))
        }
    };

    let parsed: FileConfig = toml::from_str(&contents)
        .with_context(|| format!("failed to parse config file at {}", path.display()))?;

    warn_if_world_readable_with_secret(&path, &parsed);

    Ok(parsed)
}

fn config_file_path() -> Option<PathBuf> {
    if let Ok(override_path) = env::var("MUSUBI_CONFIG") {
        if !override_path.is_empty() {
            return Some(PathBuf::from(override_path));
        }
    }
    ProjectDirs::from("", "", "musubi").map(|dirs| dirs.config_dir().join("config.toml"))
}

fn expand_tilde(input: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(input).into_owned())
}

#[cfg(unix)]
fn warn_if_world_readable_with_secret(path: &Path, file: &FileConfig) {
    use std::os::unix::fs::PermissionsExt;

    if file.anthropic_api_key.is_none() && file.openai_api_key.is_none() {
        return;
    }
    let Ok(meta) = fs::metadata(path) else {
        return;
    };
    let mode = meta.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        eprintln!(
            "warning: {} is readable by group or others (mode {:o}); it contains an API key. Consider: chmod 600 {}",
            path.display(),
            mode,
            path.display()
        );
    }
}

#[cfg(not(unix))]
fn warn_if_world_readable_with_secret(_path: &Path, _file: &FileConfig) {}
