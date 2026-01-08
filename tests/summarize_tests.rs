use musubi::summarize::Summary;

#[test]
fn test_summary_structure() {
    let summary = Summary {
        text: "This is a test summary. It has multiple sentences.".to_string(),
        tags: vec!["test".to_string(), "rust".to_string()],
    };

    assert!(!summary.text.is_empty());
    assert_eq!(summary.tags.len(), 2);
}
