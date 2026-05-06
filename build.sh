#!/bin/bash
# Build script for Typr - creates distributable DMG (macOS) and prepares for Windows EXE
# Usage: ./build.sh

set -e  # Exit on error

echo "=== Typr Build Script ==="

# Check if .env exists, if not create from example
if [ ! -f .env ]; then
    echo "Creating .env from .env.example..."
    cp .env.example .env
    echo "⚠️  Please edit .env and add your GROQ_API_KEY"
    echo "   Then run this script again."
    exit 1
fi

# Load environment variables
export $(grep -v '^#' .env | xargs)

# Check if GROQ_API_KEY is set
if [ -z "$GROQ_API_KEY" ]; then
    echo "❌ Error: GROQ_API_KEY not set in .env file"
    echo "   Get your key from https://console.groq.com"
    exit 1
fi

echo "✓ Environment loaded"

# Check dependencies
echo ""
echo "Checking dependencies..."

if ! command -v node &> /dev/null; then
    echo "❌ Node.js not found. Install from https://nodejs.org"
    exit 1
fi
echo "✓ Node.js: $(node --version)"

if ! command -v npm &> /dev/null; then
    echo "❌ npm not found"
    exit 1
fi
echo "✓ npm: $(npm --version)"

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Install from https://rustup.rs"
    exit 1
fi
echo "✓ Rust: $(rustc --version)"

# Install npm dependencies
echo ""
echo "Installing npm dependencies..."
npm install

# Build for current platform (macOS)
echo ""
echo "Building for macOS..."
npm run tauri build

# Show results
echo ""
echo "=== Build Complete ==="
echo ""
echo "macOS artifacts:"
if [ -d "src-tauri/target/release/bundle/dmg" ]; then
    echo "  DMG files:"
    ls -lh src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null || echo "    (no DMG found)"
fi
if [ -d "src-tauri/target/release/bundle/macos" ]; then
    echo "  App bundle:"
    ls -lh src-tauri/target/release/bundle/macos/*.app 2>/dev/null || echo "    (no .app found)"
fi

echo ""
echo "=== Next Steps ==="
echo "1. Test the app:"
echo "   open src-tauri/target/release/bundle/macos/Typr.app"
echo ""
echo "2. For Windows EXE, push to GitHub and create a release tag:"
echo "   git tag v0.1.0"
echo "   git push origin v0.1.0"
echo "   (GitHub Actions will build Windows EXE automatically)"
echo ""
echo "3. Or download Windows EXE from GitHub Actions artifacts"
