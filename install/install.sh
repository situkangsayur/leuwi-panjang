#!/usr/bin/env bash
# Leuwi Panjang Terminal — Universal Installer (Linux/macOS/Unix)
set -e

REPO="https://github.com/situkangsayur/leuwi-panjang.git"
INSTALL_DIR="$HOME/.local/share/leuwi-panjang"
BIN_DIR="$HOME/.local/bin"
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
DESKTOP_DIR="$HOME/.local/share/applications"

echo "╔══════════════════════════════════════════╗"
echo "║   Leuwi Panjang Terminal — Installer     ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Check Rust
if ! command -v cargo &>/dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Check dependencies (Linux)
if [ "$(uname)" = "Linux" ]; then
    echo "Checking Linux dependencies..."
    MISSING=""
    for pkg in libasound2-dev libpulse-dev; do
        dpkg -s "$pkg" &>/dev/null || MISSING="$MISSING $pkg"
    done
    if [ -n "$MISSING" ]; then
        echo "Installing:$MISSING"
        sudo apt-get install -y $MISSING
    fi
fi

# Clone or update
if [ -d "$INSTALL_DIR" ]; then
    echo "Updating..."
    cd "$INSTALL_DIR"
    git pull
else
    echo "Cloning..."
    git clone "$REPO" "$INSTALL_DIR"
    cd "$INSTALL_DIR"
fi

# Build release
echo "Building (this may take a few minutes)..."
cargo build --release

# Install binary
mkdir -p "$BIN_DIR"
cp target/release/leuwi-panjang "$BIN_DIR/"
chmod +x "$BIN_DIR/leuwi-panjang"

# Install icon (Linux)
if [ "$(uname)" = "Linux" ]; then
    mkdir -p "$ICON_DIR" "$DESKTOP_DIR"
    cp assets/icons/logo-256.png "$ICON_DIR/leuwi-panjang.png" 2>/dev/null || true

    cat > "$DESKTOP_DIR/leuwi-panjang.desktop" << DESKTOP
[Desktop Entry]
Name=Leuwi Panjang Terminal
Comment=Lightweight GPU-accelerated terminal emulator
Exec=$BIN_DIR/leuwi-panjang
Icon=leuwi-panjang
Terminal=false
Type=Application
Categories=System;TerminalEmulator;
DESKTOP
    chmod +x "$DESKTOP_DIR/leuwi-panjang.desktop"
fi

# Add to PATH if needed
if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$HOME/.bashrc"
    echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$HOME/.zshrc" 2>/dev/null || true
    echo "Added $BIN_DIR to PATH (restart shell)"
fi

echo ""
echo "✓ Leuwi Panjang Terminal installed!"
echo "  Run: leuwi-panjang"
echo "  Binary: $BIN_DIR/leuwi-panjang"
echo "  Size: $(du -h "$BIN_DIR/leuwi-panjang" | cut -f1)"
echo ""

# Install nvim config (optional)
read -p "Install nvim-leuwi-panjang config? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    NVIM_DIR="$HOME/.config/nvim-leuwi-panjang"
    git clone https://github.com/situkangsayur/nvim-leuwi-panjang.git "$NVIM_DIR" 2>/dev/null || (cd "$NVIM_DIR" && git pull)
    echo "✓ Nvim config installed at $NVIM_DIR"
    echo "  Run: NVIM_APPNAME=nvim-leuwi-panjang nvim"
fi
