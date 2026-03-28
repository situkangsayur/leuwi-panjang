# Getting Started with Leuwi Panjang Terminal

## Installation

### From Source (Rust)
```bash
# Prerequisites
# - Rust 1.75+ (rustup.rs)
# - wgpu dependencies (vulkan-loader on Linux, or Metal on macOS)

git clone git@github.com:situkangsayur/leuwi-panjang.git
cd leuwi-panjang
cargo build --release
cargo install --path .
```

### Package Managers (planned)
```bash
# Arch Linux (AUR)
yay -S leuwi-panjang

# Ubuntu/Debian
sudo apt install leuwi-panjang

# macOS
brew install leuwi-panjang

# Flatpak
flatpak install flathub com.situkangsayur.LeuwiPanjang

# Windows (winget)
winget install situkangsayur.LeuwiPanjang
```

## First Launch

On first launch, Leuwi Panjang will:
1. Create config directory at `~/.config/leuwi-panjang/`
2. Generate default `config.toml`
3. Detect your default shell (usually zsh)
4. Apply the default theme ("Leuwi Dark")
5. Open a single terminal tab

## Quick Tour

### The Interface

```
┌─────────────────────────────────────────────────────────────┐
│ Tab 1 │                                              │ ≡ │  <- Tab bar (IS the header)
├────────────────────────────────────────────────────────┼────┤
│                                                            │
│   user@hostname ~/                                        │
│  ❯                                                         │
│                                                            │
│                                                            │
├────────────────────────────────────────────────────────────┤
│ [git:main] [14:32]                                         │  <- Status bar
└────────────────────────────────────────────────────────────┘
```

### Essential Shortcuts

| Action | Shortcut |
|--------|----------|
| New tab | `Ctrl+Shift+T` |
| Split horizontal | `Ctrl+Shift+H` |
| Split vertical | `Ctrl+Shift+V` |
| Navigate panes | `Alt+Arrow` |
| Search tabs/panes | `Ctrl+Shift+Space` |
| Copy | `Ctrl+Shift+C` |
| Paste | `Ctrl+Shift+V` |
| Search | `Ctrl+Shift+F` |
| Settings | `Ctrl+Shift+,` |
| Zoom in/out | `Ctrl++` / `Ctrl+-` |

### Command Suggestions

Start typing and Leuwi Panjang suggests:
- Commands from your history (ghost text)
- Available flags with descriptions (dropdown)
- File paths (tab completion)

Press `Right Arrow` to accept ghost text, `Tab` to cycle suggestions, `Esc` to dismiss.

## Configuration

### Config File
Edit `~/.config/leuwi-panjang/config.toml`:

```toml
[general]
default_shell = "/bin/zsh"

[appearance]
theme = "leuwi-dark"
font_family = "JetBrains Mono"
font_size = 13.0
background_opacity = 0.95
corner_radius = 12

[keybindings]
split_horizontal = "ctrl+shift+h"
split_vertical = "ctrl+shift+v"
```

### GUI Settings
Open via `Ctrl+Shift+,` or hamburger menu (≡) -> Settings.

### Themes
Browse themes: `Ctrl+Shift+Alt+T` or hamburger menu -> Themes.

## Installing Plugins

```bash
# Install Claude AI integration
leuwi plugin install ai-claude

# Install Gemini AI integration
leuwi plugin install ai-gemini

# List installed plugins
leuwi plugin list

# Remove a plugin
leuwi plugin remove ai-gemini
```

## Setting Up Nvim Integration

See the [nvim-leuwi-panjang](https://github.com/situkangsayur/nvim-leuwi-panjang) repo for the lightweight Neovim IDE config that pairs perfectly with Leuwi Panjang Terminal.

```bash
# Backup existing nvim config
mv ~/.config/nvim ~/.config/nvim.bak

# Clone nvim-leuwi-panjang
git clone git@github.com:situkangsayur/nvim-leuwi-panjang.git ~/.config/nvim

# Launch nvim (plugins auto-install on first run)
nvim
```

## Next Steps

- [Core Features](../features/01-core-features.md)
- [Plugin System](../features/02-plugin-system.md)
- [AI Integration](../features/03-ai-integration.md)
- [Keymaps Reference](../guides/02-keymaps.md)
- [Configuration Reference](../guides/03-configuration.md)
