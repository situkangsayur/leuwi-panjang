// ── SSH backend (milestone B) ──────────────────────────────
//
// Pure-Rust SSH (russh) so Leuwi Panjang can reach the nvgpu GPU server the same
// way `nvgpu-s <session>` does from the desktop: SSH in and `tmux new -A -s <name>`
// (attach the session if it exists, otherwise create it). On Android this is the
// only backend (no local shell); on desktop it powers SSH tabs and the headless
// `ssh-smoke` self-test.
//
// The module is deliberately UI-agnostic: `spawn` streams remote bytes to a caller
// supplied `FnMut(&[u8])` sink, so the same code drives a real `TermGrid` in the
// GUI and a plain stdout writer in the smoke test.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use russh::client;
use russh::keys::{load_secret_key, PrivateKeyWithHashAlg};
use russh::ChannelMsg;
use tokio::sync::mpsc;

/// A saved connection, mirroring the desktop `nvgpu` SSH host family.
#[derive(Clone, Debug)]
pub struct SshProfile {
    pub label: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_path: PathBuf,
    /// Password auth. Empty means use `key_path` instead.
    pub password: String,
    /// tmux session name for `tmux new -A -s <session>` (attach-or-create).
    pub session: String,
    /// Full remote startup command. When `None`, defaults to
    /// `tmux new -A -s <session>`. Set this to open tmux's session picker on
    /// connect, e.g. `tmux new -A -s main \; choose-tree -Zs`.
    pub startup: Option<String>,
    /// TOFU host-key store (ssh-style pinning of the server's public key).
    pub known_hosts: PathBuf,
    pub cols: u16,
    pub rows: u16,
}

impl SshProfile {
    /// Embedded profile for the nvgpu GPU server (matches the `nvgpu-s` fish helper).
    pub fn nvgpu(session: &str, cols: u16, rows: u16) -> Self {
        Self {
            label: "nvgpu".into(),
            host: "10.100.21.22".into(),
            port: 1313,
            user: "hendri".into(),
            key_path: default_key_path(),
            password: String::new(),
            session: if session.is_empty() { "main".into() } else { session.into() },
            startup: None,
            known_hosts: default_known_hosts(),
            cols,
            rows,
        }
    }

    /// Remote command to run on connect: an explicit `startup` override, else
    /// tmux attach-or-create for the configured session (the `nvgpu-s` flow).
    fn startup_cmd(&self) -> String {
        match &self.startup {
            Some(s) if !s.trim().is_empty() => s.clone(),
            // `set -g mouse on` first, so it applies to the session this command is
            // about to create *and* to one that already exists. Without it a finger
            // drag sends wheel events that tmux throws away, which is what "cannot
            // scroll up" on the phone actually was — the app was reporting the wheel
            // correctly and nothing on the other end was listening.
            _ => format!(
                "tmux set -g mouse on \\; new -A -s {}",
                quote_session(&self.session)
            ),
        }
    }

    /// Build a profile from `LEUWI_SSH_*` env vars, defaulting to the nvgpu profile.
    /// Used by the `ssh-smoke` self-test to target `localhost:1313` during dev.
    pub fn from_env(session: &str, cols: u16, rows: u16) -> Self {
        let mut p = Self::nvgpu(session, cols, rows);
        if let Ok(v) = std::env::var("LEUWI_SSH_HOST") { p.host = v; }
        if let Ok(v) = std::env::var("LEUWI_SSH_PORT") { if let Ok(n) = v.parse() { p.port = n; } }
        if let Ok(v) = std::env::var("LEUWI_SSH_USER") { p.user = v; }
        if let Ok(v) = std::env::var("LEUWI_SSH_KEY") { p.key_path = PathBuf::from(v); }
        if let Ok(v) = std::env::var("LEUWI_SSH_SESSION") { if !v.is_empty() { p.session = v; } }
        if let Ok(v) = std::env::var("LEUWI_SSH_STARTUP") { if !v.trim().is_empty() { p.startup = Some(v); } }
        if let Ok(v) = std::env::var("LEUWI_SSH_KNOWN_HOSTS") { p.known_hosts = PathBuf::from(v); }
        p
    }
}

