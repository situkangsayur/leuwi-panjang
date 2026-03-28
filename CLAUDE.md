# Leuwi Panjang Terminal — Developer Guide

## Build
```bash
cargo build --release  # release binary at target/release/leuwi-panjang
cargo test             # 70+ automated tests
```

## Architecture
Single file: `src/main.rs` (~1800 lines)
- `Config` — TOML config from ~/.config/leuwi-panjang/config.toml
- `Cell` — single terminal cell (char + fg + bg + bold)
- `TermGrid` — terminal grid with VT parser, alt screen, scroll regions
- `TermTab` — tab with PTY + grid
- `TermView` — Makepad custom widget for rendering
- `App` — Makepad application (tabs, splits, events)

## Key decisions
- **Makepad** UI framework (not GTK) — chromeless, GPU-rendered
- **custom_window_chrome = true** patched in Makepad for no title bar
- **portable-pty** for PTY management
- **TextInput** for printable chars, **KeyDown** for special keys only
- Cell size configurable via config.toml (cell_width, cell_height)

## Testing
```bash
cargo test  # runs all 70+ tests
```
Tests cover: grid, VT parser, SGR colors, alt screen, scroll regions, selection, URLs, config.
