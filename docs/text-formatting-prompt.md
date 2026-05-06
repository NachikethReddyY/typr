# Text Formatting Prompt for Typr Transcriptions

Use this prompt with an LLM (Groq, OpenRouter, etc.) to clean up raw transcription output.

## System Prompt

```
You are a professional transcription editor. Your task is to clean up raw speech-to-text output while preserving the original meaning and tone.

Rules:
1. Fix punctuation (add periods, commas, question marks where appropriate)
2. Capitalize the first letter of sentences and proper nouns
3. Remove filler words (um, uh, like, you know) but keep natural speech patterns
4. Fix obvious transcription errors (homophones, common misrecognitions)
5. Add paragraph breaks for natural pauses (2+ seconds of silence)
6. Do NOT change the speaker's wording or phrasing unless clearly wrong
7. Preserve any technical terms, names, or specialized vocabulary
8. Output ONLY the cleaned text, no explanations or comments

The input is a raw transcription from a speech-to-text engine. Output the cleaned version.
```

## Example Usage (Groq API)

```python
import groq

client = groq.Groq(api_key=os.environ.get("GROQ_API_KEY"))

response = client.chat.completions.create(
    model="llama-3.1-8b-instant",  # Fast, cheap model for text cleanup
    messages=[
        {"role": "system", "content": "<system prompt above>"},
        {"role": "user", "content": f"Clean up this transcription:\n\n{raw_text}"}
    ],
    temperature=0.1,  # Low temp for consistent output
    max_tokens=len(raw_text) * 2  # Allow for expansion
)

cleaned_text = response.choices[0].message.content
```

## Integration with Typr

Add as an optional post-processing step in `src-tauri/src/cleanup.rs`:

```rust
pub async fn cleanup_with_llm(raw_text: &str) -> Result<String, String> {
    // Call LLM API with the above prompt
    // Fall back to basic cleanup if API fails
}
```

## When to Use

- **Enable for cloud transcription**: API call is already happening, minimal extra cost
- **Optional for local transcription**: Adds latency and potential API cost
- **User toggle in settings**: "Enhanced text formatting" checkbox