// Android has no home directory: `dirs::home_dir()` comes back empty and the process CWD
// is `/`, so the desktop defaults below would resolve to an unwritable `/.ssh/id_rsa` and
// `/leuwi-panjang/known_hosts`. Anchor both on the app`s own directories instead.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_turns_on_tmux_mouse() {
        // Without this a finger drag reports wheel events that tmux discards, which is
        // what "cannot scroll up" on the phone was. `set` comes first so it applies to
        // a session that already exists, not only the one being created.
        let p = SshProfile::nvgpu("main", 80, 24);
        let cmd = p.startup_cmd();
        assert!(cmd.starts_with("tmux set -g mouse on \\;"), "{cmd}");
        assert!(cmd.ends_with("new -A -s main"), "{cmd}");
    }

    #[test]
    fn test_explicit_startup_is_left_alone() {
        let mut p = SshProfile::nvgpu("main", 80, 24);
        p.startup = Some("tmux new -A -s main \\; choose-tree -Zs".into());
        assert_eq!(p.startup_cmd(), "tmux new -A -s main \\; choose-tree -Zs");
    }
}

#[cfg(target_os = "android")]
pub(crate) const ANDROID_PKG: &str = "com.situkangsayur.leuwipanjang";
#[cfg(target_os = "android")]
const ANDROID_FILES_DIR: &str = "/sdcard/Android/data/com.situkangsayur.leuwipanjang/files";

/// The app`s private data directory. It is *not* always `/data/user/0/<pkg>`: a work
/// profile, a second user on the phone, or a "dual app" clone runs the same package
/// under `/data/user/<n>/<pkg>`, and the hard-coded `0` is then a directory this
/// process cannot even stat. That is one of the ways a path written on the old phone
/// becomes a path that does not exist on the new one. Android derives the user id from
/// the process uid (`uid / 100000`), so derive it the same way.
#[cfg(target_os = "android")]
pub(crate) fn android_data_dir() -> PathBuf {
    let user = unsafe { libc::getuid() } / 100_000;
    for c in [
        format!("/data/user/{user}/{ANDROID_PKG}"),
        format!("/data/data/{ANDROID_PKG}"),
        format!("/data/user/0/{ANDROID_PKG}"),
    ] {
        let p = PathBuf::from(&c);
        if p.is_dir() {
            return p;
        }
    }
    PathBuf::from(format!("/data/user/{user}/{ANDROID_PKG}"))
}

/// The one directory new private keys are written to, and the first place they are
/// looked for. Deliberately fixed: it used to be derived from
/// `default_key_path().parent()`, which meant it moved to /sdcard as soon as some
/// unrelated `id_ed25519` happened to be lying there — so the same key name resolved
/// to different directories on different phones, and the absolute path saved in
/// config.toml pointed at nothing after the config was carried over.
#[cfg(target_os = "android")]
pub(crate) fn key_dir() -> PathBuf {
    android_data_dir().join("ssh")
}

#[cfg(not(target_os = "android"))]
pub(crate) fn key_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/home/hendri"))
        .join(".ssh")
}

/// Every directory a private key may legitimately sit in, most-preferred first. Used
/// both to find the default key and to re-find a key whose stored absolute path no
/// longer resolves (new phone, restored config, app data moved between users).
pub(crate) fn key_dirs() -> Vec<PathBuf> {
    #[cfg(target_os = "android")]
    {
        let mut v = vec![key_dir()];
        for c in [
            ANDROID_FILES_DIR.to_string(),
            format!("/data/user/0/{ANDROID_PKG}/ssh"),
            format!("/sdcard/Android/media/{ANDROID_PKG}/import"),
            format!("{ANDROID_FILES_DIR}/import"),
        ] {
            let p = PathBuf::from(&c);
            if !v.contains(&p) {
                v.push(p);
            }
        }
        v
    }
    #[cfg(not(target_os = "android"))]
    {
        let mut v = vec![key_dir()];
        if let Some(d) = dirs::config_dir().map(|d| d.join("leuwi-panjang/ssh")) {
            v.push(d);
        }
        v
    }
}

