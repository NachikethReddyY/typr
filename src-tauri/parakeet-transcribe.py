#!/usr/bin/env python3
"""
Parakeet TDT 0.6B v2 ASR Sidecar for Typr
Usage: python parakeet_transcribe.py <audio_path>
Output: Transcribed text to stdout
"""
import sys
import os
import warnings
warnings.filterwarnings("ignore")

try:
    from transformers import AutoProcessor, AutoModelForCTC
    import torch
    import soundfile as sf
except ImportError as e:
    print(f"Error: Missing dependencies. Install with: pip install transformers torch soundfile", file=sys.stderr)
    sys.exit(1)

MODEL_NAME = "nvidia/parakeet-tdt-0.6b-v2"

def transcribe(audio_path):
    try:
        # Load model and processor
        processor = AutoProcessor.from_pretrained(MODEL_NAME)
        model = AutoModelForCTC.from_pretrained(MODEL_NAME)
        
        # Load audio
        audio_input, sample_rate = sf.read(audio_path)
        
        # Resample to 16kHz if needed
        if sample_rate != 16000:
            import librosa
            audio_input = librosa.resample(audio_input, orig_sr=sample_rate, target_sr=16000)
        
        # Process
        inputs = processor(audio_input, sampling_rate=16000, return_tensors="pt")
        
        with torch.no_grad():
            outputs = model(**inputs)
        
        # Decode
        predicted_ids = torch.argmax(outputs.logits, dim=-1)
        transcription = processor.batch_decode(predicted_ids)[0]
        
        # Clean up (remove special tokens)
        transcription = transcription.strip()
        
        print(transcription)
        return 0
        
    except Exception as e:
        print(f"Transcription error: {e}", file=sys.stderr)
        return 1

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python parakeet_transcribe.py <audio_path>", file=sys.stderr)
        sys.exit(1)
    
    audio_path = sys.argv[1]
    sys.exit(transcribe(audio_path))
