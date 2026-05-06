# Typr Implementation Plan: Auto-Router + Better STT Models

## Research Summary

### Current State
- **Local**: whisper.cpp (small ~466MB, medium ~1.5GB) — batch only, ~200x RTF
- **Cloud**: Groq API (whisper-large-v3-turbo) — batch only, requires API key

### Best STT Models (2026) — English Dictation

| Model | Type | WER (LibriSpeech clean) | Speed | Languages | License | Deployment |
|-------|------|------------------------|-------|-----------|---------|------------|
| **NVIDIA Parakeet TDT 0.6B v2** | Local | 1.69% | 3380x RTF (~80ms) | 25 (EU) | Apache 2.0 | Local via NeMo |
| **Cohere Transcribe 2B** | Local/API | ~5.4% avg | 3x faster than peers | 14 | Apache 2.0 | Local or Model Vault |
| **Mistral Voxtral Realtime 4B** | Local/API | ~7.7% avg | Configurable 240-2400ms | 13 | Apache 2.0 | Local (open weights) |
| **Qwen3-ASR-1.7B** | Local | Competitive | RTF 0.064 | 52 | Apache 2.0 | Local |
| **Whisper Large v3** (current) | Local | 2.7% | ~200x RTF | 99 | MIT | Local (current) |

### Recommendation
- **Replace local Whisper with NVIDIA Parakeet**: 2x better accuracy (1.69% vs 2.7% WER), 15x faster (80ms vs 1200ms latency), English-only but Typr v1 is English-only
- **Add Mistral Voxtral as cloud option**: 13 languages, streaming support, competitive API pricing ($0.006/min)
- **Keep Groq as cloud fallback**: Already implemented, fast API

---

## Implementation Plan

### Phase 1: Auto-Router Logic
**Files to modify**: `src-tauri/src/recorder.rs`, `src-tauri/src/settings.rs`

1. **Add duration calculation utility** in `recorder.rs`:
   ```rust
   fn get_audio_duration(audio_path: &PathBuf) -> f64 {
       // WAV: 16kHz, 16-bit, mono = 32000 bytes/sec
       // File size - 44 byte header / 32000 = duration in seconds
       let metadata = std::fs::metadata(audio_path).ok()?;
       let bytes = metadata.len().saturating_sub(44) as f64;
       Some(bytes / 32000.0)
   }
   ```

2. **Add "Auto" engine option** to `Settings`:
   - Options: `"local"` (Parakeet), `"cloud"` (Groq), `"auto"` (new)
   - Store in `config.json` as `engine: "auto"`

3. **Implement routing logic** in `stop_and_transcribe()`:
   ```
   if engine == "auto":
       duration = get_audio_duration(temp_path)
       if duration > 90.0:    // > 1min 30s → cloud (Groq)
           use groq_transcribe()
       else:                   // < 1min 30s → local (Parakeet)
           use parakeet_transcribe()
   ```

### Phase 2: Add NVIDIA Parakeet Local Engine
**New file**: `src-tauri/src/transcribe_parakeet.rs`

1. **Parakeet deployment approach**:
   - NVIDIA Parakeet runs via **NVIDIA NeMo** framework or **TensorRT**
   - For Tauri sidecar: Use Python + NeMo, or compile to ONNX runtime
   - Alternative: Use **HuggingFace Transformers** with ONNX export for faster inference

2. **Implementation** (Python sidecar approach similar to whisper.cpp):
   ```rust
   pub async fn transcribe_parakeet(
       app: &AppHandle,
       audio_path: &PathBuf,
   ) -> Result<String, String> {
       // Call Python sidecar with NeMo Parakeet
       let output = app.shell()
           .sidecar("parakeet-transcribe")
           .args(["--model", "nvidia/parakeet-tdt-0.6b-v2", "--input", audio_path])
           .output()
           .await?;
       // Parse output
   }
   ```

3. **Model download** (add to `downloader.rs`):
   - Download from HuggingFace: `nvidia/parakeet-tdt-0.6b-v2`
   - Size: ~600MB (similar to Whisper small)
   - Use ONNX format for faster inference without Python dependency

### Phase 3: Add Mistral Voxtral Cloud Option
**New file**: `src-tauri/src/transcribe_mistral.rs`

1. **Mistral API integration** (similar to Groq):
   ```rust
   pub async fn transcribe_mistral(
       api_key: &str,
       audio_path: &PathBuf,
   ) -> Result<String, String> {
       // POST to https://api.mistral.ai/v1/audio/transcriptions
       // Model: "voxtral-mini-transcribe-2602" (batch) or "voxtral-mini-transcribe-realtime-2602"
       // Same multipart form as Groq
   }
   ```

2. **Update settings** to support multiple cloud providers:
   - `cloud_provider: "groq" | "mistral"`
   - `mistral_api_key: String`

### Phase 4: Update Frontend UI
**Files to modify**: `src/index.html`, `src/main.ts`

1. **Engine selector**: Add "Auto" option with description:
   - "Auto: <1:30 local, >1:30 cloud"

2. **Model downloads**: Add Parakeet model download button

3. **Cloud provider selector**: Radio buttons for Groq vs Mistral

---

## File Changes Summary

| File | Change |
|------|--------|
| `src-tauri/src/settings.rs` | Add `engine: "local"|"cloud"|"auto"`, `cloud_provider: "groq"|"mistral"` |
| `src-tauri/src/recorder.rs` | Add `get_audio_duration()`, update `stop_and_transcribe()` with auto-routing |
| `src-tauri/src/transcribe_parakeet.rs` | **New**: Parakeet local transcription |
| `src-tauri/src/transcribe_mistral.rs` | **New**: Mistral cloud transcription |
| `src-tauri/src/main.rs` | Register new Tauri commands |
| `src/index.html` | Update engine selector UI |
| `src/main.ts` | Handle new settings fields |

---

## Recommended Order
1. ✅ **Phase 1**: Auto-router (easiest, biggest impact)
2. **Phase 2**: Parakeet local engine (best accuracy + speed)
3. **Phase 3**: Mistral cloud option (more language support)
4. **Phase 4**: UI updates

---

## Open Questions
1. **Parakeet deployment**: Python sidecar (NeMo) vs ONNX runtime? Python adds dependency, ONNX is faster but requires conversion.
2. **Auto-router threshold**: Currently 90s. Should this be configurable?
3. **Mistral streaming**: Voxtral Realtime supports WebSocket streaming — worth implementing for live dictation feedback?