/// The key used when nothing names one: the first `id_ed25519`/`id_rsa` that actually
/// exists, searched in `key_dirs()` order, else where one would be created.
pub(crate) fn default_key_path() -> PathBuf {
    for d in key_dirs() {
        for name in ["id_ed25519", "id_rsa"] {
            let p = d.join(name);
            if p.is_file() {
                return p;
            }
        }
    }
    key_dir().join("id_ed25519")
}

/// The first directory in `key_dirs()` we can actually create and write into, else
/// `key_dir()`. Generating a key used to dead-end when its one destination could not
/// be created — on a phone where the app data dir is somewhere else than assumed there
/// was then no way to make a key at all. `probe` is written and removed, because
/// `create_dir_all` succeeding says nothing about being allowed to write inside.
pub(crate) fn writable_key_dir() -> PathBuf {
    for d in key_dirs() {
        if std::fs::create_dir_all(&d).is_err() {
            continue;
        }
        let probe = d.join(".leuwi-write-test");
        if std::fs::write(&probe, b"").is_ok() {
            let _ = std::fs::remove_file(&probe);
            return d;
        }
    }
    key_dir()
}

/// Turn a key path recorded in config.toml into one that exists on *this* device.
/// The recorded path is absolute, so a config that travels to another phone (cloud
/// restore, an imported config.toml, the app reinstalled under a different Android
/// user) keeps pointing at the old device`s directory. Connecting then failed with
/// "No such file or directory" even when the key file itself had been copied over, so
/// re-find it by file name before giving up. Falling back to `key_dir()` rather than
/// the stale path means the error message names the place the key should be put.
pub(crate) fn resolve_key_path(stored: &Path) -> PathBuf {
    if stored.as_os_str().is_empty() {
        return default_key_path();
    }
    if stored.is_file() {
        return stored.to_path_buf();
    }
    let Some(name) = stored.file_name() else {
        return stored.to_path_buf();
    };
    for d in key_dirs() {
        let p = d.join(name);
        if p.is_file() {
            return p;
        }
    }
    key_dir().join(name)
}

/// The public half of an existing key, in `authorized_keys` form. Reads the cached
/// `.pub` when it is there and otherwise derives it from the private key, so a key
/// provisioned by hand (adb push, file import) still yields something to paste.
pub(crate) fn public_key_of(path: &Path) -> Result<String, String> {
    if let Ok(text) = std::fs::read_to_string(path.with_extension("pub")) {
        let line = text.trim();
        if !line.is_empty() {
            return Ok(line.to_string());
        }
    }
    let key = russh::keys::load_secret_key(path, None)
        .map_err(|e| format!("baca {}: {e}", path.display()))?;
    key.public_key()
        .to_openssh()
        .map_err(|e| format!("encode public key: {e}"))
}

/// Generate a fresh ed25519 keypair at `path` (replacing any existing one) and return
/// the public key line to install in the server's `authorized_keys`. Used by the built-in
/// `keygen` command so a phone can be enrolled without going through a laptop.
///
/// The seed is read straight from `/dev/urandom` rather than going through a rand_core
/// `OsRng`: ssh-key and russh pull in different rand_core majors, so the trait bounds
/// don't line up, and a 32-byte seed needs no RNG plumbing at all.
pub(crate) fn generate_key(path: &Path, comment: &str) -> Result<String, String> {
    use russh::keys::ssh_key::private::Ed25519Keypair;
    use russh::keys::ssh_key::LineEnding;
    use russh::keys::PrivateKey;

    let mut seed = [0u8; 32];
    {
        use std::io::Read;
        let mut f = std::fs::File::open("/dev/urandom")
            .map_err(|e| format!("open /dev/urandom: {e}"))?;
        f.read_exact(&mut seed)
            .map_err(|e| format!("read /dev/urandom: {e}"))?;
    }

    let mut key = PrivateKey::from(Ed25519Keypair::from_seed(&seed));
    key.set_comment(comment);

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    let pem = key
        .to_openssh(LineEnding::LF)
        .map_err(|e| format!("encode key: {e}"))?;
    std::fs::write(path, pem.as_bytes()).map_err(|e| format!("write {}: {e}", path.display()))?;
    // Best effort: keep the private key unreadable by other apps.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }

    let pubkey = key
        .public_key()
        .to_openssh()
        .map_err(|e| format!("encode public key: {e}"))?;
    let _ = std::fs::write(path.with_extension("pub"), format!("{pubkey}\n"));
    Ok(pubkey)
}

