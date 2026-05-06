use reqwest::multipart;
use std::path::PathBuf;

/// Transcribe audio using Mistral Voxtral API
pub async fn transcribe_mistral(
    api_key: &str,
    audio_path: &PathBuf,
    model: String,
) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("Mistral API key not set. Please enter your API key in settings.".to_string());
    }

    let audio_bytes = std::fs::read(audio_path)
        .map_err(|e| format!("Failed to read audio file: {}", e))?;

    let file_part = multipart::Part::bytes(audio_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| e.to_string())?;

    let form = multipart::Form::new()
        .text("model", model)
        .text("language", "en")
        .text("response_format", "json")
        .part("file", file_part);

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.mistral.ai/v1/audio/transcriptions")
        .header("Authorization", format!("Bearer {}", api_key))
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Mistral API request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Mistral API error ({}): {}", status, body));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Mistral response: {}", e))?;

    json["text"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or("No 'text' field in Mistral response".to_string())
}
