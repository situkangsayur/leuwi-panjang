# Leuwi Panjang - Implementation Roadmap

## Phase Overview

```
Phase 1 ████████░░░░ Core Terminal (Months 1-3)
Phase 2 ░░░░████████ Advanced Features (Months 3-5)
Phase 3 ░░░░░░░░████ Plugin System (Months 5-7)
Phase 4 ░░██████░░░░ Nvim Config (Months 2-3, parallel)
Phase 5 ░░░░░░░░░░██████ Mobile App (Months 7-10)
Phase 6 ░░░░░░░░░░░░████ WireGuard Server (Months 8-10)
Phase 7 ░░░░░░░░░░░░░░██ Polish & Release (Months 10-12)
```

---

## Phase 1: Core Terminal (Months 1-3)

### 1.1 Project Setup
- [ ] Cargo workspace structure (see [Rust Ecosystem](../research/04-rust-terminal-ecosystem.md))
- [x] CI/CD pipeline (GitHub Actions: build, test, lint, cross-compile)
- [ ] Coding standards and contribution guidelines

### 1.2 Terminal Emulation (`leuwi-terminal` crate)
- [x] VT220 escape sequence parser
- [x] xterm extensions (256-color, true color, mouse, bracketed paste)
- [x] Screen buffer (cell grid with attributes)
- [x] Scrollback ring buffer
- [ ] Selection handling (word, line, block)
- [ ] URL detection (regex-based)
- [ ] Unicode / wide character support
- [ ] Tests against vttest and esctest suites

### 1.3 PTY Management (`leuwi-pty` crate)
- [x] PTY creation and management (portable-pty)
- [x] Shell spawning (default: zsh)
- [x] Environment variable setup (TERM, COLORTERM, etc.)
- [x] Signal handling (SIGWINCH for resize)
- [ ] Cross-platform (Linux, macOS, Windows)

### 1.4 GPU Rendering (`leuwi-renderer` crate)
- [ ] Makepad app setup and window creation
- [ ] Custom terminal grid shader via Makepad's draw system
- [ ] Glyph rasterization and texture atlas
- [ ] Cell grid rendering (GPU-accelerated)
- [x] Cursor rendering (block, beam, underline + blink)
- [ ] Selection highlight rendering
- [ ] Background rendering (solid, transparent, image)
- [ ] Ligature support (rustybuzz text shaping)
- [ ] Damage tracking (only redraw changed cells)

### 1.5 Window & Basic UI (`leuwi-ui` crate, Makepad)
- [ ] Makepad application structure
- [x] Chromeless window (no titlebar) via Makepad window config
- [ ] Rounded corners rendering
- [x] Tab bar widget (Chrome-style, part of content area)
- [x] Tab management (new, close, switch, reorder, drag)
- [ ] Keyboard input handling
- [ ] Mouse input handling
- [ ] Clipboard integration
- [ ] Window resize handling
- [ ] Hamburger menu (≡) widget

### 1.6 Configuration (`leuwi-config` crate)
- [ ] TOML config parsing (serde)
- [ ] Default config generation
- [ ] Config hot-reload (notify crate)
- [ ] Theme loading
- [ ] Default theme ("Leuwi Dark")
- [ ] Font configuration

### Milestone 1 Deliverable
A working terminal that can:
- Open a chromeless window with rounded corners
- Run zsh in a single tab
- Render text with GPU acceleration and ligatures
- Handle basic keyboard/mouse input
- Load TOML configuration

---

## Phase 2: Advanced Features (Months 3-5)

### 2.1 Split Panes
- [ ] Binary tree pane model
- [ ] Horizontal split (top/bottom)
- [ ] Vertical split (left/right)
- [ ] Pane navigation (Alt+Arrow)
- [ ] Pane resize (Ctrl+Alt+Arrow, drag divider)
- [ ] Pane maximize/restore
- [ ] Broadcast input mode
- [ ] Pane close/merge

### 2.2 Search
- [ ] Scrollback search UI
- [ ] Regex search engine
- [ ] Match highlighting
- [ ] Wrap-around search
- [ ] Case-sensitive toggle

### 2.3 Tab/Pane Search (Zen Browser-style)
- [ ] Fuzzy search dialog (Ctrl+Shift+Space)
- [ ] Search by: tab name, pane CWD, running command, hostname
- [ ] Quick-switch keyboard navigation
- [ ] Preview of pane content

### 2.4 Command Suggestion Engine (`leuwi-suggestions` crate)
- [ ] Suggestion engine architecture
- [ ] History provider (parse zsh/bash history)
- [ ] Path provider (filesystem scan)
- [ ] Command provider ($PATH scan)
- [ ] Flag provider (man page parsing + built-in specs)
- [ ] Git provider (branches, tags, remotes)
- [ ] Fuzzy matcher (skim algorithm)
- [ ] Ranking engine (frequency, recency, context)
- [ ] Inline ghost text rendering
- [ ] Dropdown suggestion panel rendering
- [ ] Built-in completion specs (git, docker, cargo, npm, etc.)

