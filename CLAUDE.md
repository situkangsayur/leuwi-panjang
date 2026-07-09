# Leuwi Panjang Terminal — Developer Guide

## Build
```bash
cargo build --release  # release binary at target/release/leuwi-panjang
cargo test             # 111 automated tests
```

## Architecture
- `src/lib.rs` (~3230 lines) — the whole app + the 111 tests. Built as an Android
  **cdylib** by cargo-makepad and as an `rlib` for the desktop bin. Holds `app_main!(App)`.
  - `Theme` — TOML theme loading from ~/.config/leuwi-panjang/themes/
  - `Config` — TOML config from ~/.config/leuwi-panjang/config.toml (incl. `ssh_*` fields)
  - `Cell` — single terminal cell (char + fg + bg + bold + underline)
  - `TermGrid` — terminal grid with VT parser, alt screen, scroll regions, search
  - `TermTab` — tab with a backend (local PTY **or** SSH) + grid + split
  - `TermView` — Makepad custom widget for rendering (search highlights, focus indicator)
  - `App` — Makepad application; touch multi-tab bar (tap tab/×/+), each Android tab is
    its own nvgpu tmux session (`main`, `main-2`, …) via `Config::ssh_profile_for_tab`
- `src/main.rs` — thin desktop entry: `ssh-smoke` subcommand + `leuwi_panjang::app_main()`
- `src/ssh.rs` — russh SSH backend (Android default; desktop SSH tabs); tmux attach-or-create; TOFU host-key pinning
- `src/bin/ssh-smoke.rs` — headless SSH self-test (compiles only ssh.rs, no makepad)

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

## Android build (v0.1.0-dev.16 — real signed APK with SSH)
```bash
./install/build-apk.sh          # -> target/makepad-android-apk/leuwi-panjang/apk/leuwipanjang.apk
```
The script wraps cargo-makepad and does the manual finish (zipalign + apksigner). It needs,
one-time: cargo-makepad, `cargo makepad android install-toolchain`, a **full NDK**
(symlinked at `$SDK/NDK` — the bundled one is stripped: no C headers / libunwind / llvm-ar,
which `ring` needs), and a **JDK ≤ 17** at `$SDK/openjdk` (d8 rejects Java 21 bytecode).
- Use **`-p leuwi-panjang`** (not `--package …`, which cargo-makepad mis-parses).
- App identity via makepad flags `--package-name=… --app-label=…` (metadata blocks ignored).
- Desktop deps (`portable-pty`, `arboard`) gated `cfg(not(target_os = "android"))`.
- `min_sdk_version = 29` (Makepad needs `libamidi`); `lto = false` (`prefer-dynamic`).
- **Android tab = SSH to nvgpu** (`src/ssh.rs`): `tmux new -A -s <session>`. russh uses the
  **`ring`** crypto backend (not aws-lc-rs) so it cross-compiles without nasm/clang.
- Full details incl. WireGuard (milestone C): `docs/mobile/02-android-build.md`.

## Verify SSH backend
- Headless self-test: `cargo run --bin ssh-smoke -- list|attach <session>`
  (env `LEUWI_SSH_HOST/PORT/USER/KEY/SESSION/STARTUP/KNOWN_HOSTS`). Desktop GUI SSH: `LEUWI_SSH=1 cargo run`.

## Build caveat (this dev box)
nvda11-gpu is headless: the makepad GUI bin/tests **cannot link** here (`-lasound`/`-lpulse`
missing) — and since the lib is now a cdylib, `--bin ssh-smoke` drags it in too. Use
`cargo check --lib --tests` to type-check. Run `ssh-smoke` from its pre-built binary or on a
box with the audio dev libs. The APK builds fine here (uses the Android NDK, not desktop libs).
