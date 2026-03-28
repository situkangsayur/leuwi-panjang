#!/usr/bin/env bash
# Leuwi Panjang Terminal — macOS Installer
set -e

echo "╔══════════════════════════════════════════╗"
echo "║   Leuwi Panjang Terminal — macOS         ║"
echo "╚══════════════════════════════════════════╝"

# Check Rust
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Clone/update
INSTALL_DIR="$HOME/.local/share/leuwi-panjang"
if [ -d "$INSTALL_DIR" ]; then
    cd "$INSTALL_DIR" && git pull
else
    git clone https://github.com/situkangsayur/leuwi-panjang.git "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# Build
cargo build --release

# Install
mkdir -p "$HOME/.local/bin"
cp target/release/leuwi-panjang "$HOME/.local/bin/"

echo "✓ Installed! Run: leuwi-panjang"
