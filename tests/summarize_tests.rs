use musubi::summarize::{Summary, create_provider};

#[test]
fn test_summary_structure() {
    let summary = Summary {
        summary: "This is a test summary. It has multiple sentences.".to_string(),
        tags: vec!["test".to_string(), "rust".to_string()],
    };

    assert!(!summary.summary.is_empty());
    assert_eq!(summary.tags.len(), 2);
}

#[test]
fn test_create_provider_with_openai_key() {
    let provider = create_provider(None, Some("test-openai-key".to_string()));
    assert!(provider.is_ok(), "Should create OpenAI provider when OpenAI key is provided");
}

#[test]
fn test_create_provider_prefers_anthropic() {
    let provider = create_provider(
        Some("test-anthropic-key".to_string()),
        Some("test-openai-key".to_string())
    );
    assert!(provider.is_ok(), "Should create Anthropic provider when both keys are provided");
}
