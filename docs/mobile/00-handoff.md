# Leuwi Panjang Android — Handoff / Current State

Snapshot to continue the Android build on another machine (e.g. **p16s-gen6**).
Last updated at **v0.1.0-dev.20** (2026‑07). Read this first, then
`03-makepad-android-patches.md` and `02-android-build.md`.

## Goal (non‑technical)

An Android APK of the Leuwi Panjang terminal so Hendri can SSH into the **nvgpu**
GPU server from a phone and attach tmux — exactly like the `nvgpu-s` fish helper
(`ssh` → `tmux new -A -s <session>`) on the desktop. Same Rust codebase as desktop;
only the backend (SSH vs local PTY) and the platform shim differ.

## Where things live

- **Code**: `src/lib.rs` (whole app + 111 tests, built as Android cdylib + desktop rlib),
  `src/main.rs` (thin desktop entry), `src/ssh.rs` (russh SSH backend + TOFU host-key),
  `src/bin/ssh-smoke.rs` (headless SSH self-test).
- **Branches** (`git@github.com:situkangsayur/leuwi-panjang.git`):
  - `p16s-gen6` — current work (this branch).
  - `feat/android-ssh-apk` — same history (dev.16→dev.20).
  - `apk` — **binary APKs** (`leuwipanjang_v0.1.0-dev.20.apk` + `leuwipanjang.apk` latest).
    Download: `https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang.apk`
  - `main` — older releases (dev.15 and before).

## Status — what works ✅ / what's left ⬜

- ✅ **APK builds, installs, LAUNCHES and RENDERS** the terminal UI (fonts, tab bar,
  status bar). This was the big blocker ("won't open") — fixed in dev.20, verified in
  an x86_64 emulator.
- ✅ SSH backend (russh 0.62, ring), tmux attach-or-create, `list_sessions`, TOFU
  host-key pinning. Verified against `localhost:1313` and shown connecting in-app.
- ✅ Touch: tap = keyboard (IME), one-finger drag = scroll. Multi-tab bar (tap `+`/tab/`×`),
  each Android tab = its own nvgpu tmux session.
- ⬜ **SSH key on device** — the app reads `~/.ssh/id_rsa`, which doesn't exist on a phone,
  so it shows `load key .ssh/id_rsa: No such file`. NEXT: provision a key (read from the
  app's external files dir, or user import, or generate-on-device + register pubkey).
  **Do NOT embed the private key** — the APK is on a public branch.
- ⬜ **App icon** (currently none). git cargo-makepad has no icon option → inject a
  `mipmap/ic_launcher` post-build (logo at repo root `Leuwi-Panjang.png`).
- ⬜ **Accessory key bar** (Esc/Tab/Ctrl/Alt/arrows/pipe) above the keyboard + touch
  select→copy + paste. Needs keyboard-height handling (makepad `ResizeTextIME`).
- ⬜ **WireGuard**: NOT our problem — Hendri uses the official WireGuard Android app for
  the tunnel; the app dials `10.100.21.22` over normal TCP once the tunnel is up.
- ⬜ **Reproducibility**: the makepad fixes are a patch applied to `~/.cargo` (see below),
  not a vendored fork. Longer-term: vendor makepad as a local `[patch]`.

## Build on a new machine (p16s-gen6)

```bash
git clone -b p16s-gen6 git@github.com:situkangsayur/leuwi-panjang.git
cd leuwi-panjang/leuwi-panjang           # the crate dir (has Cargo.toml)

# One-time setup: cargo-makepad(git), full NDK, JDK17, SDK symlinks, makepad patches.
# Needs: rustup, a JDK 17 (default /usr/lib/jvm/java-17-openjdk-amd64, else set LEUWI_JDK17).
./install/setup-android-build.sh
#   → if it says "makepad checkout not found yet", run one build (below), then re-run
#     setup once to apply the makepad patch, then build again.

# Build the signed arm64 APK:
./install/build-apk.sh
#   → target/makepad-android-apk/leuwi-panjang/apk/leuwipanjang.apk  (+ copy at ../)
```

### The 4 makepad patches (why the build fails without them)
Applied by `setup-android-build.sh` from `install/makepad-patches/makepad-android-ff9048c.patch`
to the cargo git checkout of makepad. Full rationale in `03-makepad-android-patches.md`:
1. `opengl.rs` — `#version` must be shader line 1 (else black screen on strict GLES).
2. `libc_sys.rs` — `SYS_GETTID` per-arch (x86_64 emulator only).
3. `android_jni.rs` — null-safe `jstring_to_string`.
4. `openxr_sys.rs` — `c_char` signedness (x86_64 emulator only).

Plus the **root-cause tooling fix**: cargo-makepad MUST come from the same makepad commit
as the dep (crates.io 0.4.0's 2‑arg `onAndroidParams` vs git-main's 6‑arg native = the
startup crash). `setup-android-build.sh` installs the matching one.

## Testing without a phone (x86_64 emulator)

The emulator can only run an **x86_64** APK (no arm64 translation). Build one with
`--abi=x86_64` and the `x86_64-…-clang` CC (see `03-…patches.md` for the exact command),
sign to a **separate** name (`leuwipanjang-x86_64.apk`), then:
```bash
LEUWI_SETUP_EMULATOR=1 ./install/setup-android-build.sh   # installs emulator+image (once)
sg kvm -c "$HOME/android_33_sdk/emulator/emulator -avd leuwi -no-window -gpu swiftshader_indirect -no-snapshot -read-only -no-audio" &
ADB=$HOME/android_33_sdk/platform-tools/adb
$ADB install -r leuwipanjang-x86_64.apk
$ADB shell am start -n com.situkangsayur.leuwipanjang/.MakepadApp
$ADB logcat -d | grep -iE 'Makepad|FATAL|SIGABRT'    # diagnose
$ADB exec-out screencap -p > screen.png              # see the UI
```
This is how dev.20 was diagnosed and verified. Needs `/dev/kvm` access (kvm group).

## nvgpu / network facts
- nvgpu SSH: `10.100.21.22:1313`, user `hendri`, key `~/.ssh/id_rsa`, `tmux new -A -s main`.
- Reach it over WireGuard (`wg0`, allowed IPs `10.100.21.0/24`, hub `103.82.92.108:52525`).
- Build box so far = **nvda11-gpu** (`10.100.21.22`); it is also the nvgpu host (sshd `:1313`),
  which is why SSH could be verified locally.
