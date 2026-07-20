# Leuwi Panjang — Android APK

SSH + tmux client for a remote dev box (arm64-v8a). Each terminal tab is its own
tmux session, with an on-screen modifier bar, vertical tabs and drag-to-scroll.

## Download — v0.1.0

- **[⬇ leuwipanjang_v0.1.0.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang_v0.1.0.apk)** (versioned)
- **[⬇ leuwipanjang.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang.apk)** (always latest)

~55 MB · `com.situkangsayur.leuwipanjang` · arm64-v8a · minSdk 29

## Highlights

- **Full-screen config with tabs** — *Perintah* / *SSH Key* / *Keymap*. Multiple
  connection profiles, each with its own host, port, user and either a password or
  a named SSH key. The connect and list words are free-form, so a profile can answer
  to `nvgpu-s` / `nvgpu-ls` or anything else you pick.
- **SSH key management on the phone** — generate a named ed25519 keypair, paste an
  existing one, or drop `id_ed25519` + `id_ed25519.pub` into the import folder and
  load them. Private keys are stored app-private, never on shared storage.
- **Load config from files** — put `commands.toml`, `id_ed25519` and `id_ed25519.pub`
  in the import folder and load them from the config screen. Press **Ekspor** first:
  the app creates the folder and writes a `commands.toml` template you can edit. The
  exact path is shown on screen (`Android/media/<pkg>/import` on most devices).
- **Minimal permissions.** The app declares **INTERNET and ACCESS_NETWORK_STATE only**,
  is not debuggable, and is signed with a real release key. Earlier builds inherited
  camera, location, media, biometric, bluetooth and full package enumeration from the
  UI framework's manifest template — enough for banking apps with anti-fraud SDKs to
  refuse to run alongside it.

## Install & run

1. Open the download link on the phone and install (allow "unknown sources").
2. Bring up your VPN if the host is only reachable through one.
3. Open the burger `≡` → **SSH Key** → generate a key, and add its public half to
   `~/.ssh/authorized_keys` on the server.
4. Under **Perintah**, set host / port / user and pick that key.
5. At the `leuwi>` prompt: `<nama>-ls` lists sessions, `<nama>-s <sesi>` attaches one.
   Arrow up/down walks command history.

> **Coming from a build before dev.21?** The signing key changed (a real release key
> instead of the Android debug key), so this cannot update such an install —
> uninstall the old one first.