### 2.5 Shell Integration (`leuwi-shell-integration` crate)
- [ ] OSC 133 prompt marking
- [ ] OSC 7 CWD tracking
- [ ] Command exit status tracking
- [ ] Prompt navigation (Ctrl+Shift+Up/Down)
- [ ] "Select last command output"
- [ ] Zsh integration script
- [ ] Bash integration script
- [ ] Fish integration script
- [ ] Custom lightweight prompt (with OS logos, git info)

### 2.6 Image Protocols
- [ ] Kitty Graphics Protocol
- [ ] Sixel support
- [ ] iTerm2 inline image protocol
- [ ] Image rendering in GPU pipeline

### 2.7 Profile System
- [ ] Named profiles in config
- [ ] Profile switching per pane
- [ ] Auto-switch rules (hostname, CWD, SSH)
- [ ] Profile inheritance

### 2.8 Status Bar
- [ ] Modular component system
- [ ] Git component
- [ ] CWD component
- [ ] CPU/Memory components
- [ ] Clock component
- [ ] Custom component API (for plugins)

### 2.9 GUI Settings Panel
- [ ] Settings UI (rendered in terminal or as overlay)
- [ ] Theme browser with live preview
- [ ] Font picker
- [ ] Color picker
- [ ] Keybinding editor
- [ ] Changes write back to TOML

### Milestone 2 Deliverable
A feature-complete terminal with splits, search, suggestions, profiles, and shell integration.

---

## Phase 3: Plugin System (Months 5-7)

### 3.1 WASM Plugin Host (`leuwi-plugin-host` crate)
- [ ] wasmtime runtime integration
- [ ] Plugin loading from .wasm files
- [ ] Plugin manifest parsing (plugin.toml)
- [ ] Capability system (declare required permissions)
- [ ] Plugin lifecycle (init, events, shutdown)

### 3.2 Plugin API (`leuwi-plugin-sdk` crate)
- [ ] WIT (WASM Interface Types) definitions
- [ ] TerminalAPI implementation
- [ ] ConfigAPI implementation
- [ ] UIAPI implementation
- [ ] CredentialAPI implementation
- [ ] AuditAPI implementation
- [ ] NotificationAPI implementation
- [ ] Plugin SDK documentation

### 3.3 Permission System
- [ ] Permission gate UI (Allow/Deny dialog)
- [ ] Permission levels (always ask, session, always allow)
- [ ] Risk classification (low, medium, high, critical)
- [ ] Permission persistence

### 3.4 Credential Vault (`leuwi-crypto` crate)
- [ ] AES-256-GCM encryption
- [ ] Argon2id key derivation
- [ ] Master password management
- [ ] Auto-lock on inactivity
- [ ] Secure memory handling (secrecy crate, zeroize)

### 3.5 Audit Trail
- [ ] Append-only JSON Lines logging
- [ ] Log rotation (daily, compress after 7 days)
- [ ] Audit viewer UI
- [ ] Filter/search audit entries
- [ ] Export (CSV/JSON)

### 3.6 AI Claude Plugin
- [ ] Claude CLI detection and integration
- [ ] AI side panel UI
- [ ] Send selection to Claude
- [ ] Command suggestion from Claude
- [ ] Permission-gated command execution
- [ ] Conversation persistence

### 3.7 AI Gemini Plugin
- [ ] Same architecture as Claude plugin
- [ ] Gemini CLI integration

### 3.8 AI Ollama Plugin
- [ ] Ollama HTTP API integration
- [ ] Model selection
- [ ] Local inference (no network)

### Milestone 3 Deliverable
Working plugin system with AI integration, credential vault, and audit trail.

---

## Phase 4: Nvim Config (Months 2-3, parallel with Phase 1-2)

### 4.1 Base Setup
- [ ] lazy.nvim bootstrap
- [ ] Directory structure
- [ ] Core options (Lua)
- [ ] Core keymaps
- [ ] Autocmds

### 4.2 LSP
- [ ] nvim-lspconfig + mason.nvim
- [ ] All LSP servers (Java, Go, Rust, Python, TS, Kotlin, Shell, HTML, CSS, PHP)
- [ ] nvim-jdtls for Java
- [ ] rustaceanvim for Rust
- [ ] On-attach keymaps (gd, gr, gi, rename, code action, etc.)

### 4.3 Completion
- [ ] nvim-cmp setup
- [ ] LSP source, buffer source, path source
- [ ] LuaSnip + friendly-snippets
- [ ] Tab/Shift-Tab navigation

### 4.4 Navigation
- [ ] Telescope (find files, live grep, buffers, symbols)
- [ ] telescope-fzf-native for performance
- [ ] LSP integration (references, implementations, symbols)

### 4.5 Syntax
- [ ] Treesitter (all target languages)
- [ ] Textobjects (select/move by function, class, parameter)
- [ ] Incremental selection

### 4.6 Git
- [ ] gitsigns.nvim
- [ ] vim-fugitive
- [ ] diffview.nvim
- [ ] Telescope git integration

### 4.7 Debugging
- [ ] nvim-dap + nvim-dap-ui
- [ ] mason-nvim-dap
- [ ] DAP configs for all languages
- [ ] nvim-dap-go, nvim-dap-python

