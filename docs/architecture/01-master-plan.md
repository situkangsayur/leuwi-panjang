# Leuwi Panjang Terminal - Master Architecture Plan

## Vision

Leuwi Panjang is a **lightweight, modern, GPU-accelerated terminal ecosystem** built primarily in Rust. It combines the best ideas from iTerm2 (splits, profiles, shell integration), Kitty (GPU rendering, extensions, chromeless mode), and GNOME Terminal (simplicity, stability) while avoiding their weaknesses.

## Project Components

```
leuwi-panjang (ecosystem) — ALL RUST, SINGLE CODEBASE
├── 1. leuwi-panjang          -- Terminal Emulator (Rust + Makepad)
│                                 Desktop: Linux, macOS, Windows
│                                 Mobile: Android, iOS
│                                 ONE codebase, ONE binary per platform
├── 2. nvim-leuwi-panjang     -- Neovim IDE Config (Lua)
├── 3. leuwi-panjang-plugins  -- Plugin Repository (WASM)
│   ├── plugin-claude          -- Claude CLI Integration
│   ├── plugin-gemini          -- Gemini CLI Integration
│   ├── plugin-ollama          -- Local AI (Ollama) Integration
│   └── ...                    -- Community plugins
└── 4. nvim config             -- Lightweight Neovim config
```

## Repositories

| Component | Repository |
|-----------|-----------|
| Terminal | git@github.com:situkangsayur/leuwi-panjang.git |
| Nvim Config | git@github.com:situkangsayur/nvim-leuwi-panjang.git |
| Plugins | git@github.com:situkangsayur/leuwi-panjang-plugins.git |

---

# Component 1: Leuwi Panjang Terminal Emulator

## Design Principles

1. **Lightweight** - fast startup (<100ms), low memory (<50MB base)
2. **GPU-accelerated** - wgpu for Vulkan/Metal/DX12/OpenGL rendering
3. **Chromeless by default** - no titlebar, tabs AS the header (Chrome-style)
4. **Standard SSH** - uses `TERM=xterm-256color`, SSH just works, no special commands
5. **Split everything** - horizontal + vertical splits, arbitrarily nested
6. **Plugin system** - WASM-based plugins for safe, sandboxed extensions
7. **Cross-platform** - Linux, macOS, Windows, Android, iOS
8. **Rust-first** - maximum performance, minimum resource usage

## Tech Stack

**100% Rust — Single Codebase for ALL platforms** (desktop + mobile)
**UI Framework: Makepad** — GPU-rendered, cross-platform Rust UI
**No Flutter, no Dart, no extra runtime**

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Language | **Rust** | Performance, safety, low power consumption |
| UI Framework | **Makepad** | Single Rust codebase for desktop + mobile, GPU-rendered, live design |
| GPU Rendering | Makepad's renderer (Metal/Vulkan/OpenGL/WebGL) | Built into Makepad, cross-API |
| Terminal Grid | Custom shader via Makepad's draw system | Cell-based GPU rendering within Makepad |
| Windowing | Makepad (handles windowing internally) | Cross-platform including mobile |
| Terminal Parsing | Custom (inspired by alacritty_terminal/vte) | Full control, optimized for our needs |
| PTY | portable-pty (desktop), russh (mobile SSH) | Platform-appropriate |
| Text Shaping | Makepad built-in + rustybuzz for terminal grid | Ligatures, complex scripts |
| Config | TOML + GUI settings | Human-readable, version-controllable |
| Plugin System | WASM (wasmtime) | Safe sandboxing, language-agnostic |
| IPC | Unix socket / Named pipe (Windows) | Remote control API |
| Serialization | serde + JSON/MessagePack | Plugin communication |
| Embedded WireGuard | boringtun (userspace, pure Rust) | Zero-config pairing, no system WG needed |
| Server Backend | Rust (tokio + axum), embedded in terminal | Started on-demand for mobile clients |

### Architecture Principle: 100% Rust, Single Codebase

```
Single Rust Codebase (Makepad)
├── Desktop builds: Linux, macOS, Windows
├── Mobile builds: Android, iOS
└── Same code, same UI framework, same binary per platform

NO Flutter, NO Dart VM, NO extra runtime
= Lowest possible memory, energy, and binary size
```

