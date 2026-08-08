# Makepad Android patches (required to build a working APK)

The stock `makepad` git-main + `cargo-makepad` crates.io 0.4.0 combo produces an APK
that **installs but crashes on launch / renders black**. Diagnosed by running the app
in an x86_64 Android emulator on the build box (`adb logcat` + screenshots). Four fixes
are required; three are edits to the makepad checkout, one is a tooling version match.

## 0. Tooling: cargo-makepad MUST match the makepad dep commit

Root cause of "won't open": crates.io `cargo-makepad` 0.4.0 generates Java that calls
`onAndroidParams(cache_path, density)` (2 args), but makepad git-main's native
`onAndroidParams` reads **6** args (`+ is_emulator, android_version, build_number,
kernel_version`). The extra args are read from garbage stack → `GetStringUTFChars(null)`
→ `SIGABRT` under CheckJNI (debug builds) on **both** arm64 and x86_64.

Fix — install cargo-makepad from the SAME commit as the `makepad-widgets` dep
(see `Cargo.lock`, currently `ff9048cc37985632c96a114a7ad602c41aef2aa1`):

```bash
cargo install --git https://github.com/makepad/makepad \
  --rev ff9048cc37985632c96a114a7ad602c41aef2aa1 cargo-makepad --force
```

This git cargo-makepad expects a standard SDK layout, so symlink the cargo-makepad-0.4.0
paths to what it wants:

```bash
SDK=$HOME/android_33_sdk
ln -sfn $HOME/android-ndk-r26d      $SDK/ndk/25.2.9519653      # full NDK (see doc 02)
ln -sfn $SDK/android-13             $SDK/build-tools/33.0.1
ln -sfn $SDK/android-33-ext4        $SDK/platforms/android-33-ext4
ln -sfn /usr/lib/jvm/java-17-openjdk-amd64 $SDK/openjdk        # d8 rejects Java 21
```

## Makepad source patches (in `~/.cargo/git/checkouts/makepad-*/ff9048c/`)

These live in the cargo git checkout (not version-controlled), so they are captured in
`install/makepad-patches/` and re-applied with:

```bash
./install/apply-makepad-patches.sh          # apply (idempotent)
./install/apply-makepad-patches.sh --check  # is the checkout patched?
```

`install/build-release.sh` runs the check itself and applies when needed, so a fresh
clone builds without hand-editing `~/.cargo`. The script also clears the compiled
`makepad-platform`: **Cargo treats a git dependency as immutable and will happily link
artifacts built before the patch**, which shows up as `cannot find function
to_java_paste_from_clipboard` even though the function is right there in the file.

Paths below are under `platform/src/os/linux/`.

1. **`opengl.rs`** — `#version` must be the FIRST token of the shader. Makepad's
   `format!("\n            {version}…")` put `#version 300 es` on line 2 → strict GLES
   drivers (emulator, many Adreno/Mali) reject it → **black screen**. Change both the
   vertex and pixel `format!` to start with `{version}` immediately (no leading newline).

2. **`libc_sys.rs`** — `SYS_GETTID` is hardcoded `178` (correct on aarch64/generic, but
   `178 = query_module` on x86_64 → seccomp `SIGSYS`). Make it arch-specific:
   `#[cfg(target_arch="x86_64")] =186; else =178`. (arm64 phone unaffected; only needed
   to run the x86_64 emulator.)

3. **`android/android_jni.rs`** — `jstring_to_string` calls `GetStringUTFChars` with no
   null-check and `.unwrap()`s UTF-8. Add `if java_string.is_null() { return String::new() }`
   and use `.to_str().unwrap_or("")`. Defensive; belt-and-suspenders with fix #0.

4. **`openxr_sys.rs`** — assumes `c_char == u8` (true on ARM, but `i8` on x86_64) →
   4 `E0308` type errors when compiling for x86_64. Cast the `&[c_char]`/`[c_char;N]`
   spots. (x86_64-only; not needed for the arm64 phone build.)

## Build

Desktop-emulator (x86_64, to test/see the app here):
```bash
CC_x86_64_linux_android=$NDK/bin/x86_64-linux-android29-clang \
AR_x86_64_linux_android=$NDK/bin/llvm-ar \
cargo makepad android --sdk-path=$SDK \
  --package-name=com.situkangsayur.leuwipanjang --app-label="Leuwi Panjang" \
  --abi=x86_64 build -p leuwi-panjang
```
Phone (arm64): same with `--abi=aarch64` and the `aarch64-…-clang` CC. The git
cargo-makepad now completes packaging itself ("Compile APK completed"); finish with
`zipalign -p -f 4` + `apksigner` as in `install/build-apk.sh`.

## Feature patches (not bug fixes — makepad has no API for these)

5. **Tab notifications** (`android_jni.rs` + `MakepadActivity.java`) — `showTabNotification`
   / `cancelTabNotification` / `takePendingTab`, so a tab waiting on an answer can raise
   a phone notification and the tap can bring that tab forward. Polled from the frame
   tick rather than pushed, which avoids a native-callback registration path for one int.

6. **Clipboard paste** (`android_jni.rs` + `MakepadActivity.java`) — upstream implements
   `copyToClipboard` but leaves the paste direction commented out
   (`from_java_on_paste_from_clipboard`), so pasting *into* the terminal from another app
   was impossible. Adds `pasteFromClipboard(): String` on the activity and
   `to_java_paste_from_clipboard()` on the Rust side. Android only serves the clipboard
   to the focused app, which is the case whenever the paste key can be pressed.

7. **Manifest trim + app icon** (`cargo_makepad/src/android/mod.rs`, `res/`) — the stock
   template requests CAMERA / LOCATION / MEDIA / BLUETOOTH / QUERY_ALL_PACKAGES and marks
   the app `debuggable`; a terminal needs INTERNET + ACCESS_NETWORK_STATE (+ POST_NOTIFICATIONS).
   The icon files are untracked binaries, so they are archived under
   `install/makepad-patches/res/` and copied rather than diffed.

## Still TODO
Vendor makepad as a local `[patch]` fork carrying all of the above and pin cargo-makepad,
so the patches are a dependency rather than a script that edits `~/.cargo` in place.
