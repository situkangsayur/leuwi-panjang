#!/usr/bin/env bash
# Re-apply the makepad fork edits the Android build depends on.
#
# They live in the cargo git checkout under ~/.cargo, which is NOT version controlled
# and is wiped by `cargo clean`-adjacent operations, a Cargo.lock bump, or a fresh
# clone on another machine. Without them the APK either crashes on launch, renders a
# black screen, has no app icon, or fails to compile (see
# docs/mobile/03-makepad-android-patches.md for what each hunk is for).
#
#   ./install/apply-makepad-patches.sh          # apply
#   ./install/apply-makepad-patches.sh --check   # report whether they are applied
#
# Cargo treats git dependencies as immutable and will keep using stale artifacts after
# the sources change, so this also drops the compiled makepad-platform.
set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
CRATE_DIR="$(cd "$HERE/.." && pwd)"
PATCH="$HERE/makepad-patches/makepad-android-ff9048c.patch"
RES="$HERE/makepad-patches/res"

CHECKOUT="$(ls -d "$HOME"/.cargo/git/checkouts/makepad-*/*/ 2>/dev/null | head -1 || true)"
[ -n "$CHECKOUT" ] || { echo "ERROR: no makepad checkout under ~/.cargo/git/checkouts"; exit 1; }
CHECKOUT="${CHECKOUT%/}"
echo ">> checkout: $CHECKOUT"

cd "$CHECKOUT"
if [ "${1:-}" = "--check" ]; then
  if git apply --reverse --check "$PATCH" >/dev/null 2>&1; then
    echo ">> patches ARE applied"
    exit 0
  fi
  echo ">> patches are NOT (fully) applied - run without --check"
  exit 1
fi

# Already applied is success, not an error: the script is safe to re-run.
if git apply --reverse --check "$PATCH" >/dev/null 2>&1; then
  echo ">> source patches already applied"
else
  git apply "$PATCH"
  echo ">> source patches applied"
fi

# App icon + context menu: untracked files, so they travel as a copy rather than a diff.
mkdir -p "$CHECKOUT/tools/cargo_makepad/src/android/res"
cp -r "$RES/." "$CHECKOUT/tools/cargo_makepad/src/android/res/"
echo ">> android resources copied"

# Cargo assumes a git dependency never changes; without this the next build links the
# makepad-platform compiled BEFORE the patch and fails on the new JNI functions.
cd "$CRATE_DIR"
# Every target/profile pair, not a hand-kept list: forgetting one (x86_64 release, for
# the emulator) fails as "cannot find function <the one just added>", which reads like a
# source problem and is not.
for target in "" "aarch64-linux-android" "x86_64-linux-android"; do
  for profile in "" "--release"; do
    tgt=""
    [ -n "$target" ] && tgt="--target $target"
    # shellcheck disable=SC2086
    cargo clean -p makepad-platform $profile $tgt >/dev/null 2>&1 || true
  done
done
echo ">> stale makepad-platform artifacts cleared"
echo ">> DONE - now run ./install/build-release.sh"