**Why Makepad?**
1. **Single Rust codebase** — desktop AND mobile from one source
2. **GPU-rendered UI** — Metal, Vulkan, OpenGL, WebGL
3. **No runtime overhead** — no Dart VM, no JavaScript, pure compiled Rust
4. **Lowest energy** — compiled native code, direct GPU, no interpreter
5. **Live design** — visual UI editor for rapid development
6. **Small binaries** — ~5-10MB vs ~15-30MB with Flutter
7. **Custom shaders** — Makepad's draw system allows custom GPU shaders for terminal grid rendering
8. **True cross-platform** — desktop + mobile + web from one codebase

## UI Design

### Window Layout (Chromeless - No Titlebar)

```
┌─────────────────────────────────────────────────────────────┐
│ Tab 1 │ Tab 2 │ Tab 3 │                              │ ≡ │ <- Tabs ARE the header bar
├────────────────────────────────────────────────────────┼────┤
│                                    │                   │    │
│   Terminal Pane 1                  │  Terminal Pane 2   │    │
│   user@machine ~/project           │  user@machine ~    │    │
│   $ ls -la                         │  $ htop            │    │
│   drwxr-xr-x  5 user ...          │  [htop output]     │    │
│                                    │                   │    │
│                                    │                   │    │
├────────────────────────────────────┴───────────────────│    │
│                                                        │    │
│   Terminal Pane 3                                      │    │
│   user@machine ~/project                               │    │
│   $ cargo build                                        │    │
│   Compiling leuwi-panjang v0.1.0                       │    │
│                                                        │    │
├────────────────────────────────────────────────────────┤    │
│ [git:main] [cpu:12%] [mem:4.2G] [14:32]               │    │ <- Status bar
└────────────────────────────────────────────────────────┴────┘
  ↑ Rounded corners (configurable radius)
```

### Key UI Elements

1. **Tab Bar = Header Bar** (Chrome-style)
   - Tabs on top, no separate titlebar
   - Drag to reorder tabs
   - `+` button for new tab
   - Hamburger menu (≡) at far right for settings, about, etc.
   - Window drag by grabbing empty tab bar area
   - Close tab: middle-click or `x` on hover

2. **Rounded Corners**
   - Configurable corner radius (default: 12px)
   - Works on all platforms via wgpu rendering

3. **Split Panes**
   - Horizontal split (top/bottom): `Ctrl+Shift+H` or configurable
   - Vertical split (side by side): `Ctrl+Shift+V` or configurable
   - Navigate: `Ctrl+Shift+Arrow` or `Alt+Arrow`
   - Resize: `Ctrl+Alt+Arrow`
   - Maximize/restore pane: `Ctrl+Shift+Enter`
   - Binary tree structure (like iTerm2) for arbitrary nesting

4. **Status Bar** (bottom, configurable)
   - Git branch, CPU, memory, clock, custom components
   - Can be hidden

5. **Command Suggestion/Autocomplete**
   - Inline suggestions as you type (like zsh-autosuggestions but built-in)
   - Dropdown suggestion panel for commands, flags, paths
   - History-based suggestions (from shell history)
   - Man page flag descriptions
   - Custom completions for common tools (git, docker, kubectl, npm, cargo, etc.)
   - Tab to accept, Esc to dismiss
   - Fuzzy matching

## Terminal Emulation

- **TERM**: `xterm-256color` (standard, no special SSH requirements)
- Full VT220 + xterm escape sequence support
- 24-bit true color
- Kitty Keyboard Protocol support (opt-in)
- Kitty Graphics Protocol + Sixel support (both!)
- iTerm2 inline image protocol support
- OSC 8 hyperlinks, OSC 7 CWD, OSC 52 clipboard
- OSC 133 prompt marking
- Bracketed paste, mouse reporting (all modes)
- Synchronized output (mode 2026)
- Ligatures via rustybuzz

## Configuration

### TOML Config File (`~/.config/leuwi-panjang/config.toml`)

