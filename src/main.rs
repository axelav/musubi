use anyhow::{Context, Result};
use clap::Parser;
use musubi::archive;
use musubi::config::Config;
use musubi::fetch;
use musubi::parse;
use musubi::summarize;
use musubi::writer;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "musubi")]
#[command(about = "Save and summarize web links to markdown", long_about = None)]
struct Cli {
    /// URL to save
    url: String,

    /// Override links directory (default: $MUSUBI_LINKS_DIR or ~/links)
    #[arg(short, long)]
    dir: Option<PathBuf>,

    /// Save archived HTML version alongside markdown summary
    #[arg(short = 'a', long = "archive")]
    archive: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let mut config = Config::from_env().context("Failed to load configuration")?;

    // Override directory if provided
    if let Some(dir) = cli.dir {
        config.links_dir = dir;
    }

    // Fetch page
    println!("⏳ Fetching: {}", cli.url);
    let page = fetch::fetch_page(&cli.url).context("Failed to fetch page")?;

    // Parse metadata
    let metadata = parse::extract_metadata(&page.html, &page.cleaned_url)
        .context("Failed to extract metadata")?;

    println!("✓ Fetched: {}", metadata.title);

    // Generate summary (optional, graceful degradation)
    let (summary_text, tags) = if config.has_llm_key() {
        match summarize::create_provider(config.anthropic_key, config.openai_key) {
            Ok(provider) => {
                // Extract text content from HTML for summarization
                let text_content = extract_text_content(&page.html);

                match provider.generate_summary(&metadata.title, &text_content) {
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
    if cli.archive {
        match archive::archive_page(
            &page.html,
            &url::Url::parse(&page.cleaned_url)?,
            &archive::ArchiveConfig::default(),
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
                // Markdown already saved, continue
            }
        }
    }

    Ok(())
}

fn extract_text_content(html: &str) -> String {
    use scraper::{Html, Selector};

    let document = Html::parse_document(html);

    // Try to get main content (common tags)
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

    // Fallback: get all text
    document.root_element().text().collect::<Vec<_>>().join(" ")
}
