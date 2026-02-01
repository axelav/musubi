use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub anthropic_key: Option<String>,
    pub openai_key: Option<String>,
    pub links_dir: PathBuf,
    pub now_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let anthropic_key = env::var("ANTHROPIC_API_KEY").ok();
        let openai_key = env::var("OPENAI_API_KEY").ok();

        let home = env::var("HOME").context("HOME environment variable not set")?;
        let home_path = PathBuf::from(&home);

        let links_dir = env::var("MUSUBI_LINKS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_path.join("links"));

        let now_dir = env::var("MUSUBI_NOW_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home_path.join("now"));

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
