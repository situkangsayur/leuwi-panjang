# Rust Terminal Ecosystem - Research

## Existing Rust-Based Terminals

### Alacritty

| Aspect | Details |
|--------|---------|
| Language | Rust |
| Rendering | OpenGL (glutin) |
| Config | TOML (`alacritty.toml`) |
| Features | Minimal - no tabs, no splits, no plugins |
| Strengths | Fastest startup (<50ms), lowest memory (~20MB), pure performance |
| Weaknesses | No tabs, no splits, no extensions, no images |
| Architecture | Single window, OpenGL renderer, vte parser |
| Code Quality | Excellent Rust code, well-structured |
| Key Library | `alacritty_terminal` crate (terminal emulation as a library) |

**Lesson for Leuwi Panjang**: Alacritty proves Rust + GPU = fastest terminal. Their `alacritty_terminal` crate could be studied for VT parser design.

### WezTerm

| Aspect | Details |
|--------|---------|
| Language | Rust |
| Rendering | OpenGL (wgpu planned, was using own renderer) |
| Config | Lua scripting (full programming language) |
| Features | Rich - tabs, splits, multiplexer, images (all 3 protocols), ligatures |
| Strengths | Most feature-rich Rust terminal, Lua config is powerful, SSH domains |
| Weaknesses | Slower than Alacritty, complex codebase, high memory |
| Architecture | Multiplexer architecture (local + SSH + serial domains) |
| Images | Kitty protocol + Sixel + iTerm2 protocol (all three!) |

**Lesson for Leuwi Panjang**: WezTerm shows you can build a full-featured terminal in Rust. Their multi-protocol image support is worth studying.

### Rio Terminal

| Aspect | Details |
|--------|---------|
| Language | Rust |
| Rendering | wgpu (Vulkan/Metal/DX12) |
| Config | TOML |
| Features | Tabs, themes, web-inspired navigation |
| Status | Newer project, actively developed |
| Key Innovation | Uses wgpu (same as our plan!) |

**Lesson for Leuwi Panjang**: Rio proves wgpu works for terminal rendering. Study their wgpu integration.

## Key Rust Libraries

### Rendering & Windowing

| Library | Purpose | Notes |
|---------|---------|-------|
| **wgpu** | GPU rendering (Vulkan/Metal/DX12/OpenGL) | Best choice - cross-API, future-proof, Rust-native |
| **winit** | Window management | Cross-platform windowing, handles events |
| **glutin** | OpenGL context | Used by Alacritty, less flexible than wgpu |
| **softbuffer** | Software rendering fallback | For when GPU is unavailable |

**Decision: wgpu + winit** - gives us Vulkan on Linux, Metal on macOS, DX12 on Windows, with OpenGL fallback.

### Terminal Emulation

| Library | Purpose | Notes |
|---------|---------|-------|
| **vte** | VT parser crate | Basic parser, used by Alacritty's terminal |
| **alacritty_terminal** | Complete terminal emulation | Full screen buffer, parsing, PTY; could be used as library |
| **termwiz** | WezTerm's terminal lib | Feature-rich but tightly coupled to WezTerm |

**Decision**: Custom parser inspired by vte/alacritty_terminal, optimized for our needs (image protocols, suggestion hooks).

### PTY Management

| Library | Purpose | Notes |
|---------|---------|-------|
| **portable-pty** | Cross-platform PTY | Works on Linux, macOS, Windows |
| **pty-process** | PTY process management | Simpler API |
| **nix** | Unix system calls | Low-level, Linux/macOS only |

**Decision: portable-pty** - cross-platform, well-tested (used by WezTerm).

### Text Shaping & Fonts

| Library | Purpose | Notes |
|---------|---------|-------|
| **rustybuzz** | HarfBuzz in Rust | Text shaping, ligatures, complex scripts |
| **swash** | Font introspection + rasterization | Color emoji, variable fonts, fast |
| **fontdue** | Font rasterization | Fastest glyph rasterizer, minimal |
| **cosmic-text** | Text layout | Full text layout engine (by System76) |