```toml
[general]
default_shell = "/bin/zsh"
term = "xterm-256color"
startup_mode = "maximized"  # normal, maximized, fullscreen
confirm_close = true

[appearance]
theme = "leuwi-dark"
font_family = "JetBrains Mono"
font_size = 13.0
background_opacity = 0.95
corner_radius = 12
window_decorations = "none"  # none, titlebar, full
tab_bar_position = "top"     # top, bottom, hidden

[appearance.padding]
top = 4
bottom = 4
left = 8
right = 8

[colors]
# Or reference a theme file
foreground = "#e0e0e0"
background = "#1a1a2e"
cursor = "#e94560"
selection_fg = "#ffffff"
selection_bg = "#0f3460"

[colors.normal]
black = "#1a1a2e"
red = "#e94560"
green = "#0cce6b"
yellow = "#ffc857"
blue = "#0f3460"
magenta = "#c77dff"
cyan = "#00b4d8"
white = "#e0e0e0"

[scrollback]
lines = 10000
multiplier = 3.0

[keybindings]
new_tab = "ctrl+shift+t"
close_tab = "ctrl+shift+w"
split_horizontal = "ctrl+shift+h"
split_vertical = "ctrl+shift+v"
navigate_up = "alt+up"
navigate_down = "alt+down"
navigate_left = "alt+left"
navigate_right = "alt+right"
maximize_pane = "ctrl+shift+enter"
next_tab = "ctrl+tab"
prev_tab = "ctrl+shift+tab"
copy = "ctrl+shift+c"
paste = "ctrl+shift+v"
search = "ctrl+shift+f"
command_palette = "ctrl+shift+p"

[suggestions]
enabled = true
history_based = true
man_page_flags = true
fuzzy_match = true
max_suggestions = 8

[profiles.default]
shell = "/bin/zsh"
working_directory = "~"

[profiles.production]
name = "Production"
colors.background = "#2d0000"
auto_switch_rules = ["hostname:prod-*"]

[plugins]
enabled = ["ai-claude", "ai-gemini"]
```

### GUI Settings
- Accessible via hamburger menu (≡) -> Settings
- Or `Ctrl+Shift+,` (like VS Code)
- Full GUI for all config options
- Changes write back to TOML file

## Plugin System (WASM-based)

### Architecture

```
Leuwi Panjang Process
├── Plugin Host (wasmtime runtime)
│   ├── Plugin: AI Claude (WASM module)
│   │   ├── Capabilities: terminal_read, terminal_write, credential_access
│   │   └── Permissions: requires user approval
│   ├── Plugin: AI Gemini (WASM module)
│   └── Plugin: Custom Theme (WASM module)
├── Plugin API (Rust traits exposed to WASM)
│   ├── TerminalAPI - read/write terminal
│   ├── ConfigAPI - read/write config
│   ├── UIAPI - add status bar components, menu items
│   ├── CredentialAPI - encrypted credential storage
│   ├── AuditAPI - log actions
│   └── NotificationAPI - show notifications
└── Plugin Manifest (plugin.toml)
```

### Why WASM?
1. **Sandboxed** - plugins can't access arbitrary system resources
2. **Language-agnostic** - write plugins in Rust, Go, C, AssemblyScript, etc.
3. **Fast** - near-native performance via wasmtime
4. **Portable** - same plugin works on all platforms
5. **Safe** - no arbitrary code execution, explicit capability grants

### Plugin Manifest (`plugin.toml`)
```toml
[plugin]
name = "ai-claude"
version = "0.1.0"
description = "Claude CLI Integration for Leuwi Panjang"
author = "situkangsayur"
license = "MIT"

[capabilities]
terminal_read = true
terminal_write = true
credential_access = true
network_access = false
file_access = false

[permissions]
requires_user_approval = true
audit_all_actions = true
```

## Command Suggestion System

