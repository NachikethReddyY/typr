use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};

use crate::audio::AudioRecorder;
use crate::cleanup;
use crate::cleanup::cleanup_text;
use crate::paste::paste_text;
use crate::settings::Settings;
use crate::transcribe_local;
use crate::transcribe_groq;

/// Calculate audio duration in seconds (16kHz, 16-bit, mono WAV)
fn get_audio_duration(audio_path: &PathBuf) -> Result<f64, String> {
    let metadata = std::fs::metadata(audio_path)
        .map_err(|e| format!("Failed to read audio metadata: {}", e))?;
    let file_size = metadata.len();
    if file_size <= 44 {
        return Err("Audio file too small".to_string());
    }
    // WAV header is 44 bytes, 32000 bytes per second (16kHz * 2 bytes * 1 channel)
    let audio_bytes = file_size - 44;
    Ok(audio_bytes as f64 / 32000.0)
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum RecordingState {
    Ready,
    Recording,
    Transcribing,
}

fn update_overlay(app: &AppHandle, state: &RecordingState) {
    if let Some(overlay) = app.get_webview_window("overlay") {
        let class = match state {
            RecordingState::Ready => "mic",
            RecordingState::Recording => "mic recording",
            RecordingState::Transcribing => "mic transcribing",
        };
        let js = format!("document.getElementById('mic').className = '{}';", class);
        let _ = overlay.eval(&js);
    }
}

pub struct Recorder {
    state: Arc<Mutex<RecordingState>>,
    audio_recorder: Arc<Mutex<AudioRecorder>>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RecordingState::Ready)),
            audio_recorder: Arc::new(Mutex::new(AudioRecorder::new())),
        }
    }

    pub fn get_state(&self) -> RecordingState {
        self.state.lock().unwrap().clone()
    }

    pub fn start_recording(&self, app: &AppHandle, mic_name: &str) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        if *state != RecordingState::Ready {
            return Err("Already recording or transcribing".to_string());
        }

        let mut recorder = self.audio_recorder.lock().unwrap();
        recorder.start(mic_name)?;

        *state = RecordingState::Recording;
        let _ = app.emit("recording-state", RecordingState::Recording);
        update_overlay(app, &RecordingState::Recording);
        Ok(())
    }

    pub async fn stop_and_transcribe(
        &self,
        app: &AppHandle,
        settings: &Settings,
        app_dir: &PathBuf,
    ) -> Result<String, String> {
        // Stop recording
        {
            let mut state = self.state.lock().unwrap();
            if *state != RecordingState::Recording {
                return Err("Not currently recording".to_string());
            }
            *state = RecordingState::Transcribing;
            let _ = app.emit("recording-state", RecordingState::Transcribing);
            update_overlay(app, &RecordingState::Transcribing);
        }

        let temp_path = app_dir.join("temp_recording.wav");

        // Save audio
        {
            let mut recorder = self.audio_recorder.lock().unwrap();
            recorder.stop_and_save(&temp_path)?;
        }

        // Transcribe with auto-routing logic
        let raw_text = match settings.engine.as_str() {
            "local" => {
                let model_path = app_dir.join(transcribe_local::model_filename(&settings.whisper_model));
                transcribe_local::transcribe_local(app, &model_path, &temp_path).await?
            }
            "cloud" => {
                // Route to selected cloud provider
                match settings.cloud_provider.as_str() {
                    "mistral" => {
                        transcribe_mistral::transcribe_mistral(
                            &settings.mistral_api_key,
                            &temp_path,
                            "voxtral-mini-transcribe-2602".to_string(),
                        ).await?
                    }
                    _ => transcribe_groq::transcribe_groq(&settings.groq_api_key, &temp_path).await?,
                }
            }
            "auto" => {
                // Auto-route: <90s local, >90s cloud
                let duration = get_audio_duration(&temp_path)?;
                println!("[Typr] Auto mode: audio duration {:.2}s", duration);
                
                if duration > 90.0 {
                    println!("[Typr] Auto: >90s, using cloud engine");
                    transcribe_groq::transcribe_groq(&settings.groq_api_key, &temp_path).await?
                } else {
                    println!("[Typr] Auto: <90s, using local engine");
                    let model_path = app_dir.join(transcribe_local::model_filename(&settings.whisper_model));
                    transcribe_local::transcribe_local(app, &model_path, &temp_path).await?
                }
            }
            _ => return Err(format!("Unknown engine: {}", settings.engine)),
        };

        // Cleanup temp file
        let _ = std::fs::remove_file(&temp_path);

        // Clean up text
        let cleaned = if settings.enhanced_formatting {
            match cleanup::cleanup_with_llm(&raw_text).await {
                Ok(llm_cleaned) => {
                    println!("[Typr] Used LLM for text cleanup");
                    llm_cleaned
                }
                Err(e) => {
                    eprintln!("[Typr] LLM cleanup failed, using basic: {}", e);
                    cleanup_text(&raw_text)
                }
            }
        } else {
            cleanup_text(&raw_text)
        };

        // Auto-paste
        if !cleaned.is_empty() {
            paste_text(&cleaned)?;
        }

        // Reset state
        {
            let mut state = self.state.lock().unwrap();
            *state = RecordingState::Ready;
            let _ = app.emit("recording-state", RecordingState::Ready);
            update_overlay(app, &RecordingState::Ready);
        }

        Ok(cleaned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_ready() {
        let recorder = Recorder::new();
        assert_eq!(recorder.get_state(), RecordingState::Ready);
    }
}
