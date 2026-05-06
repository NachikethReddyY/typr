# Shipping Typr to Real Users (macOS)

## Quick Answer
When you run `npm run tauri build`, it creates a **Typr.app** bundle in `src-tauri/target/release/bundle/macos/`. Users just download it and drag to Applications — no developer tools needed.

---

## Step-by-Step Shipping Process

### 1. Build the Release Version
```bash
# Install dependencies (developers only)
npm install

# Build release version (creates .app bundle)
npm run tauri build
```

**Output location**:
```
src-tauri/target/release/bundle/macos/Typr.app
src-tauri/target/release/bundle/dmg/Typr_0.1.0_aarch64.dmg  (if DMG created)
```

### 2. Code Signing (REQUIRED for macOS)

Without signing, users see: *"Typr cannot be opened because the developer cannot be verified"*

**Get Apple Developer ID**:
1. Join [Apple Developer Program](https://developer.apple.com/programs/) ($99/year)
2. Create Certificate:
   - Xcode → Settings → Accounts → Manage Certificates
   - Click "+" → "Developer ID Application"
3. Download and install the certificate to Keychain

**Configure Tauri** (`src-tauri/tauri.conf.json`):
```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAMID)",
      "hardenedRuntime": true,
      "notarize": true  // Optional: upload to Apple for scanning
    }
  }
}
```

**Set environment variables** (never commit these):
```bash
export APPLE_CERTIFICATE="Developer ID Application: Your Name"
export APPLE_API_KEY="path/to/api-key.p8"
export APPLE_API_KEY_ID="your-key-id"
export APPLE_API_ISSUER="your-issuer-id"
```

### 3. Create Distribution Package

**Option A: DMG (Recommended)**
```bash
# Tauri can create DMG automatically
# In tauri.conf.json:
{
  "bundle": {
    "dmg": {
      "background": "assets/dmg-background.png",  // Optional
      "window": {"width": 660, "height": 400},
      "align": {
        "applications": "bottom-right",
        "typr": "bottom-left"
      }
    }
  }
}
```

**Option B: ZIP** (Simpler)
```bash
cd src-tauri/target/release/bundle/macos/
zip -r Typr-v0.1.0.zip Typr.app
```

### 4. Distribute to Users

**Option A: GitHub Releases** (Free, Easy)
```bash
# Tag a release
git tag v0.1.0
git push origin v0.1.0

# Go to GitHub → Releases → Create New Release
# Upload the .dmg or .zip file
# Users download from: https://github.com/yourname/typr/releases
```

**Option B: Homebrew** (macOS Package Manager)
```bash
# Create tap repository: homebrew-typr
# Add formula:
# Formula content tells brew how to install your app
brew install --cask typr
```

**Option C: Your Website**
- Host the DMG/ZIP file
- Link to it from your landing page
- Example: "Download Typr for macOS"

---

## User Installation Flow

1. User downloads `Typr-v0.1.0.dmg` (or `.zip`)
2. Opens the DMG file
3. Drags **Typr.app** to **Applications** folder
4. First launch: Right-click → "Open" (if not notarized)
5. Grant microphone permissions when prompted
6. Done! App appears in menu bar

---

## Critical: Python Sidecar Issue

**Problem**: The Parakeet Python script won't work for regular users (they don't have Python + PyTorch installed).

**Solutions**:

| Option | Pros | Cons |
|--------|------|------|
| **whisper.cpp** | Already works, no deps | Slightly slower than Parakeet |
| **Bundle Python** | Full Parakeet support | Large app size (+500MB) |
| **ONNX Runtime** | Fast, no Python needed | Needs conversion work |
| **Cloud-only** | Easy, just Groq API | Requires internet |

**Recommendation for v1**:
- Ship with **whisper.cpp** for local mode (already working)
- Use **Groq API** for cloud mode (already implemented)
- Add Parakeet later when ONNX bundling is sorted

---

## Checklist Before Shipping

- [ ] Test on a **clean Mac** (no dev tools installed)
- [ ] Verify microphone permission dialog appears
- [ ] Test both local and cloud transcription
- [ ] Set `GROQ_API_KEY` env var in app (or use settings UI)
- [ ] Code sign the app (or accept warning dialogs)
- [ ] Create DMG/ZIP distribution package
- [ ] Write a simple landing page or GitHub README
- [ ] Add "Made with Typr" sample text to README

---

## Quick Build Command Reference

```bash
# Development (with hot reload)
npm run tauri dev

# Production build (creates distributable)
npm run tauri build

# Output locations:
# - App bundle: src-tauri/target/release/bundle/macos/Typr.app
# - DMG installer: src-tauri/target/release/bundle/dmg/Typr_*.dmg
# - Windows .exe: src-tauri/target/release/bundle/nsis/Typr_*.exe
```

---

## For Windows Users

Same process, different output:
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/nsis/Typr_0.1.0_x64-setup.exe
# Users just run the .exe installer
```
