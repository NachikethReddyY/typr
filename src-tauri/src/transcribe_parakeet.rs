use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

/// Transcribe audio using NVIDIA Parakeet TDT 0.6B v2 (local)
/// Uses Python + NeMo/Transformers as sidecar
pub async fn transcribe_parakeet(
    app: &AppHandle,
    audio_path: &PathBuf,
) -> Result<String, String> {
    // Call Python sidecar that runs Parakeet via NeMo or HuggingFace Transformers
    let output = app
        .shell()
        .sidecar("parakeet-transcribe")
        .map_err(|e| format!("Failed to create Parakeet sidecar: {}", e))?
        .args([
            audio_path.to_str().unwrap(),
        ])
        .output()
        .await
        .map_err(|e| format!("Parakeet sidecar failed: {}", e))?;

    if output.status.code() != Some(0) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Parakeet error: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(text)
}
