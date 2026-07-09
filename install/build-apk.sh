#!/usr/bin/env bash
# Build a signed Android APK for Leuwi Panjang (arm64).
#
# Why this wrapper exists: cargo-makepad ships a *minimal* NDK (pure-Rust makepad
# needs no C toolchain), but our SSH backend pulls in `ring`, which must compile C
# and link libunwind. So we point the build at a *full* NDK, and finish the last
# packaging steps (zipalign + apksigner) by hand because cargo-makepad's final
# aapt step errors out ("nothing to do") on this toolchain.
#
# One-time prerequisites (see docs/mobile/02-android-build.md):
#   cargo install cargo-makepad
#   cargo makepad android --sdk-path="$SDK" install-toolchain
#   # full NDK (has C headers + libunwind + llvm-ar), replacing the minimal one:
#   curl -fL https://dl.google.com/android/repository/android-ndk-r26d-linux.zip -o ndk.zip
#   unzip -q ndk.zip -d "$HOME" && ln -sfn "$HOME/android-ndk-r26d" "$SDK/NDK"
#   # a JDK <= 17 (d8 rejects Java 21 bytecode) symlinked where makepad expects it:
#   ln -sfn /usr/lib/jvm/java-17-openjdk-amd64 "$SDK/openjdk"
set -euo pipefail

SDK="${LEUWI_ANDROID_SDK:-$HOME/android_33_sdk}"
NDK="$SDK/NDK/toolchains/llvm/prebuilt/linux-x86_64"
PKG_ID="${LEUWI_PKG_ID:-com.situkangsayur.leuwipanjang}"
APP_LABEL="${LEUWI_APP_LABEL:-Leuwi Panjang}"
KS="${LEUWI_KEYSTORE:-$HOME/.android/debug.keystore}"
CRATE_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# ring compiles C for android → point cc-rs at the full NDK (api29 == min_sdk).
export CC_aarch64_linux_android="$NDK/bin/aarch64-linux-android29-clang"
export CXX_aarch64_linux_android="$NDK/bin/aarch64-linux-android29-clang++"
export AR_aarch64_linux_android="$NDK/bin/llvm-ar"
export RANLIB_aarch64_linux_android="$NDK/bin/llvm-ranlib"
export PATH="$HOME/.cargo/bin:$PATH"

echo ">> building cdylib + base APK via cargo-makepad"
cd "$CRATE_DIR"
# cargo-makepad's last aapt step fails on this toolchain; ignore it — the 38 MB
# unaligned APK it leaves behind is complete (manifest + dex + native lib).
cargo makepad android --sdk-path="$SDK" \
  --package-name="$PKG_ID" --app-label="$APP_LABEL" \
  build -p leuwi-panjang || echo ">> (ignoring cargo-makepad final aapt step; finishing manually)"

APKDIR="$CRATE_DIR/target/makepad-android-apk/leuwi-panjang/apk"
# cargo-makepad names the unaligned APK from the app-label; glob rather than guess.
UNALIGNED="$(ls -t "$APKDIR"/*.unaligned.apk 2>/dev/null | head -1)"
OUT="$APKDIR/leuwipanjang.apk"
[ -n "$UNALIGNED" ] && [ -f "$UNALIGNED" ] || { echo "ERROR: no *.unaligned.apk in $APKDIR"; exit 1; }

if [ ! -f "$KS" ]; then
  echo ">> creating debug keystore at $KS"
  mkdir -p "$(dirname "$KS")"
  "$SDK/openjdk/bin/keytool" -genkeypair -v -keystore "$KS" -storepass android \
    -alias androiddebugkey -keypass android -keyalg RSA -keysize 2048 -validity 10000 \
    -dname "CN=Android Debug,O=Android,C=US"
fi

echo ">> zipalign"
"$SDK/android-13/zipalign" -p -f 4 "$UNALIGNED" "$APKDIR/aligned.tmp.apk"
echo ">> apksigner sign (v1+v2+v3)"
"$SDK/openjdk/bin/java" -jar "$SDK/android-13/lib/apksigner.jar" sign \
  --ks "$KS" --ks-pass pass:android --key-pass pass:android \
  --ks-key-alias androiddebugkey --out "$OUT" "$APKDIR/aligned.tmp.apk"
rm -f "$APKDIR/aligned.tmp.apk"
"$SDK/openjdk/bin/java" -jar "$SDK/android-13/lib/apksigner.jar" verify "$OUT" >/dev/null \
  && echo ">> signature verified"

cp "$OUT" "$CRATE_DIR/../leuwipanjang.apk" 2>/dev/null || true
echo ">> DONE: $OUT ($(du -h "$OUT" | cut -f1))"
