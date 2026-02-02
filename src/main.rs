use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use musubi::config::Config;
use musubi::fetch;
use musubi::now;
use musubi::parse;
use musubi::summarize;
use musubi::writer;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "musubi")]
#[command(about = "Save and summarize web links to markdown", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Save a web link as markdown
    Link {
        /// URL to save
        url: String,

        /// Override links directory
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Save archived HTML version
        #[arg(short = 'a', long = "archive")]
        archive: bool,

        /// Custom prompt for summary generation
        #[arg(short, long)]
        prompt: Option<String>,
    },
    /// Create a timestamped note file
    Now {
        /// Note title (defaults to current time if omitted)
        title: Option<String>,

        /// Override output directory
        #[arg(short, long)]
        dir: Option<PathBuf>,

        /// Create file without opening editor
        #[arg(long)]
        no_edit: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::from_env().context("Failed to load configuration")?;

    match cli.command {
        Commands::Link { url, dir, archive, prompt } => run_link(config, url, dir, archive, prompt),
        Commands::Now { title, dir, no_edit } => run_now(config, title, dir, no_edit),
    }
}

fn run_now(
    config: Config,
    title: Option<String>,
    dir: Option<PathBuf>,
    no_edit: bool,
) -> Result<()> {
    let output_dir = dir.unwrap_or(config.now_dir);

    let (file_path, editor_launched) =
        now::create_now_file(&output_dir, title.as_deref(), !no_edit)?;

    if no_edit {
        eprintln!("✓ Created: {}", file_path.display());
    } else if !editor_launched {
        eprintln!("No editor set. File created at: {}", file_path.display());
    } else {
        eprintln!("✓ Created: {}", file_path.display());
    }

    Ok(())
}

fn run_link(
    mut config: Config,
    url: String,
    dir: Option<PathBuf>,
    archive: bool,
    prompt: Option<String>,
) -> Result<()> {
    if let Some(d) = dir {
        config.links_dir = d;
    }

    // Fetch page
    println!("⏳ Fetching: {}", url);
    let page = fetch::fetch_page(&url).context("Failed to fetch page")?;

    // Parse metadata
    let metadata = parse::extract_metadata(&page.html, &page.cleaned_url)
        .context("Failed to extract metadata")?;

    println!("✓ Fetched: {}", metadata.title);

    // Generate summary (optional, graceful degradation)
    let (summary_text, tags) = if config.has_llm_key() {
        match summarize::create_provider(config.anthropic_key, config.openai_key) {
            Ok(provider) => {
                let text_content = extract_text_content(&page.html);
                match provider.generate_summary(&metadata.title, &text_content, prompt.as_deref()) {
                    Ok(summary) => {
                        println!("✓ Generated summary");
                        (Some(summary.summary), summary.tags)
                    }
                    Err(e) => {
                        eprintln!("⚠ Could not generate summary: {}", e);
                        (None, vec![])
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠ Could not create LLM provider: {}", e);
                (None, vec![])
            }
        }
    } else {
        eprintln!("⚠ No LLM API key found, saving without summary");
        (None, vec![])
    };

    // Write markdown file
    let file_path = writer::write_link_file(
        &config.links_dir,
        &metadata.title,
        &page.cleaned_url,
        &metadata.fetch_date,
        summary_text.as_deref(),
        &tags,
    )
    .context("Failed to write link file")?;

    println!("✓ Saved: {}", file_path.display());

    // Archive HTML if requested
    if archive {
        match musubi::archive::archive_page(
            &page.html,
            &url::Url::parse(&page.cleaned_url)?,
            &musubi::archive::ArchiveConfig::default(),
        ) {
            Ok(archived_html) => {
                let html_path = writer::get_html_path(&file_path);
                match fs::write(&html_path, archived_html) {
                    Ok(_) => println!("✓ Archived: {}", html_path.display()),
                    Err(e) => eprintln!("⚠ Failed to write archive: {}", e),
                }
            }
            Err(e) => {
                eprintln!("⚠ Failed to archive page: {}", e);
            }
        }
    }

    Ok(())
}

fn extract_text_content(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    let content_selectors = vec![
        "article",
        "main",
        "[role='main']",
        ".content",
        "#content",
        "body",
    ];

    for selector_str in content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if !text.trim().is_empty() {
                    return text;
                }
            }
        }
    }

    document.root_element().text().collect::<Vec<_>>().join(" ")
}