**Decision: rustybuzz + swash** - rustybuzz for shaping (ligatures), swash for fast glyph rasterization.

### Config & Serialization

| Library | Purpose | Notes |
|---------|---------|-------|
| **serde** | Serialization framework | Standard Rust serialization |
| **toml** | TOML parser | With serde integration |
| **notify** | File watcher | For config hot-reload |

### Plugin System

| Library | Purpose | Notes |
|---------|---------|-------|
| **wasmtime** | WASM runtime | Production-quality, by Bytecode Alliance |
| **wasmer** | WASM runtime | Alternative to wasmtime |
| **wasm-component-model** | WASM component model | For plugin interfaces |
| **extism** | Plugin framework | Built on wasmtime, simplifies plugin dev |

**Decision: wasmtime** - most mature, best Rust integration, production-proven.

### Cryptography

| Library | Purpose | Notes |
|---------|---------|-------|
| **ring** | Crypto primitives | Fast, audited |
| **aes-gcm** | AES-256-GCM | For credential vault encryption |
| **argon2** | Key derivation | For master password hashing |
| **secrecy** | Secret management | Zeroize secrets from memory |

### SSH (for mobile/server)

| Library | Purpose | Notes |
|---------|---------|-------|
| **russh** (formerly thrussh) | SSH client/server | Pure Rust SSH implementation |
| **ssh2** | libssh2 bindings | C library bindings, more mature |

**Decision: russh** - pure Rust, no C dependencies, better for mobile.

### WireGuard (Embedded)

| Library | Purpose | Notes |
|---------|---------|-------|
| **boringtun** | Userspace WireGuard in Rust | By Cloudflare, no kernel module needed, no root |
| **x25519-dalek** | Key exchange | For WireGuard keypair generation |
| **qrcode** | QR code generation | For zero-config device pairing |

**boringtun** allows embedding WireGuard directly into the terminal binary. No system WireGuard install needed, no manual key/config management.

### IPC & Networking

| Library | Purpose | Notes |
|---------|---------|-------|
| **tokio** | Async runtime | Standard Rust async |
| **axum** | HTTP server | For embedded API server (device mgmt, AI routing) |
| **serde_json** | JSON | Plugin communication |
| **rmp-serde** | MessagePack | Efficient binary serialization |

## Flutter + Rust Integration

### flutter_rust_bridge
- Generates Flutter/Dart bindings from Rust code automatically
- Supports async, streams, complex types
- Used in production by several apps
- Process: Rust code -> codegen -> Dart FFI bindings

### Architecture for Mobile
```
Flutter (Dart)
├── UI Layer (Material/Cupertino widgets)
├── Terminal View (Custom painter)
└── Bridge Layer (flutter_rust_bridge generated)
    │
    │ FFI
    │
Rust Library (compiled to .so/.dylib)
├── VT Parser (shared with desktop)
├── SSH Client (russh)
├── Plugin Host (wasmtime)
├── Credential Vault (aes-gcm)
└── Config Manager (toml/serde)
```

### Build Process
1. Rust code compiles to native library per platform
2. flutter_rust_bridge generates Dart bindings
3. Flutter app includes native library
4. `cargo ndk` for Android, `cargo lipo` for iOS

## Rust on Mobile

### Android
- Rust compiles to Android NDK targets (aarch64, armv7, x86_64)
- Use `cargo-ndk` for building
- JNI bridge or flutter_rust_bridge for communication
- Full access to native APIs

### iOS
- Rust compiles to iOS targets (aarch64-apple-ios)
- Use `cargo-lipo` for universal binaries
- Swift/ObjC bridge or flutter_rust_bridge
- App Store compatible

## WASM Plugin System Design