### Architecture
```
User Types Command
        │
        v
┌─────────────────────────┐
│   Suggestion Engine      │
├─────────────────────────┤
│ 1. History Provider      │ <- ~/.zsh_history / ~/.bash_history
│ 2. Path Provider         │ <- filesystem scan
│ 3. Command Provider      │ <- $PATH executables
│ 4. Flag Provider         │ <- man page / --help parsing + built-in specs
│ 5. Git Provider          │ <- git branches, tags, remotes
│ 6. Custom Provider       │ <- plugin-contributed completions
└────────────┬────────────┘
             │
             v
┌─────────────────────────┐
│   Fuzzy Matcher          │ <- skim/fzf algorithm
│   + Ranking Engine       │ <- frequency, recency, context
└────────────┬────────────┘
             │
             v
┌─────────────────────────┐
│   Suggestion UI          │
│   - Inline ghost text    │ <- zsh-autosuggestion style
│   - Dropdown panel       │ <- Fig/Warp style
│   - Flag descriptions    │ <- from man pages
└─────────────────────────┘
```

### Built-in Command Specs
Pre-bundled completion specs for popular tools:
- git, docker, kubectl, npm, yarn, pnpm, cargo, go, pip, brew
- ssh, scp, rsync, curl, wget
- systemctl, journalctl
- aws, gcloud, az
- And more via community contributions

---

# Component 2: nvim-leuwi-panjang (Neovim Config)

See [docs/research/05-nvim-lightweight-ide.md](../research/05-nvim-lightweight-ide.md) for detailed research.

### Summary
- Replace CoC with native LSP (nvim-lspconfig + mason.nvim)
- lazy.nvim for plugin management (lazy loading)
- nvim-cmp for completion
- Telescope for fuzzy finding
- Treesitter for syntax
- Full DAP debugging support
- AI integration (Claude, Gemini, Avante)
- Target: 30-60ms startup, ~30-50 MB memory

---

# Component 3: Plugin System

## Repository: leuwi-panjang-plugins

### Plugin 1: AI Claude Integration (`plugin-claude`)
- Integrates Claude CLI into terminal
- Encrypted credential storage (sudo/admin passwords)
- User approval for all AI actions
- Audit trail (who did what, AI or human)
- Side panel showing AI conversation
- Send terminal selection to Claude

### Plugin 2: AI Gemini Integration (`plugin-gemini`)
- Same architecture as Claude plugin but for Gemini CLI
- Separate plugin = only install what you need

### Plugin 3: AI Ollama Integration (`plugin-ollama`)
- Local AI integration via Ollama
- No network dependency for AI features
- Privacy-first

### Plugin 4: WireGuard Remote Backend (`plugin-wireguard`)
- Connect to AI on remote hosts via WireGuard tunnel
- No direct API key to AI provider needed
- Server manages AI access for multiple clients

### AI Plugin Security Model
```
User Action -> Terminal
       │
       v
┌─────────────────────────┐
│   Permission Gate        │
│   "AI wants to run:      │
│    sudo apt install X"   │
│   [Allow] [Deny] [Always]│
└────────────┬────────────┘
             │ (if allowed)
             v
┌─────────────────────────┐
│   Credential Vault       │
│   (AES-256 encrypted)    │
│   - sudo password        │
│   - SSH keys             │
│   - API tokens           │
└────────────┬────────────┘
             │
             v
┌─────────────────────────┐
│   Audit Trail            │
│   [2026-03-26 14:32:01]  │
│   Actor: AI (Claude)     │
│   Action: sudo apt...    │
│   Result: Success        │
│   Approved by: User      │
└─────────────────────────┘
```

---

# Component 4: Mobile Version (Same Codebase via Makepad)

Mobile is NOT a separate app — it's the **same Rust codebase** compiled for mobile targets.

## Tech Stack (Same as Desktop)

| Layer | Technology | Notes |
|-------|-----------|-------|
| UI Framework | **Makepad** | Same as desktop, compiled for Android/iOS |
| Terminal Core | Same Rust crates | 100% shared code |
| SSH Client | **russh** (pure Rust) | Primary use case on mobile |
| Credential Storage | OS keychain via Rust | Android Keystore / iOS Keychain |
| WireGuard | **boringtun** (embedded) | Connect to desktop server |

### Why Makepad for Mobile (not Flutter)?
- **Single codebase** — zero code duplication between desktop and mobile
- **No Dart VM** — no extra ~30-50MB runtime overhead
- **No FFI bridge** — no flutter_rust_bridge complexity
- **Same rendering** — GPU-rendered on mobile via Metal (iOS) / OpenGL ES (Android)
- **Lower battery usage** — native compiled, no interpreter
- **Smaller binary** — ~5-10MB vs ~15-30MB with Flutter

