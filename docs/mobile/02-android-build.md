# Android Build

How Leuwi Panjang cross-compiles to an Android APK from the **same** code used on the
desktop. Introduced in **v0.1.0-dev.15** (pipeline); the SSH build below is
**v0.1.0-dev.16** (2026-07-09), a real signed APK containing the russh SSH backend.

## Goal

Run the terminal on a phone so it can reach the **nvgpu GPU server over SSH**, the
same way `nvgpu-s <session>` (SSH + tmux) works from the kitty desktop terminal.

## Crate layout

The app lives in **`src/lib.rs`** (built as an Android **cdylib** by cargo-makepad, and
as an `rlib` on desktop). **`src/main.rs`** is a thin desktop entry that calls
`leuwi_panjang::app_main()`. `src/ssh.rs` is the SSH backend; `src/bin/ssh-smoke.rs` is
a headless self-test. cargo-makepad refuses to build a bin-only package ("no library
targets found"), hence the lib split.

## Toolchain (one-time)

```bash
cargo install cargo-makepad
SDK=$HOME/android_33_sdk
cargo makepad android --sdk-path=$SDK install-toolchain   # SDK + minimal NDK + rust target
```

**Two workarounds are needed** because cargo-makepad targets *pure-Rust* makepad, while
our SSH stack pulls in `ring` (which compiles C and needs libunwind):

1. **Full NDK.** cargo-makepad's bundled NDK is stripped — no C headers, no `llvm-ar`,
   no `libunwind.a`. Replace it with the official NDK:
   ```bash
   curl -fL https://dl.google.com/android/repository/android-ndk-r26d-linux.zip -o /tmp/ndk.zip
   unzip -q /tmp/ndk.zip -d $HOME
   ln -sfn $HOME/android-ndk-r26d $SDK/NDK      # used for both C-compile and link
   ```
2. **JDK ≤ 17.** cargo-makepad's `d8` dexer rejects Java 21 bytecode
   (`Unsupported class file major version 65`). Point its `openjdk` at a JDK 17:
   ```bash
   ln -sfn /usr/lib/jvm/java-17-openjdk-amd64 $SDK/openjdk
   ```

## Building the APK

Use the wrapper — it sets the `CC/AR` env for `ring`, runs cargo-makepad, and finishes
packaging (cargo-makepad's own final `aapt` step errors with "nothing to do", so the
script does `zipalign` + `apksigner` by hand):

```bash
./install/build-apk.sh
# Output (signed, v1+v2+v3):
#   target/makepad-android-apk/leuwi-panjang/apk/leuwipanjang.apk   (~37 MB debug)
#   ../leuwipanjang.apk                                             (copy)
```

Under the hood it runs (note **`-p leuwi-panjang`**, not `--package …` — cargo-makepad
mis-parses the space-separated flag and names the app `--package`):

```bash
CC_aarch64_linux_android=$SDK/NDK/.../aarch64-linux-android29-clang \
AR_aarch64_linux_android=$SDK/NDK/.../llvm-ar \
cargo makepad android --sdk-path=$SDK \
  --package-name=com.situkangsayur.leuwipanjang --app-label="Leuwi Panjang" \
  build -p leuwi-panjang
```

Verified APK: `package=com.situkangsayur.leuwipanjang`, label `Leuwi Panjang`,
`native-code arm64-v8a`, `INTERNET` permission present, signed & verified. The native
`.so` (renamed `libmakepad.so` inside the APK, as makepad's Java shim expects) contains
the russh symbols and the SSH banner strings.

> Built on **nvda11-gpu** (headless): the SDK/NDK/JDK live there; adb `run` needs a
> connected device, so install by copying the APK to the phone.

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
- **App identity** is set on the cargo-makepad command line
  (`--package-name=com.situkangsayur.leuwipanjang --app-label="Leuwi Panjang"`), *not* the
  `[package.metadata.*]` blocks — cargo-makepad ignores those. The APK's `versionCode`/
  `versionName` come out empty unless passed as makepad options.

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

## Milestone B — SSH backend (DONE, v0.1.0-dev.16)

The local-echo placeholder is replaced by a real SSH connection using **`russh`**
(pure-Rust SSH — `portable-pty` cannot run on Android). The backend lives in
`src/ssh.rs` and is **shared by desktop and Android**: on Android it is the only
backend (no local shell); on desktop it powers optional SSH tabs and the headless
self-test.

### Dependencies (`Cargo.toml`, all targets)

```toml
russh = { version = "0.62", default-features = false, features = ["ring", "rsa", "flate2"] }
tokio = { version = "1", features = ["rt","rt-multi-thread","io-util","net","sync","macros","time"] }
```

- **`ring` crypto backend**, not the default `aws-lc-rs`: aws-lc-rs needs `nasm`/`clang`
  to build its assembly and is painful to cross-compile; `ring` only needs `cc` + `perl`
  and cross-compiles cleanly to Android arm64.
- **`rsa`** feature so the `id_rsa` (RSA) key authenticates with rsa-sha2-256/512.
- `russh-keys` is **no longer a separate crate** — key loading is `russh::keys` in 0.62.

### How it works

`src/ssh.rs` exposes:
- `SshProfile` — host/port/user/key/session (+ optional `startup` command).
- `spawn(profile, sink)` — connects on a background thread with its own current-thread
  Tokio runtime, requests a PTY, runs the startup command, and streams remote bytes into
  the caller's `sink` (the tab's `TermGrid`). Returns an `SshHandle` whose `writer()`
  feeds keystrokes and `resize()` forwards `window-change` (SIGWINCH).
- `list_sessions(profile)` — one-shot `tmux list-sessions` over an exec channel
  (mirrors `nvgpu-ls`).

`TermTab::spawn_ssh` wires a profile to a tab; Android's `TermTab::spawn` uses it by
default. On desktop, `LEUWI_SSH=1` makes tabs SSH too, for GUI testing.

### Embedded nvgpu profile (config defaults)

The app ships the `nvgpu` profile as `Config` defaults (override in
`~/.config/leuwi-panjang/config.toml`):

| Field         | Default          | Notes                                    |
|---------------|------------------|------------------------------------------|
| `ssh_host`    | `10.100.21.22`   | nvgpu VPN IP                             |
| `ssh_port`    | `1313`           |                                          |
| `ssh_user`    | `hendri`         |                                          |
| `ssh_key`     | `~/.ssh/id_rsa`  | `~` expanded to home                     |
| `ssh_session` | `main`           | `tmux new -A -s <session>` (attach-or-create) |
| `ssh_startup` | *(empty)*        | full command override — see below        |

### tmux session management (matches `nvgpu-s` / `nvgpu-ls`)

- **Attach existing or create new**: the default startup `tmux new -A -s <ssh_session>`
  attaches the session if it exists, else creates it — exactly the `nvgpu-s` behaviour.
- **See the list / pick a session on connect**: set
  `ssh_startup = "tmux new -A -s main \\; choose-tree -Zs"`. tmux's `choose-tree`
  session picker renders through the terminal; navigate + Enter to attach, or create a
  new one. This is the most touch-friendly way to browse sessions on a phone.
- Inside a session, tmux's own keys still work: `prefix s` (session list/switch),
  `prefix $` (rename), `:new -s name` (new session).

### Headless self-test (`ssh-smoke`)

The GUI can't be linked on a headless box (makepad needs desktop audio/X dev libs), so
the SSH path is verified without it via the standalone `ssh-smoke` bin (compiles only
`src/ssh.rs`):

```bash
# List sessions on the target (compare against `tmux ls`)
LEUWI_SSH_HOST=localhost LEUWI_SSH_PORT=1313 \
  cargo run --bin ssh-smoke -- list

# Attach a throwaway session, stream its PTY for a few seconds
LEUWI_SSH_HOST=localhost LEUWI_SSH_PORT=1313 \
  cargo run --bin ssh-smoke -- attach my-test-session
```

Verified on **nvda11-gpu** (which is itself the nvgpu host, sshd on `:1313`):
`list` output matches `tmux ls` byte-for-byte; `attach` creates/attaches the session and
streams the live prompt, and keystrokes (Ctrl-L redraw) reach the remote.

> After the lib/bin split, `cargo build --bin ssh-smoke` also builds the cdylib, which
> can't link on a headless box. Run it where the makepad GUI links (a desktop with
> `libasound2-dev`/`libpulse-dev`), or use the already-built `target/debug/ssh-smoke`.