### Component Model
```rust
// Plugin interface (WIT - WASM Interface Types)
interface terminal-plugin {
    // Types
    record screen-cell {
        character: string,
        fg-color: u32,
        bg-color: u32,
        bold: bool,
        italic: bool,
    }

    // Functions the plugin can call (host provides)
    read-screen: func() -> list<list<screen-cell>>
    write-text: func(text: string) -> result<_, permission-error>
    get-cwd: func() -> string
    store-credential: func(key: string, value: string) -> result<_, error>
    retrieve-credential: func(key: string) -> result<string, permission-error>
    log-audit: func(action: string, details: string)
    show-notification: func(title: string, body: string)
    add-status-component: func(id: string, content: string)

    // Functions the host calls (plugin provides)
    on-init: func() -> result<_, error>
    on-terminal-output: func(text: string)
    on-command-complete: func(command: string, exit-code: s32)
    on-keypress: func(key: string) -> bool  // true = consumed
}
```

### Plugin SDK (Rust)
```rust
// leuwi-panjang-plugin-sdk crate
use leuwi_panjang_plugin_sdk::*;

#[plugin]
struct MyPlugin;

impl TerminalPlugin for MyPlugin {
    fn on_init(&mut self, api: &PluginAPI) -> Result<()> {
        api.ui().add_status_component("my-status", "Ready");
        Ok(())
    }

    fn on_terminal_output(&mut self, api: &PluginAPI, text: &str) {
        // React to terminal output
    }

    fn on_command_complete(&mut self, api: &PluginAPI, cmd: &str, exit_code: i32) {
        if exit_code != 0 {
            api.ui().notify("Command Failed", &format!("{cmd} exited with {exit_code}"));
        }
    }
}
```

## Proposed Cargo Workspace Structure

**Single codebase for desktop AND mobile — all Rust, Makepad UI.**

```
leuwi-panjang/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── leuwi-terminal/           # Core: VT parser, screen buffer
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-renderer/           # Terminal grid renderer (Makepad shaders)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-pty/                # PTY management (desktop)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-ssh/                # SSH client (russh, mobile primary)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-ui/                 # Makepad UI: tabs, splits, status bar, settings
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-config/             # Config, profiles, themes
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-suggestions/        # Command suggestion engine
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-plugin-host/        # WASM plugin runtime (wasmtime)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-plugin-sdk/         # Plugin SDK (compiles to WASM)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-crypto/             # Credential vault (AES-256-GCM)
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-search/             # Scrollback search, tab/pane search
│   │   ├── Cargo.toml
│   │   └── src/
│   ├── leuwi-shell-integration/  # Shell integration protocol
│   │   ├── Cargo.toml
│   │   └── src/
│   └── leuwi-wireguard/          # Embedded WireGuard (boringtun)
│       ├── Cargo.toml
│       └── src/
├── src/
│   └── main.rs                   # Application entry point (Makepad app)
├── themes/                       # Built-in themes (TOML)
├── completions/                  # Command completion specs
└── docs/                         # Documentation
```

No separate `mobile/` directory — mobile builds from the same `src/` using Makepad:
```bash
# Desktop
cargo run                                        # dev
cargo build --release                            # release

# Mobile (same code)
cargo makepad android build --release            # Android APK
cargo makepad ios build --release                # iOS IPA
```

## Dependencies Summary (Cargo.toml)

```toml
[workspace]
members = ["crates/*"]

[workspace.dependencies]
# UI Framework (desktop + mobile from single codebase)
makepad-widgets = { git = "https://github.com/makepad/makepad" }

# Terminal
vte = "0.13"
portable-pty = "0.8"

# Text
rustybuzz = "0.18"
swash = "0.1"

# Config
serde = { version = "1", features = ["derive"] }
toml = "0.8"
notify = "6"

# Plugin
wasmtime = "22"

# Crypto
aes-gcm = "0.10"
argon2 = "0.5"
secrecy = "0.8"

# Async
tokio = { version = "1", features = ["full"] }
axum = "0.7"

# WireGuard (embedded)
boringtun = "0.6"
x25519-dalek = "2"
qrcode = "0.14"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

*Note: Version numbers are approximate and should be verified against crates.io at build time.*
