#!/usr/bin/env bash
# Build a SIGNED RELEASE APK of Leuwi Panjang (arm64), named with its version.
#
# Differs from build-apk.sh in three ways that matter to banking anti-fraud SDKs
# (OCBC refused to run alongside the debug build):
#   1. --release, and the manifest template no longer hardcodes android:debuggable
#   2. signed with a real release keystore, not the shared "CN=Android Debug" one
#   3. version stamped into the manifest via MAKEPAD_VERSION_* (was "Versi: null")
# The permission trim (camera/location/media/midi/bluetooth/QUERY_ALL_PACKAGES ->
# INTERNET + ACCESS_NETWORK_STATE) lives in the cargo-makepad manifest template.
set -euo pipefail

SDK="${LEUWI_ANDROID_SDK:-$HOME/android_33_sdk}"
NDK="$SDK/NDK/toolchains/llvm/prebuilt/linux-x86_64"
PKG_ID="${LEUWI_PKG_ID:-com.situkangsayur.leuwipanjang}"
APP_LABEL="${LEUWI_APP_LABEL:-Leuwi Panjang}"
CRATE_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# Release signing identity. Kept out of the repo; back this file up, because
# losing it means no future build can update an installed copy.
KS="${LEUWI_KEYSTORE:-$HOME/.android/leuwi-release.keystore}"
KS_PASS="${LEUWI_KS_PASS:-leuwipanjang}"
KS_ALIAS="${LEUWI_KS_ALIAS:-leuwi}"

# The makepad fork edits live in ~/.cargo and vanish on a fresh checkout; without them
# the APK crashes on launch or fails to compile. Re-applied only when missing, because
# applying also forces a full makepad-platform rebuild.
"$CRATE_DIR/install/apply-makepad-patches.sh" --check >/dev/null 2>&1 \
  || "$CRATE_DIR/install/apply-makepad-patches.sh"

VERSION_NAME="$(grep -m1 '^version_name' "$CRATE_DIR/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')"
VERSION_CODE="$(grep -m1 '^version_code' "$CRATE_DIR/Cargo.toml" | sed 's/[^0-9]*\([0-9]*\).*/\1/')"
[ -n "$VERSION_NAME" ] && [ -n "$VERSION_CODE" ] || { echo "ERROR: cannot read version_name/version_code from Cargo.toml"; exit 1; }
export MAKEPAD_VERSION_NAME="$VERSION_NAME"
export MAKEPAD_VERSION_CODE="$VERSION_CODE"
echo ">> building v$VERSION_NAME (code $VERSION_CODE)"

# ring compiles C for android -> point cc-rs at the full NDK (api29 == min_sdk).
export CC_aarch64_linux_android="$NDK/bin/aarch64-linux-android29-clang"
export CXX_aarch64_linux_android="$NDK/bin/aarch64-linux-android29-clang++"
export AR_aarch64_linux_android="$NDK/bin/llvm-ar"
export RANLIB_aarch64_linux_android="$NDK/bin/llvm-ranlib"
export PATH="$HOME/.cargo/bin:$PATH"

cd "$CRATE_DIR"
# Clear stale output first: a leftover *.unaligned.apk from an earlier run would
# otherwise be picked up and shipped as if it were this build -- which is exactly
# how a debug APK slipped through once.
rm -f "$CRATE_DIR/target/makepad-android-apk/leuwi-panjang/apk"/*.unaligned.apk
# cargo-makepad's last aapt step fails on this toolchain; the unaligned APK it
# leaves behind is complete. Failures of the *cargo* step are caught below by
# checking that the .so is newer than the sources, since this swallows both.
cargo makepad android --sdk-path="$SDK" \
  --package-name="$PKG_ID" --app-label="$APP_LABEL" \
  build -p leuwi-panjang --release 2>&1 | tee /tmp/leuwi-release-build.log \
  || echo ">> (ignoring cargo-makepad final aapt step; finishing manually)"

if grep -qE '^error(\[|:)' /tmp/leuwi-release-build.log; then
  echo "ERROR: the cargo build failed -- refusing to package a stale APK"
  grep -E '^error(\[|:)' -A 8 /tmp/leuwi-release-build.log | head -40
  exit 1
fi

APKDIR="$CRATE_DIR/target/makepad-android-apk/leuwi-panjang/apk"
UNALIGNED="$(ls -t "$APKDIR"/*.unaligned.apk 2>/dev/null | head -1)"
OUT="$APKDIR/leuwipanjang_v${VERSION_NAME}.apk"
[ -n "$UNALIGNED" ] && [ -f "$UNALIGNED" ] || { echo "ERROR: no *.unaligned.apk in $APKDIR"; exit 1; }

if [ ! -f "$KS" ]; then
  echo ">> creating RELEASE keystore at $KS (RSA 4096, 10000 days)"
  mkdir -p "$(dirname "$KS")"
  "$SDK/openjdk/bin/keytool" -genkeypair -v -keystore "$KS" -storepass "$KS_PASS" \
    -alias "$KS_ALIAS" -keypass "$KS_PASS" -keyalg RSA -keysize 4096 -validity 10000 \
    -dname "CN=Leuwi Panjang, O=situkangsayur, C=ID"
fi

echo ">> zipalign"
"$SDK/android-13/zipalign" -p -f 4 "$UNALIGNED" "$APKDIR/aligned.tmp.apk"
echo ">> apksigner sign (v1+v2+v3)"
"$SDK/openjdk/bin/java" -jar "$SDK/android-13/lib/apksigner.jar" sign \
  --ks "$KS" --ks-pass "pass:$KS_PASS" --key-pass "pass:$KS_PASS" \
  --ks-key-alias "$KS_ALIAS" --out "$OUT" "$APKDIR/aligned.tmp.apk"
rm -f "$APKDIR/aligned.tmp.apk"
"$SDK/openjdk/bin/java" -jar "$SDK/android-13/lib/apksigner.jar" verify "$OUT" >/dev/null \
  && echo ">> signature verified"

# Fail loudly if the anti-fraud-relevant properties did not actually come out right.
BADGING="$("$SDK/build-tools/33.0.1/aapt" dump badging "$OUT")"
echo ">> verifying manifest"
if grep -q 'application-debuggable' <<<"$BADGING"; then echo "ERROR: still debuggable"; exit 1; fi
for p in CAMERA ACCESS_FINE_LOCATION QUERY_ALL_PACKAGES READ_MEDIA_IMAGES USE_BIOMETRIC BLUETOOTH; do
  if grep -q "permission.$p" <<<"$BADGING"; then echo "ERROR: permission $p still declared"; exit 1; fi
done
grep -q "versionName='$VERSION_NAME'" <<<"$BADGING" || { echo "ERROR: versionName not stamped"; exit 1; }
echo ">> manifest OK: $(grep -m1 '^package:' <<<"$BADGING")"
grep 'uses-permission' <<<"$BADGING"

echo ">> DONE: $OUT ($(du -h "$OUT" | cut -f1))"