## Host-key verification (hardening, DONE)

`src/ssh.rs` pins the server key on first connect (TOFU) in
`~/.config/leuwi-panjang/known_hosts` and rejects a changed key on later connects with a
loud MITM warning — the classic ssh trust-on-first-use. Verified against `localhost:1313`:
first connect pins `SHA256:…`, second matches, a tampered entry is rejected. Remaining:
an explicit "new host key, accept?" prompt (currently silent-pin) and a native session
picker UI (vs. the `choose-tree` approach).

## Reaching nvgpu's VPN IP — WireGuard

The SSH backend dials `10.100.21.22`, which is only routable through the WireGuard tunnel.

**Working today (recommended):** install the official **WireGuard Android app**, import
the same peer config used on the laptop (`wg0`), and bring the tunnel up. Leuwi Panjang
then reaches `10.100.21.22:1313` over normal TCP — no in-app VPN needed. For a quick test
without WG, put the phone on a network that already routes to nvgpu, or point `ssh_host`
at a reachable host.

**Milestone C — embedded WireGuard (future).** Bundle **boringtun** (userspace WG, pure
Rust) so the tunnel is one tap inside the app. Two viable designs:
- **boringtun + smoltcp, fully in-process**: a userspace TCP stack turns "connect to
  10.100.21.22:1313" into IP packets, boringtun encrypts them to UDP for the WG endpoint,
  and russh runs over that stream via `client::connect_stream`. No Android `VpnService`,
  no root — the elegant path, but ~400 lines of intricate async glue.
- **Android `VpnService` + boringtun**: a system TUN routes all app traffic; needs custom
  Java/Kotlin in the (cargo-makepad-generated) shim and a foreground service.

Not yet implemented: it can't be validated without the real WG server keys/endpoint and a
reachable peer, and shipping untested networking into the working SSH build is a net
negative. Config plumbing (`[wireguard]` block → dial via WG when present, else direct
TCP) lands with the implementation.