/// Store a keypair the user pasted into the config form. The private half is parsed
/// before it is written so a truncated paste fails here, with a message, rather than
/// at connect time as an opaque auth failure. Returns the public key line: the pasted
/// one if given, otherwise derived from the private key.
pub(crate) fn import_key(path: &Path, private_pem: &str, public_line: &str) -> Result<String, String> {
    use russh::keys::PrivateKey;

    let pem = private_pem.trim();
    if pem.is_empty() {
        return Err("private key kosong".into());
    }
    // Parsing validates the whole PEM; an encrypted key has no passphrase prompt here,
    // so reject it now instead of failing later inside the SSH handshake.
    let key = PrivateKey::from_openssh(pem)
        .map_err(|e| format!("private key tidak valid: {e}"))?;
    if key.is_encrypted() {
        return Err("private key terenkripsi (passphrase) belum didukung".into());
    }

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    // OpenSSH refuses keys without a trailing newline.
    std::fs::write(path, format!("{pem}\n")).map_err(|e| format!("write {}: {e}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }

    let pubkey = if public_line.trim().is_empty() {
        key.public_key().to_openssh().map_err(|e| format!("encode public key: {e}"))?
    } else {
        public_line.trim().to_string()
    };
    let _ = std::fs::write(path.with_extension("pub"), format!("{pubkey}\n"));
    Ok(pubkey)
}

#[cfg(target_os = "android")]
pub(crate) fn default_known_hosts() -> PathBuf {
    android_data_dir().join("known_hosts")
}

#[cfg(not(target_os = "android"))]
fn default_known_hosts() -> PathBuf {
    dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
        .join("leuwi-panjang/known_hosts")
}

// ── TOFU host-key store ────────────────────────────────────
// Simple `known_hosts`-style file: one `host:port SHA256:...` line per server.
fn known_hosts_lookup(path: &Path, host_port: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let mut it = line.split_whitespace();
        if let (Some(h), Some(fp)) = (it.next(), it.next()) {
            if h == host_port {
                return Some(fp.to_string());
            }
        }
    }
    None
}

fn known_hosts_append(path: &Path, host_port: &str, fp: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(f, "{} {}", host_port, fp)
}

/// tmux-safe quoting for a session name (letters/digits/-/_/. are safe as-is).
fn quote_session(s: &str) -> String {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

// ── russh client handler ───────────────────────────────────
// TOFU host-key verification: pin the server key on first connect, then require
// it to match on every later connect (ssh's trust-on-first-use). A mismatch —
// the classic MITM signal — aborts the connection; the reason is surfaced to the
// user through `reason` (check_server_key has no access to the terminal sink).
struct Client {
    host_port: String,
    known_hosts: PathBuf,
    reason: Arc<Mutex<Option<String>>>,
}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        let fp = server_public_key
            .fingerprint(russh::keys::ssh_key::HashAlg::Sha256)
            .to_string();
        match known_hosts_lookup(&self.known_hosts, &self.host_port) {
            Some(stored) if stored == fp => Ok(true),
            Some(stored) => {
                *self.reason.lock().unwrap() = Some(format!(
                    "PERINGATAN: host key {} BERUBAH — kemungkinan MITM!\r\n  \
                     tersimpan: {}\r\n  sekarang : {}\r\n  \
                     hapus baris '{}' di {} jika perubahan ini memang sah.",
                    self.host_port, stored, fp, self.host_port, self.known_hosts.display()
                ));
                Ok(false)
            }
            None => {
                // First contact: pin it. (Best-effort persist; in-memory otherwise.)
                let _ = known_hosts_append(&self.known_hosts, &self.host_port, &fp);
                Ok(true)
            }
        }
    }
}

