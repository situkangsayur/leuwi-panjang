# Android Build

How Leuwi Panjang cross-compiles to an Android APK from the **same** `src/main.rs`
used on the desktop. Introduced in **v0.1.0-dev.15** (2026-07-08).

## Goal

Run the terminal on a phone so it can reach the **nvgpu GPU server over SSH**, the
same way `nvgpu-s <session>` (SSH + tmux) works from the kitty desktop terminal.

## Toolchain

```bash
# One-time: Android SDK + NDK, plus cargo-makepad
cargo install cargo-makepad
cargo makepad android install-toolchain   # downloads SDK/NDK/OpenJDK if missing

# Build a signed debug APK (arm64)
cargo makepad android build --package leuwi-panjang

# Output:
#   target/android/makepad-android-apk/leuwi_panjang/apk/leuwipanjang.apk

# Install on a connected device / emulator
cargo makepad android run --package leuwi-panjang
```

## Build decisions (Cargo.toml)

- **`min_sdk_version = 29`** — Makepad's platform layer links `AMidi` (`libamidi`),
  which only exists in the NDK sysroot from **API 29** onward. API 26 fails with
  `unable to find -lamidi`.
- **LTO disabled** (`lto = false`) — Makepad links the Android app crate with
  `prefer-dynamic`, which is incompatible with LTO
  (`cannot prefer dynamic linking when performing LTO`).
- **Desktop-only deps are gated** behind `cfg(not(target_os = "android"))`:
  - `portable-pty` pulls in `termios`, which does not build for Android.
  - `arboard` (clipboard) has no Android backend.
- **Package metadata**: `identifier = com.situkangsayur.leuwipanjang`,
  `product_name = "Leuwi Panjang"`, `version_code = 1`.

## Milestone A — pipeline de-risk (DONE)

Prove the full cross-compile → package → signed-APK pipeline works before adding the
SSH stack. On Android there is no local shell/PTY, so the terminal tab runs a
**local-echo backend** (`cfg(target_os = "android")`) that echoes typed input and
shows a welcome banner:

```
  Leuwi Panjang  Android
  Terminal UI aktif. Backend SSH (nvgpu) menyusul.
```

All rendering, tabs, splits, themes, and search work — only the byte source differs
from desktop. Clipboard and resize are branched: desktop uses arboard/PTY; Android
will notify the SSH backend of size changes.

## Milestone B — SSH backend (NEXT)

Replace the local-echo backend with a real SSH connection to nvgpu using **`russh`**
(pure-Rust SSH; `portable-pty` cannot run on Android). Dependencies are already staged
(commented) in `Cargo.toml`:

```toml
# [target.'cfg(target_os = "android")'.dependencies]
# russh = "0.45"
# russh-keys = "0.45"
# tokio = { version = "1", features = ["rt","rt-multi-thread","io-util","net","sync","macros","time"] }
```

### Embedded nvgpu connection profiles

The APK ships pre-saved connection profiles mirroring the desktop `nvgpu` family so the
user gets one-tap access:

| Profile   | Host / VPN IP    | Port | User   | Auth        | Startup                  |
|-----------|------------------|------|--------|-------------|--------------------------|
| `nvgpu-s` | `10.100.21.22`   | 1313 | hendri | `id_rsa`    | `tmux new -A -s <session>` (default `main`) |

Reaching `10.100.21.22` requires the WireGuard tunnel; on mobile this is the embedded
boringtun client (Phase 5.3 / desktop pairing).
