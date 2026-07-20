# Leuwi Panjang — Android APK

SSH + tmux client for the nvgpu GPU server (arm64-v8a). Each terminal tab is its
own tmux session, with an on-screen modifier bar, vertical tabs and drag-to-scroll.

## Download (v0.1.0-dev.22 — release build, minimal permissions)

- **[⬇ leuwipanjang_v0.1.0-dev.22.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang_v0.1.0-dev.22.apk)** (versioned)
- **[⬇ leuwipanjang.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang.apk)** (always latest)

~55 MB · `com.situkangsayur.leuwipanjang` · arm64-v8a · minSdk 29

## What's new in dev.22

- **Command history at the prompt** — arrow up/down walks previous commands. They
  previously typed a literal `[A`, because the REPL never parsed the escape sequence.
- **Generated keys now go to app-private storage** (`/data/user/0/<pkg>/ssh/`). The
  fallback path used to point at `/sdcard/Android/data/<pkg>/files/`, where any app
  holding storage access could read a private key.

## Carried over from dev.21

- **Full-screen config form with tabs** — *Perintah* / *SSH Key* / *Keymap*.
  Multiple connection profiles, each with its own host, port, user and either a
  password or a named SSH key. The connect and list words are free-form, so a
  profile can answer to `nvgpu-s` / `nvgpu-ls` or anything else you pick.
- **SSH key management** — generate a named ed25519 keypair on the phone, or
  paste an existing private + public pair. No more installing keys by hand.
- **Release build, minimal permissions.** Previous builds shipped debuggable and
  requested camera, location, media, biometric, bluetooth and full package
  enumeration — all cargo-makepad template defaults, none of them needed. Banking
  apps with anti-fraud SDKs (OCBC) refused to run alongside it. This build
  declares **INTERNET and ACCESS_NETWORK_STATE only**, is not debuggable, and is
  signed with a real release key.
- Version is finally stamped in the manifest (was showing "Versi: null").

> **Coming from dev.20 or earlier?** The signing key changed in dev.21 (a real
> release key instead of the Android debug key), so this build cannot update such
> an install — uninstall the old one first. dev.21 → dev.22 updates normally.

## Install & run

1. Open the download link on the phone and install (allow "unknown sources").
2. Bring up **WireGuard** (official app) so `10.100.21.22` is reachable.
3. Open the burger `≡` → **SSH Key** → generate a key, and add its public half to
   `~/.ssh/authorized_keys` on the server.
4. Under **Perintah**, set the host/port/user and pick that key.
5. At the `leuwi>` prompt: `<nama>-ls` lists sessions, `<nama>-s <sesi>` attaches one.
