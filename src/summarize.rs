use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub text: String,
    pub tags: Vec<String>,
}

pub trait LlmProvider {
    fn generate_summary(&self, title: &str, content: &str) -> Result<Summary>;
}

pub struct AnthropicProvider {
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: String,
}

impl LlmProvider for AnthropicProvider {
    fn generate_summary(&self, title: &str, content: &str) -> Result<Summary> {
        let truncated_content = truncate_content(content, 4000);

        let prompt = format!(
            r#"Given this webpage:
Title: {}
Content: {}

Generate a JSON response with:
1. A "summary" field containing 2-3 sentences summarizing the main content
2. A "tags" field containing an array of 3-5 relevant topic tags (single words, lowercase)

Respond ONLY with valid JSON in this format:
{{"summary": "...", "tags": ["tag1", "tag2", "tag3"]}}
"#,
            title, truncated_content
        );

        let client = reqwest::blocking::Client::new();
        let request_body = AnthropicRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!(
                "Anthropic API error ({}): {}",
                status,
                error_text
            ));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .context("Failed to parse Anthropic response")?;

        let text = anthropic_response
            .content
            .first()
            .map(|c| c.text.clone())
            .ok_or_else(|| anyhow!("No content in Anthropic response"))?;

        // Parse the JSON from the LLM response
        let summary: Summary = serde_json::from_str(&text)
            .context("Failed to parse summary JSON from LLM")?;

        Ok(summary)
    }
}

fn truncate_content(content: &str, max_chars: usize) -> String {
    if content.len() <= max_chars {
        content.to_string()
    } else {
        format!("{}...", &content[..max_chars])
    }
}

pub fn create_provider(anthropic_key: Option<String>, _openai_key: Option<String>) -> Result<Box<dyn LlmProvider>> {
    if let Some(key) = anthropic_key {
        Ok(Box::new(AnthropicProvider::new(key)))
    } else {
        Err(anyhow!(
            "No LLM API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable."
        ))
    }
}
