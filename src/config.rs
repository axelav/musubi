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

        // First, try to read explicit directories from the environment.
        let links_dir_env = env::var("MUSUBI_LINKS_DIR").ok().map(PathBuf::from);
        let now_dir_env = env::var("MUSUBI_NOW_DIR").ok().map(PathBuf::from);

        // Only require HOME when we need to fall back to default locations.
        let (links_dir, now_dir) = match (links_dir_env, now_dir_env) {
            (Some(links_dir), Some(now_dir)) => (links_dir, now_dir),
            (links_dir_opt, now_dir_opt) => {
                let home = env::var("HOME").context("HOME environment variable not set")?;
                let home_path = PathBuf::from(home);

                let links_dir = links_dir_opt.unwrap_or_else(|| home_path.join("links"));
                let now_dir = now_dir_opt.unwrap_or_else(|| home_path.join("now"));

                (links_dir, now_dir)
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
