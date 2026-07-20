#!/usr/bin/env bash
# One-time setup to build the Leuwi Panjang Android APK on a fresh machine
# (e.g. p16s-gen6). Idempotent — safe to re-run. See docs/mobile/00-handoff.md
# and docs/mobile/03-makepad-android-patches.md for the why behind each step.
#
# After this, build with:  ./install/build-apk.sh
set -euo pipefail

SDK="${LEUWI_ANDROID_SDK:-$HOME/android_33_sdk}"
NDK_VER="android-ndk-r26d"
NDK_URL="https://dl.google.com/android/repository/${NDK_VER}-linux.zip"
CMDTOOLS_URL="https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip"
# Must match the makepad-widgets commit in Cargo.lock:
MAKEPAD_REV="$(grep -A3 'name = "makepad-widgets"' "$(dirname "$0")/../Cargo.lock" \
  | grep -oE '#[0-9a-f]{40}' | tr -d '#' | head -1)"
JDK17="${LEUWI_JDK17:-/usr/lib/jvm/java-17-openjdk-amd64}"
HERE="$(cd "$(dirname "$0")" && pwd)"

echo ">> makepad rev: $MAKEPAD_REV"

echo ">> [1/6] cargo-makepad from the SAME makepad commit (crates.io 0.4.0 is incompatible)"
cargo install --git https://github.com/makepad/makepad --rev "$MAKEPAD_REV" cargo-makepad --force

echo ">> [2/6] android SDK + minimal toolchain (idempotent)"
[ -d "$SDK" ] || cargo makepad android --sdk-path="$SDK" install-toolchain

echo ">> [3/6] full NDK (bundled one is stripped: no C headers/libunwind/llvm-ar, ring needs them)"
if [ ! -d "$HOME/$NDK_VER" ]; then
  curl -fL "$NDK_URL" -o /tmp/ndk.zip && unzip -q /tmp/ndk.zip -d "$HOME" && rm -f /tmp/ndk.zip
fi

echo ">> [4/6] SDK layout symlinks expected by the git cargo-makepad"
mkdir -p "$SDK/ndk" "$SDK/build-tools" "$SDK/platforms"
ln -sfn "$HOME/$NDK_VER"      "$SDK/ndk/25.2.9519653"
ln -sfn "$SDK/android-13"     "$SDK/build-tools/33.0.1"      2>/dev/null || true
ln -sfn "$SDK/android-33-ext4" "$SDK/platforms/android-33-ext4" 2>/dev/null || true
[ -d "$JDK17" ] && ln -sfn "$JDK17" "$SDK/openjdk" || echo "   WARN: JDK17 not at $JDK17 — set LEUWI_JDK17 (d8 rejects Java 21)"

echo ">> [5/6] apply makepad patches (#version shader, gettid, jstring null, c_char) to the cargo checkout"
MK="$(find "$HOME/.cargo/git/checkouts" -maxdepth 2 -type d -path '*makepad*' -name "$(echo "$MAKEPAD_REV" | cut -c1-7)" 2>/dev/null | head -1)"
if [ -n "$MK" ] && [ -d "$MK" ]; then
  if git -C "$MK" apply --check "$HERE/makepad-patches/makepad-android-ff9048c.patch" 2>/dev/null; then
    git -C "$MK" apply "$HERE/makepad-patches/makepad-android-ff9048c.patch"
    echo "   patched $MK"
  else
    echo "   patch already applied (or conflicts) — verify manually if the build fails"
  fi
else
  echo "   makepad checkout not found yet; run a build once, then re-run this script to patch"
fi

echo ">> [6/6] optional: x86_64 Android emulator for on-box testing (see handoff doc)"
echo "   (skipped by default; set LEUWI_SETUP_EMULATOR=1 to install emulator + system image)"
if [ "${LEUWI_SETUP_EMULATOR:-0}" = "1" ]; then
  [ -x "$SDK/cmdline-tools/latest/bin/sdkmanager" ] || {
    curl -fL "$CMDTOOLS_URL" -o /tmp/ct.zip && mkdir -p "$SDK/cmdline-tools" \
      && unzip -q -o /tmp/ct.zip -d "$SDK/cmdline-tools" \
      && mv "$SDK/cmdline-tools/cmdline-tools" "$SDK/cmdline-tools/latest" 2>/dev/null || true; }
  JAVA_HOME="$SDK/openjdk" yes | "$SDK/cmdline-tools/latest/bin/sdkmanager" --sdk_root="$SDK" --licenses >/dev/null 2>&1 || true
  JAVA_HOME="$SDK/openjdk" "$SDK/cmdline-tools/latest/bin/sdkmanager" --sdk_root="$SDK" \
    "emulator" "platform-tools" "platforms;android-33" "system-images;android-33;google_apis;x86_64"
  echo "no" | "$SDK/cmdline-tools/latest/bin/avdmanager" create avd -n leuwi \
    -k "system-images;android-33;google_apis;x86_64" --device pixel_5 --force || true
  sudo usermod -aG kvm "$USER" 2>/dev/null || true
fi

echo ">> DONE. Build with: ./install/build-apk.sh"
