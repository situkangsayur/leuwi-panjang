#!/usr/bin/env bash
# Build the static aarch64 binaries that ship inside the APK: busybox, bash, git.
#
# Why bundle them at all: Android's userland is toybox + mksh and the platform has no
# package manager, so `bash`, `git` and ~200 other standard commands simply do not exist
# on the device and there is nothing to install them with.
#
# Why they go in lib/<abi>/ (done by build-release.sh, not here): that is the ONLY place
# an app targeting a modern SDK may execute a binary from. The installer extracts it
# read-only and system-owned, which escapes the W^X rule; the same binary written into
# the app's own storage gives "Permission denied" (verified on a real phone). That is
# also why a download-and-install package manager cannot work without dropping the whole
# app to targetSdk 28, the way Termux does.
#
#   ./install/build-userland.sh            # build everything into install/userland/aarch64
#   ./install/build-userland.sh busybox    # just one
#
# Needs: the full NDK (see docs/mobile/02-android-build.md) and network access.
set -euo pipefail

NDK_ROOT="${LEUWI_NDK:-$HOME/android-ndk-r26d}"
TC="$NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64"
API=29
CRATE_DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$CRATE_DIR/install/userland/aarch64"
WORK="${LEUWI_USERLAND_WORK:-/tmp/leuwi-userland}"

BUSYBOX_V=1.36.1
BASH_V=5.2.21
NCURSES_V=6.4
ZLIB_V=1.3.1
GIT_V=2.43.0

export CC="$TC/bin/aarch64-linux-android$API-clang"
export CXX="$TC/bin/aarch64-linux-android$API-clang++"
export AR="$TC/bin/llvm-ar"
export RANLIB="$TC/bin/llvm-ranlib"
export STRIP="$TC/bin/llvm-strip"
[ -x "$CC" ] || { echo "ERROR: no NDK clang at $CC (set LEUWI_NDK)"; exit 1; }

