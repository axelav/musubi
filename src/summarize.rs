use anyhow::{anyhow, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Summary {
    /// A direct, confident summary of what this content is about. State facts plainly without hedging.
    pub summary: String,
    /// An array of 3-5 broad topic tags (lowercase, hyphenated if multi-word). Prefer general categories over narrow/specific terms.
    pub tags: Vec<String>,
}

pub trait LlmProvider {
    fn generate_summary(
        &self,
        title: &str,
        content: &str,
        custom_prompt: Option<&str>,
    ) -> Result<Summary>;
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
    tools: Vec<Tool>,
    tool_choice: ToolChoice,
}

#[derive(Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Serialize)]
struct ToolChoice {
    #[serde(rename = "type")]
    type_: String,
    name: String,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

impl LlmProvider for AnthropicProvider {
    fn generate_summary(
        &self,
        title: &str,
        content: &str,
        custom_prompt: Option<&str>,
    ) -> Result<Summary> {
        let truncated_content = truncate_content(content, 4000);

        let instruction = custom_prompt.unwrap_or("Provide a 2-3 sentence summary");
        let prompt = format!(
            "Given this webpage, generate a summary and relevant tags.\n\nInstructions for summary: {}\n\nStyle: Write with confidence and directness. State what the content IS, not what it \"appears to be\" or \"seems to discuss\". Never start with \"This webpage\", \"This article\", \"This page\", \"This piece\", \"This appears\", \"A publication titled\", or any meta-reference to the format. Never use hedging language like \"appears to\", \"seems to\", \"likely\", or \"possibly\". Just describe the subject matter itself as if you were telling someone about it in conversation.\n\nTag guidelines: Generate 3-5 broad, general-purpose tags. Prefer wide categories (e.g., 'fiction', 'design', 'programming') over narrow or niche terms (e.g., avoid 'interactive-fiction', 'ios-app', 'mobile-reading'). Tags should be lowercase, using hyphens for multi-word tags (e.g., 'open-source' not 'opensource').\n\nTitle: {}\n\nContent: {}",
            instruction, title, truncated_content
        );

        // Generate JSON schema from the Summary struct
        let schema = schemars::schema_for!(Summary);
        let schema_json = serde_json::to_value(schema).context("Failed to serialize schema")?;

        let client = reqwest::blocking::Client::new();
        let request_body = AnthropicRequest {
            model: "claude-sonnet-4-5-20250929".to_string(),
            max_tokens: 1024,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            tools: vec![Tool {
                name: "generate_summary".to_string(),
                description: "Generate a summary and tags for a webpage".to_string(),
                input_schema: schema_json,
            }],
            tool_choice: ToolChoice {
                type_: "tool".to_string(),
                name: "generate_summary".to_string(),
            },
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
            return Err(anyhow!("Anthropic API error ({}): {}", status, error_text));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .context("Failed to parse Anthropic response")?;

        // Extract tool use from response
        for block in anthropic_response.content {
            if let ContentBlock::ToolUse { input, .. } = block {
                let summary: Summary = serde_json::from_value(input)
                    .context("Failed to parse summary from tool use")?;
                return Ok(summary);
            }
        }

        Err(anyhow!("No tool use found in Anthropic response"))
    }
}

fn truncate_content(content: &str, max_bytes: usize) -> String {
    if content.len() <= max_bytes {
        content.to_string()
    } else {
        // Find the largest valid char boundary at or before max_bytes
        let mut end = max_bytes;
        while !content.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &content[..end])
    }
}

pub struct OpenAIProvider {
    api_key: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    functions: Vec<OpenAIFunction>,
    function_call: OpenAIFunctionCall,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Serialize)]
struct OpenAIFunctionCall {
    name: String,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize, Debug)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize, Debug)]
struct OpenAIResponseMessage {
    function_call: Option<OpenAIFunctionCallResponse>,
}

#[derive(Deserialize, Debug)]
struct OpenAIFunctionCallResponse {
    arguments: String,
}

impl LlmProvider for OpenAIProvider {
    fn generate_summary(
        &self,
        title: &str,
        content: &str,
        custom_prompt: Option<&str>,
    ) -> Result<Summary> {
        let truncated_content = truncate_content(content, 4000);

        let instruction = custom_prompt.unwrap_or("Provide a 2-3 sentence summary");
        let prompt = format!(
            "Given this webpage, generate a summary and relevant tags.\n\nInstructions for summary: {}\n\nStyle: Write with confidence and directness. State what the content IS, not what it \"appears to be\" or \"seems to discuss\". Never start with \"This webpage\", \"This article\", \"This page\", \"This piece\", \"This appears\", \"A publication titled\", or any meta-reference to the format. Never use hedging language like \"appears to\", \"seems to\", \"likely\", or \"possibly\". Just describe the subject matter itself as if you were telling someone about it in conversation.\n\nTag guidelines: Generate 3-5 broad, general-purpose tags. Prefer wide categories (e.g., 'fiction', 'design', 'programming') over narrow or niche terms (e.g., avoid 'interactive-fiction', 'ios-app', 'mobile-reading'). Tags should be lowercase, using hyphens for multi-word tags (e.g., 'open-source' not 'opensource').\n\nTitle: {}\n\nContent: {}",
            instruction, title, truncated_content
        );

        let schema = schemars::schema_for!(Summary);
        let schema_json = serde_json::to_value(schema).context("Failed to serialize schema")?;

        let client = reqwest::blocking::Client::new();
        let request_body = OpenAIRequest {
            model: "gpt-4o-mini".to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: prompt,
            }],
            functions: vec![OpenAIFunction {
                name: "generate_summary".to_string(),
                description: "Generate a summary and tags for a webpage".to_string(),
                parameters: schema_json,
            }],
            function_call: OpenAIFunctionCall {
                name: "generate_summary".to_string(),
            },
        };

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", &self.api_key))
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().unwrap_or_default();
            return Err(anyhow!("OpenAI API error ({}): {}", status, error_text));
        }

        let openai_response: OpenAIResponse =
            response.json().context("Failed to parse OpenAI response")?;

        if let Some(choice) = openai_response.choices.first() {
            if let Some(function_call) = &choice.message.function_call {
                let summary: Summary = serde_json::from_str(&function_call.arguments)
                    .context("Failed to parse summary from function call")?;
                return Ok(summary);
            }
        }

        Err(anyhow!("No function call found in OpenAI response"))
    }
}

pub fn create_provider(
    anthropic_key: Option<String>,
    openai_key: Option<String>,
) -> Result<Box<dyn LlmProvider>> {
    if let Some(key) = anthropic_key {
        Ok(Box::new(AnthropicProvider::new(key)))
    } else if let Some(key) = openai_key {
        Ok(Box::new(OpenAIProvider::new(key)))
    } else {
        Err(anyhow!(
            "No LLM API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY environment variable."
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_content_with_multibyte_chars() {
        // Create a string where byte 4000 falls inside a multi-byte character
        // Use "a" (1 byte) repeated 3998 times, then "'" (3 bytes: e2 80 99)
        let mut content = "a".repeat(3998);
        content.push('\u{2019}'); // Right single quotation mark - 3-byte UTF-8 character
        content.push_str("more content");

        // This should NOT panic - it should truncate at a valid char boundary
        let result = truncate_content(&content, 4000);

        // Should truncate before the multi-byte char
        assert_eq!(result.len(), 3998 + 3); // 3998 'a's + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_content_short_string() {
        let content = "short";
        let result = truncate_content(content, 100);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_content_exact_boundary() {
        let content = "a".repeat(4000);
        let result = truncate_content(&content, 4000);
        assert_eq!(result, content);
    }
}
