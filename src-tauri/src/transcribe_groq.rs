use reqwest::multipart;
use std::path::PathBuf;

pub async fn transcribe_groq(audio_path: &PathBuf) -> Result<String, String> {
    // Read API key from environment variable
    let api_key = std::env::var("GROQ_API_KEY")
        .map_err(|_| "GROQ_API_KEY environment variable not set. Please add it to your .env file or system environment.".to_string())?;

    let audio_bytes = std::fs::read(audio_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    let file_part = multipart::Part::bytes(audio_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let form = multipart::Form::new()
        .text("model", "whisper-large-v3-turbo")
        .text("language", "en")
        .text("response_format", "json")
        .part("file", file_part);

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.groq.com/openai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Groq API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Groq API error ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Groq response: {}", e))?;

    json["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("No 'text' field in Groq response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_missing_env_var() {
        // Test that missing GROQ_API_KEY env var returns error
        let path = PathBuf::from("/tmp/test.wav");
        // Ensure env var is not set for this test
        std::env::remove_var("GROQ_API_KEY");
        let result = transcribe_groq(&path).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("GROQ_API_KEY"));
    }
}
