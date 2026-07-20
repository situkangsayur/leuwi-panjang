# Leuwi Panjang — Android APK

SSH + tmux client for a remote dev box (arm64-v8a). Each terminal tab is its own
tmux session, with an on-screen modifier bar, vertical tabs and drag-to-scroll.

## Download — v0.1.1

- **[⬇ leuwipanjang_v0.1.1.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang_v0.1.1.apk)** (versioned)
- **[⬇ leuwipanjang.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang.apk)** (always latest)

~55 MB · `com.situkangsayur.leuwipanjang` · arm64-v8a · minSdk 29

## What's new in 0.1.1

- **Notifications for sessions waiting on a reply.** A tab that rings the terminal
  bell while it is in the background starts blinking (`*` in the tab list, `[!] tab N`
  in the status bar) and raises a phone notification. Tapping the notification opens
  that tab. Built for AI CLIs, which sit waiting for an answer while you are elsewhere.
- **Terminal width now updates when the tab sidebar is collapsed.** It used to keep the
  old column count until an SSH session pushed its own size, so text stayed narrow at
  the local prompt.
- **Command history** at the `leuwi>` prompt with arrow up/down.

## Carried over from 0.1.0

- **Full-screen config with tabs** — *Perintah* / *SSH Key* / *Keymap*. Multiple
  connection profiles, each with its own host, port, user and either a password or a
  named SSH key. Connect and list words are free-form.
- **SSH key management** — generate a keypair on the phone, paste an existing one, or
  drop `id_ed25519` + `id_ed25519.pub` into the import folder and load them. Private
  keys are stored app-private, never on shared storage.
- **Load config from files** — `commands.toml` plus the key files in the import folder.
  Press **Ekspor** first: the app creates the folder and writes a template you can edit.
  The exact path is shown on screen.
- **Minimal permissions** — INTERNET, ACCESS_NETWORK_STATE and POST_NOTIFICATIONS only;
  not debuggable; signed with a real release key. Earlier builds inherited camera,
  location, media, biometric, bluetooth and full package enumeration from the UI
  framework's manifest template, which made banking anti-fraud SDKs refuse to run.

## Install & run

1. Open the download link on the phone and install (allow "unknown sources").
2. Bring up your VPN if the host is only reachable through one.
3. Open the burger `≡` → **SSH Key** → generate a key, and add its public half to
   `~/.ssh/authorized_keys` on the server.
4. Under **Perintah**, set host / port / user and pick that key.
5. At the `leuwi>` prompt: `<nama>-ls` lists sessions, `<nama>-s <sesi>` attaches one.

> **Coming from a build before dev.21?** The signing key changed (a real release key
> instead of the Android debug key), so this cannot update such an install —
> uninstall the old one first.
