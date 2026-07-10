# Leuwi Panjang — Android APK

SSH + tmux client for the nvgpu GPU server (arm64-v8a). Connects and runs
`tmux new -A -s <session>` like `nvgpu-s`, with touch multi-tab (each tab = its
own tmux session), on-screen keyboard, and drag-to-scroll.

## Download (v0.1.0-dev.20 — startup crash fixed, now launches & renders)

- **[⬇ leuwipanjang_v0.1.0-dev.20.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang_v0.1.0-dev.20.apk)** (versioned)
- **[⬇ leuwipanjang.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang.apk)** (always latest)

~85 MB · `com.situkangsayur.leuwipanjang` · arm64-v8a

## What's fixed in dev.20
- **App now opens & renders** (was crashing on launch). Root cause: cargo-makepad
  version mismatch (`onAndroidParams` arg count) + a strict-GLES shader `#version`
  bug — both fixed. Verified running in an Android emulator.

## Install & run
1. Open the download link on the phone, install (allow "unknown sources").
2. Bring up the **WireGuard** tunnel (official app) so `10.100.21.22` is reachable.
3. Open Leuwi Panjang — you should see the terminal UI and a connect banner.

> Known next step: the app still needs your SSH private key on the device to
> authenticate (currently shows `load key .ssh/id_rsa: No such file`). Key import
> is coming in the next build.