### Mobile <-> Desktop Connection
Mobile connects to desktop's embedded server (started on-demand):
- Pairing via QR code or 6-digit code (zero-config WireGuard)
- Mobile can access AI, shared sessions, and remote terminals via desktop
- No separate always-running server needed

## Mobile-Specific Features

### SSH Connection Manager
```
┌─────────────────────────────────┐
│ Saved Connections               │
├─────────────────────────────────┤
│ ★ Production Server             │
│   user@prod.example.com:22      │
│   Last connected: 2h ago        │
│                                 │
│ ★ Dev Server                    │
│   dev@192.168.1.100:22          │
│   Last connected: 1d ago        │
│                                 │
│ ★ Home Lab                      │
│   admin@homelab.local:2222      │
│   Last connected: 3d ago        │
│                                 │
│ [+ New Connection]              │
└─────────────────────────────────┘
```

Features:
- Save SSH connections with name, host, port, user, key
- Quick-connect to saved connections (one tap)
- Connection groups/folders
- SSH key management (generate, import, store securely)
- Port forwarding configuration
- Jump host / bastion support
- Mosh support for unstable connections

### Mobile AI Integration
- Connect to AI on a specific network host
- AI plugin communicates via WireGuard tunnel to server
- No direct API key on mobile device
- Server-side AI proxy handles authentication

---

# Component 5: WireGuard Backend Server (Embedded in Desktop Terminal)

The server backend is NOT a separate application. It is **built into the desktop terminal** and started on-demand when mobile clients need to connect.

## How It Works

1. User enables server mode in desktop terminal: `Settings > Server > Enable` or `leuwi-panjang --start-server`
2. Desktop terminal starts embedded WireGuard + API server
3. Desktop shows **QR code** or **6-digit pairing code**
4. Mobile app scans QR / enters code — done. Connected.
5. When no clients connected, server sleeps (near-zero resource usage)

**User NEVER needs to:**
- Manually create WireGuard keys
- Edit WireGuard config files
- Copy/share public keys
- Set up IP addresses or endpoints

Everything is automatic.

## Embedded WireGuard (boringtun)

