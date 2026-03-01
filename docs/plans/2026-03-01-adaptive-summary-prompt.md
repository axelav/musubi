# Adaptive Summary Prompt

## Problem

The default summarization prompt ("Provide a 2-3 sentence summary") produces generic, fixed-length summaries regardless of content complexity. This makes summaries less useful for later retrieval via grep or tag browsing.

## Design

Replace the default prompt instruction with an adaptive one that:

1. **Scales length to complexity** — simple content gets 1-2 sentences, dense content gets more
2. **Prioritizes searchable terminology** — specific terms, names, and concepts are included
3. **Captures why it matters** — what the piece contributes or argues, not just the topic

### New default instruction

> Summarize this content proportionally to its complexity. Simple or short content gets 1-2 sentences. Dense or technical content gets a longer summary that captures the key concepts, techniques, and terminology. Always include specific terms, names, and concepts someone would search for later. Focus on what makes this particular piece notable — not just what topic it covers, but what it contributes or argues.

### Changes

- `src/summarize.rs`: Replace the `unwrap_or` default string in both `AnthropicProvider` and `OpenAIProvider` implementations
- No structural changes needed — just a string swap in two places

### What stays the same

- Style instructions (no hedging, no meta-references)
- Tag guidelines (broad categories, hyphenated, lowercase)
- The `--prompt` flag still overrides the default
