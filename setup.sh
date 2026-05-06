#!/bin/bash
# Setup development environment for Typr
# Installs Rust, Node.js (via nvm), and project dependencies

set -e

echo "=== Typr Development Setup ==="
echo ""

# Install Rust
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed: $(rustc --version)"
else
    echo "✓ Rust already installed: $(rustc --version)"
fi

# Install Node.js via nvm (macOS)
if ! command -v node &> /dev/null; then
    echo ""
    echo "Installing Node.js..."
    if ! command -v nvm &> /dev/null; then
        # Install nvm
        curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
        export NVM_DIR="$HOME/.nvm"
        [ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"
    fi
    nvm install 20
    nvm use 20
    echo "✓ Node.js installed: $(node --version)"
else
    echo "✓ Node.js already installed: $(node --version)"
fi

# Install project dependencies
echo ""
echo "Installing project dependencies..."
npm install

# Create .env from example if needed
if [ ! -f .env ]; then
    echo ""
    echo "Creating .env from .env.example..."
    cp .env.example .env
    echo "⚠️  Edit .env and add your GROQ_API_KEY"
fi

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Next steps:"
echo "1. Edit .env and add your GROQ_API_KEY"
echo "2. Build for macOS: ./build.sh"
echo "3. Or run in dev mode: npm run tauri dev"
echo ""
echo "For Windows EXE:"
echo "  - Push code to GitHub"
echo "  - Create a release tag: git tag v0.1.0 && git push origin v0.1.0"
echo "  - GitHub Actions will build Windows EXE automatically"
