pub fn cleanup_text(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Normalize multiple spaces to single space
    let normalized: String = trimmed
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");

    // Capitalize first letter of each sentence
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in normalized.chars() {
        if capitalize_next && ch.is_alphabetic() {
            result.extend(ch.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(ch);
            if ch == '.' || ch == '!' || ch == '?' {
                capitalize_next = true;
            }
        }
    }

    // Ensure ending punctuation
    if let Some(last) = result.chars().last() {
        if !matches!(last, '.' | '!' | '?') {
            result.push('.');
        }
    }

    result
}

/// Clean up text using LLM API for better formatting (removes fillers, fixes punctuation)
pub async fn cleanup_with_llm(text: &str) -> Result<String, String> {
    let api_key = std::env::var("GROQ_API_KEY")
        .map_err(|_| "GROQ_API_KEY not set for LLM cleanup".to_string())?;

    let client = reqwest::Client::new();
    
    let system_prompt = "You are a professional transcription editor. Clean up raw speech-to-text output. \
        Fix punctuation and capitalization. Remove filler words (um, uh, like, you know). \
        Do NOT change wording unless clearly wrong. Output ONLY the cleaned text.";
    
    let user_prompt = format!("Clean up this transcription:\n\n{}", text);
    
    let body = serde_json::json!({
        "model": "llama-3.1-8b-instant",  // Fast, cheap model for cleanup
        "messages": [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt}
        ],
        "temperature": 0.1,
        "max_tokens": text.len() * 2
    });

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("LLM cleanup request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("LLM cleanup error: {}", response.status()));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

    let cleaned = json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or(text)
        .trim()
        .to_string();

    Ok(if cleaned.is_empty() { text.to_string() } else { cleaned })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_whitespace() {
        assert_eq!(cleanup_text("  hello world  "), "Hello world.");
    }

    #[test]
    fn test_normalize_spaces() {
        assert_eq!(cleanup_text("hello    world"), "Hello world.");
    }

    #[test]
    fn test_capitalize_first_letter() {
        assert_eq!(cleanup_text("hello world"), "Hello world.");
    }

    #[test]
    fn test_capitalize_after_period() {
        assert_eq!(cleanup_text("hello. world"), "Hello. World.");
    }

    #[test]
    fn test_capitalize_after_question_mark() {
        assert_eq!(cleanup_text("hello? world"), "Hello? World.");
    }

    #[test]
    fn test_capitalize_after_exclamation() {
        assert_eq!(cleanup_text("hello! world"), "Hello! World.");
    }

    #[test]
    fn test_ensure_ending_punctuation() {
        assert_eq!(cleanup_text("hello world"), "Hello world.");
    }

    #[test]
    fn test_preserve_existing_ending_punctuation() {
        assert_eq!(cleanup_text("hello world."), "Hello world.");
        assert_eq!(cleanup_text("hello world!"), "Hello world!");
        assert_eq!(cleanup_text("hello world?"), "Hello world?");
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(cleanup_text(""), "");
        assert_eq!(cleanup_text("   "), "");
    }

    #[test]
    fn test_already_clean() {
        assert_eq!(cleanup_text("Hello world."), "Hello world.");
    }
}