mkdir -p "$WORK" "$OUT" "$WORK/prefix"
PREFIX="$WORK/prefix"
want() { [ $# -eq 0 ] || [ -z "${TARGETS:-}" ] || grep -qw "$1" <<<"$TARGETS"; }
TARGETS="${*:-}"

fetch() { # url file
  [ -f "$WORK/$2" ] || curl -fsSL --max-time 600 -o "$WORK/$2" "$1"
}

build_busybox() {
  echo ">> busybox $BUSYBOX_V"
  fetch "https://busybox.net/downloads/busybox-$BUSYBOX_V.tar.bz2" "busybox.tar.bz2"
  cd "$WORK" && rm -rf "busybox-$BUSYBOX_V" && tar xf busybox.tar.bz2 && cd "busybox-$BUSYBOX_V"
  make defconfig >/dev/null
  python3 - <<'PY'
import re
cfg = open('.config').read()
# Applets bionic cannot support (no shadow/utmp/crypt/NSS), ones that need a real init
# system, and ones whose headers collide with bionic's (in6_ifreq, semun).
off = """CONFIG_FEATURE_UTMP CONFIG_FEATURE_WTMP CONFIG_LOGIN CONFIG_SU CONFIG_PASSWD
CONFIG_ADDUSER CONFIG_DELUSER CONFIG_ADDGROUP CONFIG_DELGROUP CONFIG_GETTY CONFIG_SULOGIN
CONFIG_VLOCK CONFIG_CHPASSWD CONFIG_CRYPTPW CONFIG_MKPASSWD CONFIG_INETD CONFIG_TELNETD
CONFIG_FTPD CONFIG_HTTPD CONFIG_UDHCPD CONFIG_UDHCPC CONFIG_DNSD CONFIG_NTPD
CONFIG_SYSLOGD CONFIG_KLOGD CONFIG_LOGGER CONFIG_INIT CONFIG_HALT CONFIG_POWEROFF
CONFIG_REBOOT CONFIG_RUNLEVEL CONFIG_MDEV CONFIG_MOUNT CONFIG_UMOUNT CONFIG_SWAPON
CONFIG_SWAPOFF CONFIG_LOSETUP CONFIG_MODPROBE CONFIG_INSMOD CONFIG_RMMOD CONFIG_LSMOD
CONFIG_DEPMOD CONFIG_PIVOT_ROOT CONFIG_SWITCH_ROOT CONFIG_SELINUX CONFIG_NOLOGIN
CONFIG_TCPSVD CONFIG_UDPSVD CONFIG_IFUPDOWN CONFIG_IFPLUGD CONFIG_ARPING
CONFIG_FSCK_MINIX CONFIG_MKFS_MINIX CONFIG_MKFS_EXT2 CONFIG_MKFS_VFAT CONFIG_FSCK
CONFIG_RUNSV CONFIG_RUNSVDIR CONFIG_SV CONFIG_SVC CONFIG_SVOK CONFIG_SVLOGD
CONFIG_CHPST CONFIG_SETUIDGID CONFIG_ENVUIDGID CONFIG_ENVDIR CONFIG_SOFTLIMIT
CONFIG_MAKEMIME CONFIG_REFORMIME CONFIG_POPMAILDIR CONFIG_SENDMAIL CONFIG_FBSPLASH
CONFIG_FBSET CONFIG_SETFONT CONFIG_LOADFONT CONFIG_LOADKMAP CONFIG_OPENVT
CONFIG_DEALLOCVT CONFIG_CHVT CONFIG_SETLOGCONS CONFIG_SETKEYCODES CONFIG_KBD_MODE
CONFIG_ACPID CONFIG_BLKID CONFIG_BLOCKDEV CONFIG_UEVENT CONFIG_WATCHDOG CONFIG_HWCLOCK
CONFIG_RTCWAKE CONFIG_NANDWRITE CONFIG_NANDDUMP CONFIG_UBIATTACH CONFIG_UBIDETACH
CONFIG_UBIMKVOL CONFIG_UBIRMVOL CONFIG_UBIRSVOL CONFIG_UBIUPDATEVOL CONFIG_FLASHCP
CONFIG_FLASH_ERASEALL CONFIG_FLASH_LOCK CONFIG_FLASH_UNLOCK CONFIG_FEATURE_SYSTEMD
CONFIG_INOTIFYD CONFIG_LAST CONFIG_LSSCSI CONFIG_MODINFO CONFIG_SETARCH CONFIG_LINUX32
CONFIG_LINUX64 CONFIG_NSENTER CONFIG_UNSHARE CONFIG_FSFREEZE CONFIG_FATATTR
CONFIG_SETSERIAL CONFIG_TC CONFIG_BRCTL CONFIG_NAMEIF CONFIG_SLATTACH CONFIG_VCONFIG
CONFIG_ZCIP CONFIG_CROND CONFIG_CRONTAB CONFIG_RUN_INIT CONFIG_CHROOT CONFIG_RDATE
CONFIG_RDEV CONFIG_READPROFILE CONFIG_SCRIPT CONFIG_SCRIPTREPLAY CONFIG_LSPCI
CONFIG_LSUSB CONFIG_I2CGET CONFIG_I2CSET CONFIG_I2CDUMP CONFIG_I2CDETECT
CONFIG_I2CTRANSFER CONFIG_DEVMEM CONFIG_FREERAMDISK CONFIG_MKSWAP CONFIG_RAIDAUTORUN
CONFIG_TUNCTL CONFIG_ETHER_WAKE CONFIG_ADJTIMEX CONFIG_CONSPY CONFIG_HOSTID
CONFIG_IFCONFIG CONFIG_ROUTE CONFIG_NETSTAT CONFIG_IPADDR CONFIG_IPLINK CONFIG_IPROUTE
CONFIG_IPRULE CONFIG_IPTUNNEL CONFIG_IPNEIGH CONFIG_IP CONFIG_IPCRM CONFIG_IPCS
CONFIG_MII_TOOL CONFIG_ARP CONFIG_IFENSLAVE CONFIG_SHELL_HUSH""".split()
for sym in off:
    cfg = re.sub(r'^%s=y$' % re.escape(sym), '# %s is not set' % sym, cfg, flags=re.M)
# hush needs sigisemptyset, which bionic does not have.
cfg = re.sub(r'^CONFIG_HUSH_([A-Z_0-9]*)=y$', r'# CONFIG_HUSH_\1 is not set', cfg, flags=re.M)
# sendfile() is killed by the app sandbox's seccomp filter (SIGSYS, "Bad system call"),
# which took out every applet that copies a stream — i.e. anything in a pipeline.
cfg = re.sub(r'^CONFIG_FEATURE_USE_SENDFILE=y$',
             '# CONFIG_FEATURE_USE_SENDFILE is not set', cfg, flags=re.M)
cfg = re.sub(r'^# CONFIG_STATIC is not set$', 'CONFIG_STATIC=y', cfg, flags=re.M)
open('.config','w').write(cfg)
PY
  # busybox still carries replacements for functions modern bionic has, which collide at
  # static link time as "duplicate symbol".
  python3 - <<'PY'
p='include/platform.h'; s=open(p).read()
s = s.replace("# undef HAVE_MEMPCPY\n# undef HAVE_STRCHRNUL",
              "# undef HAVE_MEMPCPY\n# if __ANDROID_API__ < 24\n#  undef HAVE_STRCHRNUL\n# endif")
open(p,'w').write(s)
p='libbb/missing_syscalls.c'; s=open(p).read()
s = s.replace("#if defined(ANDROID) || defined(__ANDROID__)",
              "#if (defined(ANDROID) || defined(__ANDROID__)) && __ANDROID_API__ < 24", 1)
open(p,'w').write(s)
PY
  # bionic keeps the resolver in libc; there is no libresolv to link against.
  sed -i 's/^LDLIBS += resolv/#LDLIBS += resolv/' Makefile.flags
  yes "" | make oldconfig >/dev/null 2>&1
  make -j"$(nproc)" CC="$CC" AR="$AR" STRIP="$STRIP" HOSTCC=cc >/dev/null
  cp busybox "$OUT/busybox"
  echo "   -> $(du -h "$OUT/busybox" | cut -f1), $(./busybox --list | wc -l) applets"
}

build_ncurses() { # bash's readline needs termcap, which bionic has no version of
  [ -f "$PREFIX/lib/libncurses.a" ] && return 0
  echo ">> ncurses $NCURSES_V (for bash/readline termcap)"
  fetch "https://ftp.gnu.org/gnu/ncurses/ncurses-$NCURSES_V.tar.gz" "ncurses.tar.gz"
  cd "$WORK" && rm -rf "ncurses-$NCURSES_V" && tar xf ncurses.tar.gz && cd "ncurses-$NCURSES_V"
  CFLAGS="-Os -fPIC" ./configure --host=aarch64-linux-android --prefix="$PREFIX" \
    --without-shared --without-debug --without-ada --without-cxx-binding \
    --without-manpages --without-progs --without-tests --disable-db-install \
    --enable-termcap --with-fallbacks=xterm,xterm-256color,linux,vt100,dumb \
    --disable-stripping >/dev/null
  make -j"$(nproc)" >/dev/null && make install >/dev/null
}

build_zlib() {
  [ -f "$PREFIX/lib/libz.a" ] && return 0
  echo ">> zlib $ZLIB_V (for git)"
  fetch "https://github.com/madler/zlib/releases/download/v$ZLIB_V/zlib-$ZLIB_V.tar.gz" "zlib.tar.gz"
  cd "$WORK" && rm -rf "zlib-$ZLIB_V" && tar xf zlib.tar.gz && cd "zlib-$ZLIB_V"
  CFLAGS="-Os -fPIC" ./configure --static --prefix="$PREFIX" >/dev/null
  make -j"$(nproc)" >/dev/null && make install >/dev/null
}

build_bash() {
  build_ncurses
  echo ">> bash $BASH_V"
  fetch "https://ftp.gnu.org/gnu/bash/bash-$BASH_V.tar.gz" "bash.tar.gz"
  cd "$WORK" && rm -rf "bash-$BASH_V" && tar xf bash.tar.gz && cd "bash-$BASH_V"
  # The bash_cv_* answers cannot be probed when cross-compiling, so they are supplied.
  CFLAGS="-Os -static -I$PREFIX/include -I$PREFIX/include/ncurses" \
  LDFLAGS="-static -L$PREFIX/lib" \
  ./configure --host=aarch64-linux-android --without-bash-malloc --disable-nls \
    --enable-static-link \
    bash_cv_getcwd_malloc=yes bash_cv_job_control_missing=present \
    bash_cv_sys_named_pipes=present bash_cv_func_sigsetjmp=present \
    bash_cv_printf_a_format=yes bash_cv_unusable_rtsigs=no bash_cv_wcwidth_broken=no \
    bash_cv_dev_fd=standard bash_cv_termcap_lib=libncurses \
    ac_cv_func_strtoimax=yes ac_cv_func_strtoumax=yes >/dev/null
  # configure still queues its own strtoimax, which bionic also defines.
  sed -i 's|${LIBOBJDIR}strtoimax$U\.o||g' lib/sh/Makefile
  rm -f lib/sh/strtoimax.o lib/sh/libsh.a
  make -j"$(nproc)" >/dev/null
  "$STRIP" bash && cp bash "$OUT/bash"
  echo "   -> $(du -h "$OUT/bash" | cut -f1)"
}

build_git() {
  build_zlib
  echo ">> git $GIT_V (local repositories; no network transports)"
  fetch "https://mirrors.edge.kernel.org/pub/software/scm/git/git-$GIT_V.tar.gz" "git.tar.gz"
  cd "$WORK" && rm -rf "git-$GIT_V" && tar xf git.tar.gz && cd "git-$GIT_V"
  # NO_PTHREADS: bionic has no pthread cancellation, which git's run-command.c uses.
  # NEEDS_LIBRT empty: bionic keeps clock_gettime in libc.
  # No curl/openssl: https would drag in a TLS stack; clone/fetch belong on the server.
  make -j"$(nproc)" \
    uname_S=Linux uname_O=Linux NO_PTHREADS=YesPlease NEEDS_LIBRT= \
    NO_GETTEXT=YesPlease NO_CURL=YesPlease NO_OPENSSL=YesPlease BLK_SHA1=YesPlease \
    NO_ICONV=YesPlease NO_TCLTK=YesPlease NO_PERL=YesPlease NO_PYTHON=YesPlease \
    NO_EXPAT=YesPlease NO_REGEX=YesPlease NO_INSTALL_HARDLINKS=YesPlease \
    ZLIB_PATH="$PREFIX" \
    CFLAGS="-Os -static -I$PREFIX/include" LDFLAGS="-static -L$PREFIX/lib" \
    git >/dev/null
  "$STRIP" git && cp git "$OUT/git"
  echo "   -> $(du -h "$OUT/git" | cut -f1)"
}

want busybox && build_busybox
want bash    && build_bash
want git     && build_git

echo
echo ">> DONE: $OUT"
ls -la "$OUT"
echo ">> now run ./install/build-release.sh — it bundles these into the APK"
