# Leuwi Panjang — Android APK

SSH + tmux client for a remote dev box (arm64-v8a). Each terminal tab is its own
tmux session, with an on-screen modifier bar, vertical tabs and drag-to-scroll.

## Download — v0.1.10

- **[⬇ leuwipanjang_v0.1.10.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang_v0.1.10.apk)** (versioned)
- **[⬇ leuwipanjang.apk](https://github.com/situkangsayur/leuwi-panjang/raw/apk/leuwipanjang.apk)** (always latest)

~59 MB · `com.situkangsayur.leuwipanjang` · arm64-v8a · minSdk 29

## What's new in 0.1.10

The release that makes the terminal usable with a full TUI like Claude Code.

- **23 columns became 64.** The tab sidebar took a fixed 150 px out of a ~390 px phone
  window, so the grid was 23 columns wide and everything wrapped or was cut off. It
  starts folded now (the ▌ button toggles it, and the choice is saved), and **A− / A+**
  on the key bar change the font size — there was no control for it at all before.
- **⇧Tab, PgUp and PgDn.** Shift+Tab is how Claude Code moves backwards through its
  options and the Android keyboard cannot produce it. The key bar is two rows so they
  fit.
- **Scrolling back works.** A drag reported wheel notches that tmux discarded unless
  `mouse` was on, so nothing moved; connecting turns it on now. When the program inside
  tmux holds the mouse itself, the new **Gulir** key enters tmux copy-mode directly.
- **The cursor stops disappearing.** A sideways swipe carried enough vertical movement
  to push the view off the bottom, and the cursor is only drawn at the bottom. Only
  vertical drags scroll now, and any key returns to the bottom.

## What's new in 0.1.9

- **The command profile actually saves the key you pick.** The Perintah page opened on
  a blank "new profile" form that is prefilled with the same host, port, user and
  session as the real entry — so it looked like the configured profile with only the
  scrolled-off name empty. Pressing Simpan failed with a complaint rendered below the
  fold, nothing was saved, and connecting kept using the old key. The page now opens on
  the first profile, the result appears directly under Simpan, a blank name is derived
  from the connect word, and an unknown key name warns instead of discarding the whole
  profile.
- **Generate keypair reports where you pressed it.** Its result used to go only to a
  label at the very bottom of the page, so the button looked inert. Keys are also
  written to the first directory that can actually be written to, and the six-key cap
  no longer blocks regenerating a key whose file went missing with the old phone.
- **Public key export**: *Salin public key* to the clipboard, or
  *Ekspor public key (.txt)* into the import folder to share through a file manager.

## What's new in 0.1.8

- **SSH keys survive changing phones.** The key path is stored in full in
  `config.toml`, so a config that came across from another phone still named that
  phone's directory and connecting failed with *"No such file or directory"* even
  when the key itself had been copied over. Keys are now re-found by file name, and
  the config repairs itself when the app starts. The key directory is fixed as well
  (it used to follow whatever directory the default key happened to be in, including
  `/sdcard`), and a phone running the app under a work profile or a second user is
  read correctly instead of being looked for under `/data/user/0`.
- **The Identity field accepts what you type** — an identity name, a file name, or a
  path. A name it did not recognise used to fall through to the default key, so
  setting the new key's name silently authenticated with a different one.
- **`keys`** (new) lists every key with `ok`/`HILANG` per file, the key directory, and
  which key each connect word resolves to. The first thing to run after moving phones.
- **`keygen <name>`** makes a named key *and* registers it as an identity, so the name
  at the prompt is the name a command profile points at. It no longer overwrites an
  existing key without `-f` — doing so used to throw away a key that was already
  installed in `authorized_keys`, leaving no way back in from the phone.
  **`pubkey <name>`** re-copies any of them.

## What's new in 0.1.6 – 0.1.7

These two were never uploaded here; both are included in 0.1.8.

- **`bash` 5.2.21, `git` 2.43.0 and busybox (241 commands) are bundled in the APK** —
  grep, sed, awk, find, tar, vi, wget, less and the rest run on the phone itself, with
  no Termux and no network. The shell stays Android's own `sh`; type `bash` for bash.
- **`cd` and `pwd` work.** Local commands used to run from `/`, which an app may not
  read, so a plain `ls` always failed.

## What's new in 0.1.5

- **Sessions stay connected while the app is in the background.** Android freezes a
  backgrounded app and its sockets die with it, so until now the connection was already
  gone by the time you came back. A foreground service keeps the process running while
  sessions are attached — which is why you now get a permanent notification saying how
  many are connected. It appears only while something is actually connected, and goes
  away when you return to the app or the last session ends.

## What's new in 0.1.4

- **`exit` in tmux returns to the `leuwi>` prompt.** It used to dial the session straight
  back and recreate what you had just closed. Only a connection that actually dropped is
  redialled now; `sambung` brings back one you ended on purpose.

## What's new in 0.1.3

The release that makes the app survive being put down.

- **Sessions come back.** Open tabs are written down and re-attached on launch, and a
  connection the phone drops is dialled again — tmux was still holding the session on the
  server all along; only the app's knowledge of it was lost. The SSH layer also had a
  one-hour inactivity timeout, which by itself killed anything left overnight; liveness is
  now decided by keepalives instead.
- **Command history is kept**, shared across tabs, with a greyed-out completion from past
  commands that Tab or → accepts. It used to live in memory only, and Android kills a
  backgrounded app without warning.
- **No more `62;22c` filling the screen.** Two causes: replies to terminal capability
  questions were being sent late and to the wrong question, and — the bigger one — the
  remote PTY was requested at the desktop config's 115 columns while the phone draws about
  23, so every tmux status redraw wrapped five times.
- **Touch scroll and copy/paste.** A drag scrolls inside tmux (needs `tmux set -g mouse on`
  — arrow keys are deliberately *not* faked, they would type old commands into your shell),
  a long press selects and copies to the Android clipboard on release, and **Tempel** pastes
  from any other app. `keygen` now copies the public key straight to the clipboard, and
  `pubkey` re-copies it later.
- **Bigger touch targets** — tab entries and the tab-list/new/close/settings buttons.
- **7× less CPU when idle** (a full core down to about 14% of one): the terminal only
  repaints when something actually changed.

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