// ── shared connect + authenticate ──────────────────────────
async fn connect_auth(p: &SshProfile) -> Result<client::Handle<Client>, String> {
    let config = Arc::new(client::Config {
        // No inactivity timeout: a session left open overnight is the normal case here
        // (that is what tmux is for), and dropping it after an hour of silence is
        // exactly the "came back and everything was gone" failure. Liveness is decided
        // by keepalives instead, which say something about the *link* rather than about
        // how recently the user typed.
        inactivity_timeout: None,
        // Three unanswered keepalives ≈ 45 s to notice a dead link — quick enough that
        // the tab reconnects itself soon after the phone wakes, slow enough to ride out
        // a mobile network hiccup.
        keepalive_interval: Some(Duration::from_secs(15)),
        keepalive_max: 3,
        ..Default::default()
    });

    let reason = Arc::new(Mutex::new(None));
    let client = Client {
        host_port: format!("{}:{}", p.host, p.port),
        known_hosts: p.known_hosts.clone(),
        reason: reason.clone(),
    };

    let mut handle = client::connect(config, (p.host.as_str(), p.port), client)
        .await
        .map_err(|e| {
            // Prefer the specific host-key reason over russh's generic error.
            reason
                .lock()
                .unwrap()
                .take()
                .unwrap_or_else(|| format!("connect {}:{} failed: {}", p.host, p.port, e))
        })?;

    // A password in the profile wins; otherwise authenticate with the key.
    let auth = if !p.password.is_empty() {
        handle
            .authenticate_password(&p.user, &p.password)
            .await
            .map_err(|e| format!("auth error: {}", e))?
    } else {
        // Re-resolve here too: a profile can be built from a config that was written
        // on another device, and the key may well be present under a different
        // directory on this one.
        let key_path = resolve_key_path(&p.key_path);
        if !key_path.is_file() {
            return Err(format!(
                "key tidak ada: {}\r\n  buat yang baru dengan `keygen <nama>`, \
                 atau salin key lama ke folder itu",
                key_path.display()
            ));
        }
        let key = load_secret_key(&key_path, None)
            .map_err(|e| format!("load key {}: {}", key_path.display(), e))?;
        let hash = handle
            .best_supported_rsa_hash()
            .await
            .map_err(|e| format!("rsa hash negotiation: {}", e))?
            .flatten();
        let key = PrivateKeyWithHashAlg::new(Arc::new(key), hash);
        handle
            .authenticate_publickey(&p.user, key)
            .await
            .map_err(|e| format!("auth error: {}", e))?
    };
    if !auth.success() {
        return Err(format!("authentication failed for {}@{}", p.user, p.host));
    }
    Ok(handle)
}

// ── one-shot: list tmux sessions ───────────────────────────
/// Connect, run `tmux ls`, and return its stdout (blocking; spins its own runtime).
/// Mirrors `nvgpu-ls`. Returns a friendly message when the server has no sessions.
pub fn list_sessions(p: &SshProfile) -> io::Result<String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(async move {
        let handle = connect_auth(p).await.map_err(other)?;
        let mut channel = handle.channel_open_session().await.map_err(other)?;
        // `tmux ls` exits non-zero with "no server running" when nothing is up.
        channel
            .exec(true, "tmux list-sessions 2>/dev/null || true")
            .await
            .map_err(other)?;
        let mut out = Vec::new();
        loop {
            match channel.wait().await {
                Some(ChannelMsg::Data { data }) => out.extend_from_slice(&data),
                Some(ChannelMsg::ExtendedData { data, .. }) => out.extend_from_slice(&data),
                Some(ChannelMsg::Eof) | None => break,
                _ => {}
            }
        }
        let s = String::from_utf8_lossy(&out).trim().to_string();
        Ok(if s.is_empty() { "(no tmux sessions)".to_string() } else { s })
    })
}

// ── interactive: attach a PTY session ──────────────────────
enum InMsg {
    Data(Vec<u8>),
    Resize(u16, u16),
    Close,
}

