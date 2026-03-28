# Terminal Emulator Comparison Matrix

## Feature Comparison

| Feature | GNOME Terminal | Kitty | iTerm2 | Alacritty | WezTerm | **Leuwi Panjang** |
|---------|:-:|:-:|:-:|:-:|:-:|:-:|
| **Rendering** | CPU (Cairo) | GPU (OpenGL) | GPU (Metal) | GPU (OpenGL) | GPU (OpenGL) | **GPU (Makepad)** |
| **Language** | C | C/Python/Go | ObjC/Swift | Rust | Rust | **Rust** |
| **Tabs** | Yes | Yes | Yes | No | Yes | **Yes** |
| **Split Panes** | No | Yes | Yes (best) | No | Yes | **Yes** |
| **Chromeless** | No | Yes | No | Yes | Partial | **Yes (default)** |
| **Rounded Corners** | No | No | No | No | No | **Yes** |
| **Plugin System** | None | Kittens (Python) | Python API | None | Lua scripting | **WASM (sandboxed)** |
| **SSH Works** | Yes | Broken* | macOS only | Yes | Yes | **Yes (standard)** |
| **Command Suggestions** | No | No | No | No | No | **Yes (built-in)** |
| **AI Integration** | No | No | No | No | No | **Yes (plugins)** |
| **Tab/Pane Search** | No | No | No | No | No | **Yes** |
| **Inline Images** | Sixel | Kitty Protocol | iTerm Protocol | No | All 3 | **All 3** |
| **Ligatures** | No | Yes | Yes | No | Yes | **Yes** |
| **Shell Integration** | Basic | Yes | Yes (best) | No | Basic | **Yes** |
| **Profile System** | Yes | Basic | Yes (best) | Basic | Yes | **Yes (auto-switch)** |
| **Credential Vault** | No | No | Keychain (macOS) | No | No | **Yes (AES-256)** |
| **Audit Trail** | No | No | No | No | No | **Yes** |
| **Config Format** | dconf | .conf | plist/JSON | TOML | Lua/TOML | **TOML + GUI** |
| **Mobile App** | No | No | No | No | No | **Yes (Flutter+Rust)** |
| **Status Bar** | No | No | Yes | No | Basic | **Yes (configurable)** |
| **Instant Replay** | No | No | Yes | No | No | **Planned** |
| **tmux Integration** | No | No | Yes (-CC) | No | Mux domains | **Planned** |

*Kitty SSH: requires `kitty +kitten ssh` due to custom TERM, otherwise breaks on remote servers.

## Platform Support

| Platform | GNOME Terminal | Kitty | iTerm2 | Alacritty | WezTerm | **Leuwi Panjang** |
|----------|:-:|:-:|:-:|:-:|:-:|:-:|
| Linux | Yes | Yes | No | Yes | Yes | **Yes** |
| macOS | No | Yes | Yes | Yes | Yes | **Yes** |
| Windows | No | No | No | Yes | Yes | **Yes** |
| Android | No | No | No | No | No | **Yes** |
| iOS | No | No | No | No | No | **Yes** |

## Performance Comparison (Estimated)

| Metric | GNOME | Kitty | iTerm2 | Alacritty | WezTerm | **Leuwi Panjang (target)** |
|--------|-------|-------|--------|-----------|---------|---------------------------|
| Startup Time | 300-500ms | <200ms | 500-1500ms | <50ms | <200ms | **<100ms** |
| Base Memory | 30-50MB | 30-60MB | 80-150MB | 20-40MB | 40-80MB | **<50MB** |
| Throughput | 50-100 MB/s | 500+ MB/s | 80-120 MB/s | 500+ MB/s | 200+ MB/s | **500+ MB/s** |
| Input Latency | ~10ms | 2-5ms | ~8ms | 2-5ms | ~5ms | **<5ms** |

## Leuwi Panjang Design Decisions (Informed by Research)

### From GNOME Terminal (Learn)
- VTE-level terminal emulation quality and correctness
- Profile system with named configurations
- Standard SSH support (TERM=xterm-256color)

### From GNOME Terminal (Avoid)
- Large header bar eating screen space
- No split panes
- No plugin system
- CPU-only rendering
- dconf-only config

### From Kitty (Learn)
- GPU-accelerated rendering
- Kitty Keyboard Protocol
- Kitten-style plugin architecture
- Multiple layout engines for splits
- Chromeless window mode
- Shell integration

### From Kitty (Avoid)
- Custom TERM breaking SSH
- Refusing Sixel support
- OpenGL-only (wgpu gives Vulkan/Metal/DX12)
- Requiring special SSH command

### From iTerm2 (Learn)
- Best-in-class split pane UX (Cmd+D simple shortcuts)
- Automatic Profile Switching (by host/path)
- Shell Integration working over SSH
- Trigger system (regex-based automation)
- Instant Replay concept
- Password Manager / credential storage
- Status bar with configurable components
- Search + scrollback autocomplete

### From iTerm2 (Avoid)
- Single platform (macOS only)
- High memory usage
- Slow startup
- Overwhelming settings UI
- plist config format
- Python API with high barrier to entry

### From Alacritty (Learn)
- Rust + GPU = fastest terminal
- TOML config (human-readable, version-controllable)
- Minimal resource usage
- Fast startup

### From WezTerm (Learn)
- Rust implementation quality
- Multiple image protocol support
- Multiplexer domains concept
- Good cross-platform support
