# Leuwi Panjang Terminal — Developer Guide

## Build
```bash
cargo build --release  # release binary at target/release/leuwi-panjang
cargo test             # 111 automated tests
```

## Architecture
Single file: `src/main.rs` (~3100 lines)
- `Theme` — TOML theme loading from ~/.config/leuwi-panjang/themes/
- `Config` — TOML config from ~/.config/leuwi-panjang/config.toml
- `Cell` — single terminal cell (char + fg + bg + bold + underline)
- `TermGrid` — terminal grid with VT parser, alt screen, scroll regions, search
- `TermTab` — tab with PTY + grid + split (vertical/horizontal)
- `TermView` — Makepad custom widget for rendering (search highlights, focus indicator)
- `App` — Makepad application (tabs, splits, search, status bar, themes)

## Key decisions
- **Makepad** UI framework (not GTK) — chromeless, GPU-rendered
- **custom_window_chrome = true** patched in Makepad for no title bar
- **portable-pty** for PTY management
- **TextInput** for printable chars, **KeyDown** for special keys only
- Cell size configurable via config.toml (cell_width, cell_height)
- **SplitDir** enum: Vertical (side-by-side) and Horizontal (top-bottom)
- **Theme system**: TOML files in themes/ dir, 16 ANSI colors + UI colors

## Testing
```bash
cargo test  # runs all 111 tests
```
Tests cover: grid, VT parser, SGR colors, alt screen, scroll regions, selection, URLs, config, theme, search, split direction.

## Android build
Same `src/main.rs` cross-compiles to a signed APK (milestone A done, v0.1.0-dev.15):
```bash
cargo install cargo-makepad
cargo makepad android install-toolchain
cargo makepad android build --package leuwi-panjang
# APK: target/android/makepad-android-apk/leuwi_panjang/apk/leuwipanjang.apk
```
- Desktop deps (`portable-pty`, `arboard`) are gated `cfg(not(target_os = "android"))`.
- `min_sdk_version = 29` (Makepad needs `libamidi`); `lto = false` (Makepad uses `prefer-dynamic`).
- Android tab uses a **local-echo backend** placeholder; SSH backend (`russh` → nvgpu) is milestone B.
- Full details: `docs/mobile/02-android-build.md`.