### 4.8 Editor Enhancements
- [ ] neo-tree (file explorer)
- [ ] which-key (keybinding discovery)
- [ ] toggleterm (terminal management)
- [ ] refactoring.nvim

### 4.9 AI Integration
- [ ] claude-code.nvim or toggleterm Claude
- [ ] avante.nvim for multi-provider AI
- [ ] Gemini via toggleterm

### 4.10 Documentation
- [ ] README for nvim-leuwi-panjang repo
- [ ] Keymap reference
- [ ] Language-specific guides

### Milestone 4 Deliverable
Complete nvim-leuwi-panjang config: 30-60ms startup, full IDE features, all target languages.

---

## Phase 5: Mobile Build (Months 7-10)

Same Rust codebase as desktop, compiled for mobile via Makepad. No separate app, no Flutter.

### 5.1 Mobile Adaptation
- [ ] Makepad mobile build targets (Android NDK, iOS)
- [ ] Mobile-specific layout (responsive UI for small screens)
- [ ] Touch input handling (tap, swipe, pinch, long-press)
- [ ] Extra key row widget (Tab, Ctrl, Alt, Esc, Arrows)
- [ ] On-screen keyboard integration
- [ ] Hardware keyboard support (Bluetooth, Samsung DeX)

### 5.2 SSH Client (Mobile Primary Use Case)
- [ ] russh integration (pure Rust SSH)
- [ ] SSH key generation/import
- [ ] SSH connection manager UI
- [ ] Connection groups/folders
- [ ] Quick-connect (one tap)
- [ ] Port forwarding
- [ ] Jump host support
- [ ] Mosh support
- [ ] Connection status indicators

### 5.3 Desktop Pairing (Embedded WireGuard)
- [ ] boringtun client mode on mobile
- [ ] QR code scanning for pairing
- [ ] 6-digit code pairing fallback
- [ ] Auto key exchange and tunnel setup
- [ ] Remote AI access via desktop tunnel

### 5.4 Platform-Specific
- [ ] Android: NDK build pipeline, min SDK 26
- [ ] Android: Samsung DeX support
- [ ] iOS: cargo-lipo build pipeline, min iOS 15
- [ ] iOS: External keyboard shortcuts
- [ ] OS-level keychain integration (Keystore/Keychain)

### 5.5 App Store
- [ ] Android: Play Store listing
- [ ] Android: F-Droid listing
- [ ] iOS: App Store listing

### Milestone 5 Deliverable
Mobile SSH terminal from same Rust codebase, with connection manager and desktop pairing.

---

## Phase 6: Embedded WireGuard Server (Months 8-10)

Server is embedded in desktop terminal, NOT a separate deployment.

### 6.1 Embedded WireGuard (boringtun)
- [ ] boringtun integration (userspace WireGuard, pure Rust)
- [ ] Auto key generation (x25519-dalek)
- [ ] Auto IP assignment (internal 10.lp.0.0/24 range)
- [ ] QR code generation for pairing (qrcode crate)
- [ ] 6-digit pairing code fallback
- [ ] One-time token handshake for key exchange
- [ ] Device registration and management
- [ ] Per-device permissions (AI, view, write, file transfer)

### 6.2 API Server (embedded in terminal, started on-demand)
- [ ] tokio + axum HTTP server on WireGuard interface
- [ ] Client authentication (device-based)
- [ ] Session management
- [ ] Zero overhead when no clients connected

### 6.3 AI Router
- [ ] Claude CLI proxy (forward to local claude)
- [ ] Gemini CLI proxy (forward to local gemini)
- [ ] Ollama proxy (forward to local ollama)
- [ ] Model routing logic
- [ ] Rate limiting per device

### 6.4 Device Management UI
- [ ] Pair new device screen (QR code + code)
- [ ] Connected devices list
- [ ] Per-device permission editor
- [ ] Revoke device access
- [ ] Server enable/disable toggle

### Milestone 6 Deliverable
Desktop terminal with embedded WireGuard server, zero-config pairing, AI routing for mobile clients.

---

## Phase 7: Polish & Release (Months 10-12)

### 7.1 Performance
- [ ] Profile and optimize startup
- [ ] Profile and optimize rendering
- [ ] Memory leak detection
- [ ] Benchmark suite

### 7.2 Cross-Platform Testing
- [ ] Linux (X11, Wayland, major distros)
- [ ] macOS (Intel, Apple Silicon)
- [ ] Windows (10, 11)
- [ ] Android (various devices)
- [ ] iOS (iPhone, iPad)

### 7.3 Distribution
- [ ] cargo install
- [ ] AUR package
- [ ] Homebrew formula
- [ ] APT/DEB package
- [ ] Flatpak
- [ ] Windows installer (winget, MSI)

### 7.4 Documentation
- [ ] Complete all docs
- [ ] Video demos
- [ ] Plugin development guide
- [ ] Theme creation guide

### 7.5 Community
- [ ] Plugin contribution guidelines
- [ ] Theme contribution guidelines
- [ ] Issue templates
- [ ] Discussion forum

### Milestone 7 Deliverable
Production-ready release v1.0 across all platforms.
