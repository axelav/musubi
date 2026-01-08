use anyhow::{Context, Result};
use url::Url;

const TRACKING_PARAMS: &[&str] = &[
    "utm_source", "utm_medium", "utm_campaign", "utm_term", "utm_content",
    "fbclid", "gclid", "gclsrc",
    "mc_cid", "mc_eid",
    "_hsenc", "_hsmi",
    "ref", "source",
];

pub fn clean_url(url_str: &str) -> Result<String> {
    let mut url = Url::parse(url_str)
        .context("Failed to parse URL")?;

    let filtered_pairs: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(key, _)| !TRACKING_PARAMS.contains(&key.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    if filtered_pairs.is_empty() {
        url.set_query(None);
    } else {
        let query_string = filtered_pairs
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        url.set_query(Some(&query_string));
    }

    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_url_no_params() {
        let input = "https://example.com/page";
        let cleaned = clean_url(input).unwrap();
        assert_eq!(cleaned, "https://example.com/page");
    }
}