/// Live handle to a running SSH PTY session. Cloneable writer feeds keystrokes to
/// the remote; `resize` forwards SIGWINCH; dropping/close tears the channel down.
pub struct SshHandle {
    tx: mpsc::UnboundedSender<InMsg>,
    /// Set by the worker when the session ends (clean exit or error), so the tab
    /// can drop this handle and return to the built-in prompt.
    done: Arc<AtomicBool>,
    /// Set when the *remote command* reported an exit status — i.e. tmux ended because
    /// someone typed `exit`, not because the link died. A dropped connection never
    /// delivers one, which is the only trustworthy way to tell the two apart: both look
    /// identical from the outside, and "the session lasted a while" says nothing (a
    /// session you worked in for an hour and then exited looks exactly like a healthy
    /// one that dropped).
    exited: Arc<AtomicBool>,
}

impl SshHandle {
    /// True once the SSH worker has finished — the tab should fall back to the prompt.
    pub fn is_done(&self) -> bool {
        self.done.load(Ordering::Relaxed)
    }
    /// True when the remote command ended by itself (`exit` in tmux). The tab uses this
    /// to decide between "go back to the prompt" and "the phone dropped us, redial".
    pub fn exited_cleanly(&self) -> bool {
        self.exited.load(Ordering::Relaxed)
    }
    pub fn writer(&self) -> SshWriter {
        SshWriter { tx: self.tx.clone() }
    }
    pub fn resize(&self, cols: u16, rows: u16) {
        let _ = self.tx.send(InMsg::Resize(cols, rows));
    }
    #[allow(dead_code)]
    pub fn close(&self) {
        let _ = self.tx.send(InMsg::Close);
    }
}

/// `Write` adapter: terminal input bytes → SSH channel. Non-blocking (queued).
pub struct SshWriter {
    tx: mpsc::UnboundedSender<InMsg>,
}

impl Write for SshWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = self.tx.send(InMsg::Data(buf.to_vec()));
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Connect in a background thread and stream remote bytes to `sink`. Returns
/// immediately with a handle; connection status is reported through `sink` as
/// styled banner text so the terminal shows progress/errors inline.
pub fn spawn<S>(profile: SshProfile, mut sink: S) -> SshHandle
where
    S: FnMut(&[u8]) + Send + 'static,
{
    let (tx, rx) = mpsc::unbounded_channel::<InMsg>();
    let done = Arc::new(AtomicBool::new(false));
    let done_worker = done.clone();
    let exited = Arc::new(AtomicBool::new(false));
    let exited_worker = exited.clone();
    // Spawn best-effort: never panic the caller (UI thread) if the OS refuses a
    // thread. The whole worker body is caught so an SSH/tokio/ring panic can't
    // escape and abort the process.
    let spawned = std::thread::Builder::new()
        .name("ssh".into())
        .spawn(move || {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
                    Ok(rt) => rt,
                    Err(e) => {
                        sink(format!("\r\n\x1b[31mSSH runtime error: {}\x1b[0m\r\n", e).as_bytes());
                        return;
                    }
                };
                rt.block_on(async move {
                    if let Err(e) = run_session(&profile, &mut sink, rx, &exited_worker).await {
                        sink(format!("\r\n\x1b[31m{}\x1b[0m\r\n", e).as_bytes());
                    }
                });
            }));
            // Whatever happened (ok, error, or caught panic), the session is over.
            done_worker.store(true, Ordering::Relaxed);
        });
    if spawned.is_err() {
        done.store(true, Ordering::Relaxed);
        // Extremely unlikely, but degrade gracefully instead of panicking.
    }
    SshHandle { tx, done, exited }
}