WireGuard is **embedded directly** in the Leuwi Panjang binary using **boringtun** (Cloudflare's userspace WireGuard implementation in pure Rust). No system-level WireGuard installation needed.

| Aspect | Traditional WireGuard | Leuwi Panjang Embedded |
|--------|----------------------|------------------------|
| Installation | Install wg-tools, kernel module | Nothing — built into terminal |
| Key generation | `wg genkey`, `wg pubkey` manually | Auto-generated on first enable |
| Config | Edit `/etc/wireguard/wg0.conf` | Automatic, zero-touch |
| Pairing | Exchange pubkeys, IPs, endpoints manually | QR code or 6-digit code |
| IP assignment | Manual IP allocation | Auto-assigned from internal range |
| Firewall | Manual iptables/nftables rules | Handled internally |
| Runs as | Root/sudo (kernel module) | Userspace (no root needed) |

### Pairing Flow

```
DESKTOP                                    MOBILE
────────                                   ──────
1. User clicks "Enable Server"
2. Auto-generate WireGuard keypair
3. Auto-assign internal IP (10.lp.0.1)
4. Start embedded WireGuard (boringtun)
5. Start API server on WireGuard interface
6. Show pairing screen:
   ┌────────────────────────┐
   │  Pair Mobile Device    │
   │                        │
   │  ┌──────────────────┐  │
   │  │ ██▀▀██▀▀██▀▀██  │  │
   │  │ ██  ██  ██  ██  │  │    ──►  7. Scan QR code
   │  │ ██▄▄██▄▄██▄▄██  │  │         OR
   │  └──────────────────┘  │
   │                        │
   │  Or enter code:        │
   │  [ 4 8 2 7 1 3 ]      │    ──►  8. Enter 6-digit code
   │                        │
   │  Waiting for device... │
   └────────────────────────┘

                                        9. Mobile auto-generates its keypair
                                       10. Exchange keys via encrypted handshake
                                           (QR/code contains: desktop pubkey +
                                            endpoint + one-time auth token)
                                       11. Mobile connects to desktop WireGuard
                                       12. Tunnel established!

13. Desktop shows:
   "📱 Phone connected (10.lp.0.2)"
   [Manage Devices]
```

### What the QR Code Contains
```json
{
  "v": 1,
  "pubkey": "desktop_wireguard_public_key_base64",
  "endpoint": "192.168.1.50:51820",
  "token": "one_time_auth_token_for_pairing",
  "api_port": 8443
}
```

The mobile app:
1. Reads QR data
2. Generates its own keypair locally
3. Sends its public key to desktop using the one-time token (via temporary HTTP)
4. Desktop registers mobile's public key as authorized peer
5. WireGuard tunnel established
6. All further communication encrypted via WireGuard
7. One-time token is invalidated (cannot be reused)

### Device Management

```toml
# Auto-managed in ~/.config/leuwi-panjang/devices.toml
# User never edits this — managed via GUI

[server]
enabled = true
private_key = "encrypted_in_vault"  # stored in credential vault
listen_port = 51820
internal_ip = "10.lp.0.1/24"

[[devices]]
name = "Hendri's Phone"
public_key = "mobile_pubkey_base64"
internal_ip = "10.lp.0.2"
last_seen = "2026-03-26T14:32:00Z"
permissions = ["ai_access", "terminal_view"]

[[devices]]
name = "Hendri's Laptop 2"
public_key = "laptop2_pubkey_base64"
internal_ip = "10.lp.0.3"
last_seen = "2026-03-25T10:00:00Z"
permissions = ["ai_access", "terminal_view", "terminal_write"]
```

GUI to manage devices:
```
┌─────────────────────────────────────────┐
│ Connected Devices                       │
├─────────────────────────────────────────┤
│ 🟢 Hendri's Phone     10.lp.0.2       │
│    Last: just now                       │
│    Permissions: AI, View                │
│    [Edit] [Revoke]                      │
│                                         │
│ 🔴 Hendri's Laptop 2  10.lp.0.3       │
│    Last: 1 day ago                      │
│    Permissions: AI, View, Write         │
│    [Edit] [Revoke]                      │
│                                         │
│ [+ Pair New Device]                     │
└─────────────────────────────────────────┘
```

### Per-Device Permissions
Each paired device gets granular permissions:

| Permission | Description |
|-----------|-------------|
| `ai_access` | Use AI via desktop (Claude, Gemini, Ollama) |
| `terminal_view` | View desktop terminal sessions (read-only) |
| `terminal_write` | Send input to desktop terminal sessions |
| `file_transfer` | Upload/download files via desktop |
| `ssh_proxy` | Use desktop as SSH jump host |

## Architecture

```
┌────────────┐   Embedded WireGuard   ┌──────────────────────────────┐
│ Mobile App │◄═══ (boringtun) ══════►│  Leuwi Panjang Desktop       │
│ (Client)   │   auto-paired via QR   │  (Pure Rust Terminal)         │
└────────────┘                        │                               │
                                      │  ┌─────────────────────────┐  │
┌────────────┐   Embedded WireGuard   │  │ Embedded Server Backend │  │
│ Laptop 2   │◄═══ (boringtun) ══════►│  │ (started on-demand)     │  │
│ (Client)   │   auto-paired via code │  │                         │  │
└────────────┘                        │  │ Embedded WireGuard      │  │
                                      │  │ (boringtun, userspace)  │  │
                                      │  │                         │  │
                                      │  │ AI Router               │  │
                                      │  │ - Claude CLI (local)    │  │
                                      │  │ - Gemini CLI (local)    │  │
                                      │  │ - Ollama (local)        │  │
                                      │  │                         │  │
                                      │  │ Session Manager         │  │
                                      │  │ - Auth + Audit Trail    │  │
                                      │  │ - Rate Limiting         │  │
                                      │  │ - Device Management     │  │
                                      │  └─────────────────────────┘  │
                                      └──────────────────────────────┘
```

Like OpenClaw/Claude Cowork but:
- Works via terminal (not browser)
- Supports ANY AI platform (Claude, Gemini, Ollama, etc.)
- Clients don't need API keys directly
- WireGuard embedded (boringtun) — no system WireGuard install needed
- **Zero-config pairing** via QR code or 6-digit code
- Auto key generation, auto IP assignment, auto config
- Server is PART of the desktop terminal, not a separate deployment
- Started only when needed, zero overhead when not in use
- Per-device permissions (AI access, terminal view/write, file transfer)

---

# Implementation Roadmap

## Phase 1: Core Terminal (Months 1-3)
1. Rust project setup with wgpu + winit
2. Terminal parsing (VT220 + xterm escape sequences)
3. PTY management (portable-pty)
4. GPU text rendering (glyph atlas + cell grid shader)
5. Basic tab support
6. TOML configuration
7. Chromeless window with rounded corners
8. Basic theming (font, colors, opacity)

## Phase 2: Advanced Features (Months 3-5)
1. Split panes (horizontal + vertical, binary tree model)
2. Search in scrollback (regex)
3. Command suggestion/autocomplete system
4. Shell integration (prompt marks, CWD tracking)
5. Kitty Graphics Protocol + Sixel
6. Hyperlink detection (OSC 8 + auto URL detection)
7. Profile system with auto-switching
8. Status bar with configurable components
9. GUI settings panel

## Phase 3: Plugin System (Months 5-7)
1. WASM plugin runtime (wasmtime)
2. Plugin API design (TerminalAPI, ConfigAPI, CredentialAPI, AuditAPI)
3. Plugin manifest and distribution
4. AI Claude plugin
5. AI Gemini plugin
6. AI Ollama plugin
7. Encrypted credential vault

## Phase 4: Nvim Config (Month 2-3, parallel)
1. lazy.nvim base setup
2. Native LSP + mason for all languages
3. nvim-cmp completion
4. Telescope fuzzy finding
5. Treesitter syntax + textobjects
6. DAP debugging
7. Git integration
8. AI integration (Claude, Gemini)
9. Documentation

## Phase 5: Mobile App (Months 7-10)
1. Flutter project setup
2. Rust FFI bridge (flutter_rust_bridge)
3. Terminal rendering on mobile
4. SSH client (Rust)
5. SSH connection manager UI
6. Mobile-optimized keyboard
7. AI integration via WireGuard
8. Android release
9. iOS release

## Phase 6: WireGuard Server (Months 8-10)
1. Rust server with WireGuard integration
2. AI router (Claude, Gemini, Ollama backends)
3. Authentication and session management
4. Audit trail system
5. Rate limiting
6. Multi-client support

## Phase 7: Polish & Release (Months 10-12)
1. Performance optimization
2. Cross-platform testing (Linux, macOS, Windows)
3. Documentation completion
4. Community plugin guidelines
5. Package distribution (cargo install, brew, apt, flatpak)
6. Mobile app store releases

---

# Competitive Advantage Summary

| Feature | GNOME | Kitty | iTerm2 | Alacritty | WezTerm | **Leuwi Panjang** |
|---------|-------|-------|--------|-----------|---------|-------------------|
| GPU Rendering | No | OpenGL | Metal | OpenGL | OpenGL | **wgpu (Vulkan/Metal/DX12)** |
| Split Panes | No | Yes | Yes | No | Yes | **Yes** |
| Chromeless | No | Yes | No | Yes | Partial | **Yes (default)** |
| Plugin System | No | Kittens | Python | No | Lua | **WASM (sandboxed)** |
| SSH Works | Yes | Broken | macOS only | Yes | Yes | **Yes (standard)** |
| Command Suggestions | No | No | No | No | No | **Yes (built-in)** |
| AI Integration | No | No | No | No | No | **Yes (plugin)** |
| Mobile App | No | No | No | No | No | **Yes** |
| Config Format | dconf | .conf | plist | TOML | Lua | **TOML + GUI** |
| Cross-Platform | Linux | Linux/Mac | Mac | All | All | **All + Mobile** |
| Audit Trail | No | No | No | No | No | **Yes** |
| Credential Vault | No | No | Keychain | No | No | **Yes (encrypted)** |
| Rounded Corners | No | No | No | No | No | **Yes** |
| Language | C | C/Python | ObjC | Rust | Rust | **Rust** |
