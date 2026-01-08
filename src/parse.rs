use anyhow::Result;
use chrono::{DateTime, Utc};
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct PageMetadata {
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub fetch_date: DateTime<Utc>,
}

pub fn collapse_consecutive(s: &str) -> String {
    let mut result = String::new();
    let mut prev = '\0';

    for c in s.chars() {
        if c == ' ' && prev == ' ' {
            continue; // skip consecutive spaces
        }
        if c == '-' && prev == '-' {
            continue; // skip consecutive hyphens
        }
        result.push(c);
        prev = c;
    }
    result
}

pub fn extract_metadata(html: &str, url: &str) -> Result<PageMetadata> {
    let document = Html::parse_document(html);

    // Extract title
    let title_selector = Selector::parse("title").unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_else(|| "Untitled".to_string())
        .trim()
        .to_string();

    // Extract description (try meta description, then og:description)
    let meta_desc_selector = Selector::parse("meta[name='description']").unwrap();
    let og_desc_selector = Selector::parse("meta[property='og:description']").unwrap();

    let description = document
        .select(&meta_desc_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .or_else(|| {
            document
                .select(&og_desc_selector)
                .next()
                .and_then(|el| el.value().attr("content"))
        })
        .map(|s| s.to_string());

    Ok(PageMetadata {
        title,
        description,
        url: url.to_string(),
        fetch_date: Utc::now(),
    })
}