async fn run_session<S>(
    p: &SshProfile,
    sink: &mut S,
    mut rx: mpsc::UnboundedReceiver<InMsg>,
    exited: &AtomicBool,
) -> Result<(), String>
where
    S: FnMut(&[u8]),
{
    sink(format!(
        "\r\n\x1b[2mmenyambung ke \x1b[0m\x1b[1;38;5;39m{}\x1b[0m\x1b[2m ({}@{}:{}) ...\x1b[0m\r\n",
        p.label, p.user, p.host, p.port
    ).as_bytes());

    let handle = connect_auth(p).await?;
    let mut channel = handle.channel_open_session().await.map_err(|e| e.to_string())?;

    channel
        .request_pty(false, "xterm-256color", p.cols as u32, p.rows as u32, 0, 0, &[])
        .await
        .map_err(|e| format!("request pty: {}", e))?;

    let cmd = p.startup_cmd();
    sink(format!("\x1b[2m$ {}\x1b[0m\r\n", cmd).as_bytes());
    channel
        .exec(true, cmd.as_bytes())
        .await
        .map_err(|e| format!("exec tmux: {}", e))?;

    loop {
        tokio::select! {
            msg = channel.wait() => {
                match msg {
                    Some(ChannelMsg::Data { data }) => sink(&data),
                    Some(ChannelMsg::ExtendedData { data, .. }) => sink(&data),
                    Some(ChannelMsg::ExitStatus { exit_status }) => {
                        // The remote command finished on its own terms. Recorded before
                        // the loop ends so the tab knows this was `exit`, not a drop.
                        exited.store(true, Ordering::Relaxed);
                        sink(format!("\r\n\x1b[2m[sesi berakhir, status {}]\x1b[0m\r\n", exit_status).as_bytes());
                    }
                    Some(ChannelMsg::Eof) | None => break,
                    _ => {}
                }
            }
            inbound = rx.recv() => {
                match inbound {
                    Some(InMsg::Data(d)) => {
                        channel.data(&d[..]).await.map_err(|e| format!("write: {}", e))?;
                    }
                    Some(InMsg::Resize(c, r)) => {
                        let _ = channel.window_change(c as u32, r as u32, 0, 0).await;
                    }
                    Some(InMsg::Close) | None => {
                        exited.store(true, Ordering::Relaxed);
                        let _ = channel.eof().await;
                        break;
                    }
                }
            }
        }
    }
    Ok(())
}

fn other<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}

// ── headless self-test (desktop only) ──────────────────────
// Reached via `leuwi-panjang ssh-smoke [list|attach [session]]`. Verifies the
// whole SSH path without the GUI: point it at localhost:1313 during dev with
//   LEUWI_SSH_HOST=localhost LEUWI_SSH_PORT=1313 leuwi-panjang ssh-smoke list
#[cfg(not(target_os = "android"))]
pub fn smoke(sub: &str, session: &str) {
    let profile = SshProfile::from_env(session, 100, 30);
    eprintln!(
        "[ssh-smoke] {} -> {}@{}:{} key={} session={}",
        sub, profile.user, profile.host, profile.port, profile.key_path.display(), profile.session
    );
    match sub {
        "list" => match list_sessions(&profile) {
            Ok(list) => {
                println!("{}", list);
                eprintln!("[ssh-smoke] OK: received {} bytes", list.len());
            }
            Err(e) => {
                eprintln!("[ssh-smoke] FAIL: {}", e);
                std::process::exit(1);
            }
        },
        "attach" => {
            use std::sync::atomic::{AtomicUsize, Ordering};
            let count = Arc::new(AtomicUsize::new(0));
            let c2 = count.clone();
            let handle = spawn(profile, move |bytes| {
                c2.fetch_add(bytes.len(), Ordering::Relaxed);
                let _ = io::stdout().write_all(bytes);
                let _ = io::stdout().flush();
            });
            // Stream for a few seconds, then send a harmless keystroke and quit.
            std::thread::sleep(Duration::from_millis(2500));
            let mut w = handle.writer();
            let _ = w.write_all(b"\x0c"); // Ctrl-L: redraw, proves input path works
            std::thread::sleep(Duration::from_millis(1500));
            handle.close();
            std::thread::sleep(Duration::from_millis(300));
            let total = count.load(Ordering::Relaxed);
            eprintln!("\n[ssh-smoke] attach received {} bytes total", total);
            if total == 0 {
                eprintln!("[ssh-smoke] FAIL: no data from remote");
                std::process::exit(1);
            }
        }
        other => {
            eprintln!("[ssh-smoke] unknown subcommand '{}'; use list|attach", other);
            std::process::exit(2);
        }
    }
}
