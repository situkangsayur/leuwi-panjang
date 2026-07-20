use makepad_widgets::*;
use makepad_widgets::event::TouchState;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};

pub mod ssh;

// ── Clipboard (platform-gated) ─────────────────────────────
// Desktop uses the system clipboard via arboard. Android has no arboard
// backend, so we fall back to a process-local buffer (copy/paste within app).
#[cfg(not(target_os = "android"))]
mod clip {
    use arboard::Clipboard;
    pub fn set(text: &str) { if let Ok(mut cb) = Clipboard::new() { let _ = cb.set_text(text); } }
    pub fn get() -> String {
        Clipboard::new().ok().and_then(|mut cb| cb.get_text().ok()).unwrap_or_default()
    }
}
#[cfg(target_os = "android")]
mod clip {
    use std::sync::Mutex;
    static BUF: Mutex<String> = Mutex::new(String::new());
    pub fn set(text: &str) { if let Ok(mut b) = BUF.lock() { *b = text.to_string(); } }
    pub fn get() -> String { BUF.lock().map(|b| b.clone()).unwrap_or_default() }
}

// ── Theme ──────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
struct Theme {
    #[serde(default = "default_theme_name")]
    name: String,
    #[serde(default = "default_bg")]
    bg: String,
    #[serde(default = "default_fg")]
    fg: String,
    #[serde(default = "default_cursor_color")]
    cursor: String,
    #[serde(default = "default_selection_color")]
    selection: String,
    #[serde(default = "default_status_bg")]
    status_bg: String,
    #[serde(default = "default_status_fg")]
    status_fg: String,
    #[serde(default = "default_tab_bg")]
    tab_bg: String,
    #[serde(default)]
    ansi: Vec<String>,  // 16 ANSI colors
}

fn default_theme_name() -> String { "leuwi-dark".into() }
fn default_cursor_color() -> String { "#58A6FF".into() }
fn default_selection_color() -> String { "#264F78".into() }
fn default_status_bg() -> String { "#181818".into() }
fn default_status_fg() -> String { "#6E7681".into() }
fn default_tab_bg() -> String { "#181818".into() }

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: default_theme_name(),
            bg: default_bg(), fg: default_fg(),
            cursor: default_cursor_color(), selection: default_selection_color(),
            status_bg: default_status_bg(), status_fg: default_status_fg(),
            tab_bg: default_tab_bg(),
            ansi: vec![
                "#141416".into(), "#F24D4D".into(), "#4DBF59".into(), "#D9B840".into(),
                "#669EF2".into(), "#BF73F2".into(), "#59CCD9".into(), "#E0E5F0".into(),
                "#808C9E".into(), "#FF7373".into(), "#66E07A".into(), "#F5DB61".into(),
                "#8CC2FF".into(), "#E0A6FF".into(), "#80EDFB".into(), "#F2F8FF".into(),
            ],
        }
    }
}

impl Theme {
    fn load(name: &str) -> Self {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("leuwi-panjang/themes");
        let path = dir.join(format!("{}.toml", name));
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(theme) = toml::from_str(&content) {
                    return theme;
                }
            }
        }
        // Write default theme if not exists
        let theme = Theme::default();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(s) = toml::to_string_pretty(&theme) {
            let _ = std::fs::write(dir.join("leuwi-dark.toml"), s);
        }
        theme
    }

    fn hex_to_vec4(&self, hex: &str) -> Vec4 {
        parse_hex_color(hex)
    }

    fn ansi_color(&self, idx: u8) -> Vec4 {
        if (idx as usize) < self.ansi.len() {
            parse_hex_color(&self.ansi[idx as usize])
        } else {
            ansi_to_vec4(idx)
        }
    }
}

fn parse_hex_color(hex: &str) -> Vec4 {
    let h = hex.trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        vec4(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
    } else {
        vec4(0.77, 0.79, 0.82, 1.0)
    }
}

// ── Config ─────────────────────────────────────────────────

#[derive(Clone, Serialize, Deserialize)]
struct Config {
    #[serde(default = "default_shell")]
    shell: String,
    #[serde(default = "default_font_size")]
    font_size: f64,
    #[serde(default = "default_cols")]
    cols: usize,
    #[serde(default = "default_rows")]
    rows: usize,
    #[serde(default = "default_scrollback")]
    scrollback: usize,
    #[serde(default = "default_bg")]
    bg_color: String,
    #[serde(default = "default_fg")]
    fg_color: String,
    #[serde(default = "default_prompt")]
    prompt: String,
    #[serde(default = "default_opacity")]
    opacity: f64,
    #[serde(default = "default_cursor_style")]
    cursor_style: String,  // "block" or "beam"
    #[serde(default = "default_cell_width")]
    cell_width: f64,
    #[serde(default = "default_cell_height")]
    cell_height: f64,
    #[serde(default = "default_theme_name")]
    theme: String,
    // ── SSH backend (milestone B) ──
    // On Android these drive the default tab (there is no local shell). On desktop
    // they configure SSH tabs / the ssh-smoke self-test. Defaults = nvgpu profile.
    #[serde(default = "default_ssh_host")]
    ssh_host: String,
    #[serde(default = "default_ssh_port")]
    ssh_port: u16,
    #[serde(default = "default_ssh_user")]
    ssh_user: String,
    #[serde(default = "default_ssh_key")]
    ssh_key: String,
    // Empty means key auth. Stored in the app-private config, not the APK.
    #[serde(default)]
    ssh_password: String,
    // The command words themselves are configurable, so the profile can be renamed
    // without touching code (mirrors the nvgpu-s / nvgpu-ls shell functions).
    #[serde(default = "default_cmd_connect")]
    ssh_cmd_connect: String,
    #[serde(default = "default_cmd_list")]
    ssh_cmd_list: String,
    #[serde(default = "default_ssh_session")]
    ssh_session: String,
    // Optional full startup command; empty = `tmux new -A -s <ssh_session>`.
    // Set to e.g. `tmux new -A -s main \; choose-tree -Zs` for a session picker.
    #[serde(default)]
    ssh_startup: String,
}

fn default_shell() -> String { std::env::var("SHELL").unwrap_or("/bin/zsh".into()) }
fn default_ssh_host() -> String { "10.100.21.22".into() }
fn default_ssh_port() -> u16 { 1313 }
fn default_ssh_user() -> String { "hendri".into() }
fn default_ssh_key() -> String { "~/.ssh/id_rsa".into() }
fn default_ssh_session() -> String { "main".into() }
fn default_cmd_connect() -> String { "nvgpu-s".into() }
fn default_cmd_list() -> String { "nvgpu-ls".into() }
fn default_font_size() -> f64 { 12.0 }
fn default_cols() -> usize { 115 }
fn default_rows() -> usize { 33 }
fn default_scrollback() -> usize { 5000 }
fn default_bg() -> String { "#1E1E1E".into() }
fn default_fg() -> String { "#C5C8C6".into() }
fn default_prompt() -> String { "%n@%m %~ %# ".into() }
fn default_opacity() -> f64 { 0.97 }
fn default_cursor_style() -> String { "block".into() }
fn default_cell_width() -> f64 { 9.2 }
fn default_cell_height() -> f64 { 20.0 }

impl Default for Config {
    fn default() -> Self {
        Self {
            shell: default_shell(), font_size: default_font_size(),
            cols: default_cols(), rows: default_rows(),
            scrollback: default_scrollback(),
            bg_color: default_bg(), fg_color: default_fg(),
            prompt: default_prompt(), opacity: default_opacity(),
            cursor_style: default_cursor_style(),
            cell_width: default_cell_width(), cell_height: default_cell_height(),
            theme: default_theme_name(),
            ssh_host: default_ssh_host(), ssh_port: default_ssh_port(),
            ssh_user: default_ssh_user(), ssh_key: default_ssh_key(),
            ssh_session: default_ssh_session(), ssh_startup: String::new(),
            ssh_password: String::new(),
            ssh_cmd_connect: default_cmd_connect(), ssh_cmd_list: default_cmd_list(),
        }
    }
}

impl Config {
    /// Build an SSH profile from config (used by the Android default tab and
    /// desktop SSH tabs). `~` in the key path is expanded to the home dir.
    fn ssh_profile(&self) -> ssh::SshProfile {
        let key_path = if let Some(rest) = self.ssh_key.strip_prefix("~/") {
            // Android has no home directory, so `~/.ssh/id_rsa` would collapse to a
            // relative path resolved against `/` — unreadable and unwritable. Fall back
            // to the app`s own key locations there.
            #[cfg(target_os = "android")]
            { let _ = rest; ssh::default_key_path() }
            #[cfg(not(target_os = "android"))]
            { dirs::home_dir().unwrap_or_default().join(rest) }
        } else {
            std::path::PathBuf::from(&self.ssh_key)
        };
        ssh::SshProfile {
            label: "nvgpu".into(),
            host: self.ssh_host.clone(),
            port: self.ssh_port,
            user: self.ssh_user.clone(),
            key_path,
            password: self.ssh_password.clone(),
            session: self.ssh_session.clone(),
            startup: if self.ssh_startup.trim().is_empty() { None } else { Some(self.ssh_startup.clone()) },
            known_hosts: {
                #[cfg(target_os = "android")]
                { ssh::default_known_hosts() }
                #[cfg(not(target_os = "android"))]
                { dirs::config_dir()
                    .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
                    .unwrap_or_default()
                    .join("leuwi-panjang/known_hosts") }
            },
            cols: self.cols as u16,
            rows: self.rows as u16,
        }
    }

    /// Per-tab SSH profile for Termius-style multi-session: tab 1 attaches the
    /// configured session; each additional tab gets its own `<session>-<n>` so the
    /// tabs are independent tmux sessions on nvgpu (not mirrors of the same one).
    /// Skipped when `ssh_startup` overrides the command (e.g. a choose-tree picker).
    fn ssh_profile_for_tab(&self, tab_id: usize) -> ssh::SshProfile {
        let mut p = self.ssh_profile();
        if tab_id > 1 && self.ssh_startup.trim().is_empty() {
            p.session = format!("{}-{}", p.session, tab_id);
        }
        p
    }
}

impl Config {
    /// Where the config lives. Android has no XDG config dir (`dirs::config_dir()` is
    /// empty and the CWD is `/`), so anchor it in the app`s own data directory.
    fn config_path() -> std::path::PathBuf {
        #[cfg(target_os = "android")]
        {
            std::path::PathBuf::from("/data/user/0/com.situkangsayur.leuwipanjang/config.toml")
        }
        #[cfg(not(target_os = "android"))]
        {
            dirs::config_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
                .join("leuwi-panjang/config.toml")
        }
    }

    /// Persist the current settings so edits made in the config form survive a restart.
    fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        let toml = toml::to_string_pretty(self).map_err(|e| format!("encode config: {e}"))?;
        std::fs::write(&path, toml).map_err(|e| format!("write {}: {e}", path.display()))
    }

    fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(cfg) = toml::from_str(&content) {
                    return cfg;
                }
            }
        }
        // Write default config if not exists
        let cfg = Config::default();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, toml::to_string_pretty(&cfg).unwrap_or_default());
        cfg
    }
}

// ── Split direction ────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq)]
enum SplitDir { Vertical, Horizontal }

// ── Terminal Tab ───────────────────────────────────────────
#[allow(dead_code)]
struct TermTab {
    grid: Arc<Mutex<TermGrid>>,
    writer: Option<Box<dyn Write + Send>>,
    #[cfg(not(target_os = "android"))]
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    // SSH backend handle (any target). Present when this tab is an SSH session;
    // resize is forwarded here, keystrokes go through `writer` (an SshWriter).
    ssh: Option<ssh::SshHandle>,
    // Deferred SSH profile: the connection (heavy: tokio+ring) is started a few
    // frames AFTER launch, off the startup/first-paint path, so the app window
    // appears even if the network stack is slow — and never blocks/crashes launch.
    pending_profile: Option<ssh::SshProfile>,
    // Built-in CLI. On Android a tab opens at a local prompt instead of dialing
    // SSH; keystrokes go here until a session starts, and come back when it ends.
    #[cfg(target_os = "android")]
    repl: Option<Repl>,
    title: String,
    split: Option<Box<TermTab>>,
    split_dir: SplitDir,
    split_focused: bool,
}

/// The built-in prompt shown in every Android tab. Leuwi Panjang is a terminal
/// first: a new tab lands here, not in an SSH session. `nvgpu` starts the session,
/// and unknown commands fall through to the OS shell.
#[cfg(target_os = "android")]
struct Repl {
    line: String,
}

#[cfg(target_os = "android")]
impl Repl {
    const PROMPT: &'static str = "\x1b[1;38;5;39mleuwi\x1b[0m\x1b[2m>\x1b[0m ";
    const HELP: &'static str = concat!(
        "\x1b[2mperintah built-in:\x1b[0m\r\n",
        "  nvgpu [sesi]   sambung ke server nvgpu (tmux attach-or-create)\r\n",
        "  config         lihat/ubah host, port, user, session\r\n",
        "  keygen         buat SSH key baru + tampilkan public key\r\n",
        "  rename <nama>  ganti nama tab\r\n",
        "  clear          bersihkan layar\r\n",
        "  help           tampilkan bantuan ini\r\n",
        "\x1b[2mperintah lain diteruskan ke /system/bin/sh (non-interaktif)\x1b[0m\r\n",
    );

    fn new() -> Self {
        Self { line: String::new() }
    }

    fn banner() -> String {
        format!(
            "\r\n  \x1b[1;38;5;39mLeuwi Panjang\x1b[0m \x1b[2mAndroid\x1b[0m\r\n  \x1b[2m—————————————————————————————\x1b[0m\r\n  \x1b[2mketik \x1b[0mhelp\x1b[2m untuk daftar perintah, \x1b[0mnvgpu\x1b[2m untuk menyambung\x1b[0m\r\n\r\n"
        )
    }
}

impl TermTab {
    // Android build: the only backend is SSH (no local shell). The tab appears
    // immediately with a banner; the actual connection (tokio+ring) starts a few
    // frames later via start_pending_ssh() so launch never blocks or crashes.
    #[cfg(target_os = "android")]
    fn spawn(id: usize, cfg: &Config) -> Self {
        let grid = Arc::new(Mutex::new(TermGrid::new(cfg.cols, cfg.rows)));
        {
            let mut g = grid.lock().unwrap_or_else(|e| e.into_inner());
            g.process(Repl::banner().as_bytes());
            g.process(Repl::PROMPT.as_bytes());
        }
        Self {
            grid,
            writer: None,
            ssh: None,
            pending_profile: None,
            repl: Some(Repl::new()),
            title: format!("leuwi {}", id),
            split: None, split_dir: SplitDir::Vertical, split_focused: false,
        }
    }

    /// Keystrokes belong to the built-in prompt whenever no session is attached.
    #[cfg(target_os = "android")]
    fn repl_active(&self) -> bool {
        self.ssh.is_none() && self.repl.is_some()
    }

    /// Drop a finished SSH session and hand control back to the prompt.
    #[cfg(target_os = "android")]
    fn reap_finished_ssh(&mut self) {
        let finished = self.ssh.as_ref().map(|h| h.is_done()).unwrap_or(false);
        if finished {
            self.ssh = None;
            self.writer = None;
            self.title = "leuwi".into();
            let mut g = self.grid.lock().unwrap_or_else(|e| e.into_inner());
            g.process(b"\r\n\x1b[2msesi berakhir\x1b[0m\r\n");
            g.process(Repl::PROMPT.as_bytes());
        }
    }

    /// Feed keystrokes to the built-in prompt: echo, edit, and run on Enter.
    #[cfg(target_os = "android")]
    fn repl_feed(&mut self, data: &[u8], cfg: &mut Config, tab_id: usize) {
        let mut lines: Vec<String> = Vec::new();
        {
            let repl = match self.repl.as_mut() { Some(r) => r, None => return };
            let mut g = self.grid.lock().unwrap_or_else(|e| e.into_inner());
            for &b in data {
                match b {
                    b'\r' | b'\n' => {
                        g.process(b"\r\n");
                        lines.push(std::mem::take(&mut repl.line));
                    }
                    0x7f | 0x08 => {
                        if repl.line.pop().is_some() {
                            g.process(b"\x08 \x08");
                        }
                    }
                    0x03 => {
                        repl.line.clear();
                        g.process(b"^C\r\n");
                        g.process(Repl::PROMPT.as_bytes());
                    }
                    b if b >= 0x20 => {
                        repl.line.push(b as char);
                        g.process(&[b]);
                    }
                    _ => {}
                }
            }
        }
        for line in lines {
            self.repl_exec(line.trim(), cfg, tab_id);
        }
    }

    /// Run one prompt line: built-ins first, anything else goes to the OS shell.
    #[cfg(target_os = "android")]
    fn repl_exec(&mut self, line: &str, cfg: &mut Config, tab_id: usize) {
        let mut out = String::new();
        let mut connect = false;
        let mut suppress_prompt = false;
        let mut argv = line.split_whitespace();
        let cmd = argv.next().unwrap_or("");
        let args: Vec<&str> = argv.collect();

        // Name the tab after its first real command, unless it was renamed by hand.
        // `nvgpu` overrides this below with the session name.
        if !cmd.is_empty() && cmd != "rename" && self.title.starts_with("leuwi") {
            self.title = cmd.to_string();
        }

        match cmd {
            "" => {}
            "help" | "?" => out.push_str(Repl::HELP),
            "clear" => out.push_str("\x1b[2J\x1b[H"),
            "rename" | "title" => match args.first() {
                Some(name) => {
                    self.title = (*name).to_string();
                    out.push_str(&format!("\x1b[2mtab: {}\x1b[0m\r\n", name));
                }
                None => out.push_str("\x1b[2mpakai: rename <nama-tab>\x1b[0m\r\n"),
            },
            "exit" | "quit" => out.push_str("\x1b[2mtutup tab dengan tombol × di atas\x1b[0m\r\n"),
            "keygen" => {
                let path = ssh::default_key_path();
                match ssh::generate_key(&path, "leuwi-panjang-android") {
                    Ok(pubkey) => out.push_str(&format!(
                        "\x1b[2mkey baru: {}\x1b[0m\r\n\r\n{}\r\n\r\n\x1b[2msalin baris di atas ke ~/.ssh/authorized_keys di server\x1b[0m\r\n",
                        path.display(), pubkey)),
                    Err(e) => out.push_str(&format!("\x1b[31m{}\x1b[0m\r\n", e)),
                }
            }
            "config" => match args.split_first() {
                None | Some((&"show", _)) => out.push_str(&format!(
                    "\x1b[2mhost\x1b[0m    {}\r\n\x1b[2mport\x1b[0m    {}\r\n\x1b[2muser\x1b[0m    {}\r\n\x1b[2msession\x1b[0m {}\r\n\x1b[2mkey\x1b[0m     {}\r\n",
                    cfg.ssh_host, cfg.ssh_port, cfg.ssh_user, cfg.ssh_session,
                    ssh::default_key_path().display())),
                Some((&"set", rest)) if rest.len() == 2 => {
                    let (k, v) = (rest[0], rest[1]);
                    match k {
                        "host" => { cfg.ssh_host = v.into(); }
                        "user" => { cfg.ssh_user = v.into(); }
                        "session" => { cfg.ssh_session = v.into(); }
                        "port" => match v.parse::<u16>() {
                            Ok(p) => { cfg.ssh_port = p; }
                            Err(_) => out.push_str("\x1b[31mport harus angka\x1b[0m\r\n"),
                        },
                        _ => out.push_str("\x1b[31mkunci: host | port | user | session\x1b[0m\r\n"),
                    }
                    if out.is_empty() {
                        out.push_str(&format!("\x1b[2m{} = {}\x1b[0m\r\n", k, v));
                    }
                }
                _ => out.push_str("\x1b[2mpakai: config show | config set <host|port|user|session> <nilai>\x1b[0m\r\n"),
            },
            other if other == cfg.ssh_cmd_connect => {
                if let Some(sess) = args.first() {
                    cfg.ssh_session = (*sess).to_string();
                }
                connect = true;
            }
            other if other == cfg.ssh_cmd_list => {
                // `tmux ls` over SSH is blocking, so run it off the UI thread and let it
                // print its own prompt when the answer arrives.
                let profile = cfg.ssh_profile_for_tab(tab_id);
                let grid = self.grid.clone();
                suppress_prompt = true;
                std::thread::spawn(move || {
                    let text = match ssh::list_sessions(&profile) {
                        Ok(s) if s.trim().is_empty() => "(tidak ada session)\r\n".to_string(),
                        Ok(s) => s.replace(char::from(10), "\r\n"),
                        Err(e) => format!("\x1b[31m{}\x1b[0m\r\n", e),
                    };
                    if let Ok(mut g) = grid.lock() {
                        g.process(text.as_bytes());
                        g.process(Repl::PROMPT.as_bytes());
                    }
                });
            }
            other => {
                // Not a built-in: hand it to the OS shell. This is non-interactive
                // (output is captured, no PTY), so `ls`/`cat`/`ps` work but `vim` does not.
                match std::process::Command::new("/system/bin/sh").arg("-c").arg(line).output() {
                    Ok(o) => {
                        for chunk in [&o.stdout[..], &o.stderr[..]] {
                            out.push_str(&String::from_utf8_lossy(chunk).replace(char::from(10), "\r\n"));
                        }
                    }
                    Err(e) => out.push_str(&format!("\x1b[31m{}: {}\x1b[0m\r\n", other, e)),
                }
            }
        }

        {
            let mut g = self.grid.lock().unwrap_or_else(|e| e.into_inner());
            if !out.is_empty() { g.process(out.as_bytes()); }
            if !connect && !suppress_prompt { g.process(Repl::PROMPT.as_bytes()); }
        }
        if connect {
            let profile = cfg.ssh_profile_for_tab(tab_id);
            self.title = format!("nvgpu {}", tab_id);
            self.pending_profile = Some(profile);
            self.start_pending_ssh();
        }
    }

    /// Start a deferred SSH connection into this tab's existing grid (Android).
    /// No-op if there is nothing pending or a connection already exists.
    fn start_pending_ssh(&mut self) {
        if let Some(profile) = self.pending_profile.take() {
            let sink_grid = self.grid.clone();
            let handle = ssh::spawn(profile, move |bytes| {
                if let Ok(mut g) = sink_grid.lock() { g.process(bytes); }
            });
            self.writer = Some(Box::new(handle.writer()));
            self.ssh = Some(handle);
        }
    }

    /// SSH-backed tab that connects immediately (desktop `LEUWI_SSH=1` path).
    #[cfg(not(target_os = "android"))]
    fn spawn_ssh(id: usize, cfg: &Config, profile: ssh::SshProfile) -> Self {
        let grid = Arc::new(Mutex::new(TermGrid::new(cfg.cols, cfg.rows)));
        let sink_grid = grid.clone();
        let handle = ssh::spawn(profile, move |bytes| {
            if let Ok(mut g) = sink_grid.lock() { g.process(bytes); }
        });
        let writer: Box<dyn Write + Send> = Box::new(handle.writer());
        Self {
            grid,
            writer: Some(writer),
            master: None,
            ssh: Some(handle),
            pending_profile: None,
            title: format!("nvgpu {}", id),
            split: None, split_dir: SplitDir::Vertical, split_focused: false,
        }
    }

    #[cfg(not(target_os = "android"))]
    fn spawn(id: usize, cfg: &Config) -> Self {
        // Desktop opt-in: LEUWI_SSH=1 makes even desktop tabs SSH into the profile,
        // so the SSH backend can be exercised in the real GUI before shipping to phone.
        if std::env::var("LEUWI_SSH").is_ok() {
            return Self::spawn_ssh(id, cfg, cfg.ssh_profile_for_tab(id));
        }
        let grid = Arc::new(Mutex::new(TermGrid::new(cfg.cols, cfg.rows)));
        let pty_system = portable_pty::native_pty_system();
        let size = portable_pty::PtySize { rows: cfg.rows as u16, cols: cfg.cols as u16, pixel_width: 0, pixel_height: 0 };
        let pair = pty_system.openpty(size).unwrap();

        let shell = &cfg.shell;
        let mut cmd = portable_pty::CommandBuilder::new(shell);
        if shell.contains("zsh") { cmd.args(["--no-globalrcs", "--no-rcs"]); }
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("PROMPT", &cfg.prompt);
        cmd.env("RPROMPT", "");
        cmd.env("LS_COLORS", "di=1;34:ln=1;36:so=1;35:pi=33:ex=1;32:bd=33;40:cd=33;40:*.tar=1;31:*.gz=1;31:*.zip=1;31:*.jpg=1;35:*.png=1;35:*.rs=33:*.go=36:*.py=33:*.js=33:*.ts=36:*.java=31:*.toml=33:*.json=33:*.md=37:*.sh=32");
        cmd.env("CLICOLOR", "1");
        for v in &["HOME","USER","PATH","LANG","DISPLAY","WAYLAND_DISPLAY","XDG_RUNTIME_DIR","DBUS_SESSION_BUS_ADDRESS","SSH_AUTH_SOCK","EDITOR"] {
            if let Ok(val) = std::env::var(v) { cmd.env(v, &val); }
        }
        pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().unwrap();
        let mut writer = pair.master.take_writer().unwrap();
        let master = pair.master;

        let _ = writer.write_all(b"alias ls='ls --color=auto'\nalias ll='ls -lah --color=auto'\nalias grep='grep --color=auto'\nclear\n");
        let _ = writer.flush();

        let g = grid.clone();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => { g.lock().unwrap_or_else(|e| e.into_inner()).process(&buf[..n]); }
                }
            }
        });

        Self { grid, writer: Some(writer), master: Some(master), ssh: None, pending_profile: None, title: format!("Terminal {}", id), split: None, split_dir: SplitDir::Vertical, split_focused: false }
    }

    fn write(&mut self, data: &[u8]) {
        if let Some(w) = &mut self.writer {
            let _ = w.write_all(data);
            let _ = w.flush();
        }
    }

    #[allow(dead_code)]
    fn get_selected_text(&self) -> String {
        let grid = self.grid.lock().unwrap_or_else(|e| e.into_inner());
        grid.render()
    }

    fn resize(&mut self, cols: usize, rows: usize) {
        let mut grid = self.grid.lock().unwrap_or_else(|e| e.into_inner());
        grid.resize(cols, rows);
        grid.scroll_bottom = rows.saturating_sub(1);
        drop(grid);
        // SSH tab: forward the new size to the remote PTY (tmux/vim/htop adapt).
        if let Some(ssh) = &self.ssh {
            ssh.resize(cols as u16, rows as u16);
        }
        // Local PTY tab (desktop): resize triggers SIGWINCH.
        #[cfg(not(target_os = "android"))]
        if let Some(master) = &self.master {
            let _ = master.resize(portable_pty::PtySize {
                rows: rows as u16, cols: cols as u16,
                pixel_width: 0, pixel_height: 0,
            });
        }
    }

    fn dynamic_title(&self) -> String {
        let grid = self.grid.lock().unwrap_or_else(|e| e.into_inner());
        // Use OSC title if set
        if !grid.title.is_empty() {
            return grid.title.clone();
        }
        // Fallback: extract from prompt
        for r in (0..grid.rows).rev() {
            let line: String = grid.cells[r].iter().map(|c| if c.ch == ' ' { ' ' } else { c.ch }).collect();
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            if let Some(at_pos) = trimmed.find('@') {
                if let Some(space_after) = trimmed[at_pos..].find(' ') {
                    let after = &trimmed[at_pos + space_after + 1..];
                    let path = after.split(' ').next().unwrap_or("~");
                    let display_path = if path == "~" { "home/" } else { path };
                    let name = display_path.rsplit('/').next().unwrap_or(display_path);
                    if !name.is_empty() && name != "%" && name != "$" {
                        return name.to_string();
                    }
                    return display_path.to_string();
                }
            }
            break;
        }
        self.title.clone()
    }
}

// ── Cell with color ────────────────────────────────────────
#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: u32,   // packed: 0xFF=default, 0-15=ansi, 0x01RRGGBB=truecolor
    bg: u32,   // same format
    bold: bool,
    underline: bool,
}

const DEFAULT_FG: u32 = 0xFF;
const DEFAULT_BG: u32 = 0xFF;

impl Default for Cell {
    fn default() -> Self { Self { ch: ' ', fg: DEFAULT_FG, bg: DEFAULT_BG, bold: false, underline: false } }
}

fn pack_rgb(r: u8, g: u8, b: u8) -> u32 {
    0x01000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn is_truecolor(c: u32) -> bool { c & 0x01000000 != 0 }

fn color_to_vec4(c: u32) -> Vec4 {
    if is_truecolor(c) {
        let r = ((c >> 16) & 0xFF) as f32 / 255.0;
        let g = ((c >> 8) & 0xFF) as f32 / 255.0;
        let b = (c & 0xFF) as f32 / 255.0;
        return vec4(r, g, b, 1.0);
    }
    let idx = c as u8;
    match idx {
        0..=15 | 254 | 255 => ansi_to_vec4(idx),
        // 256-color: 16-231 = 6x6x6 color cube
        16..=231 => {
            let n = idx - 16;
            let r = (n / 36) as f32 * 51.0 / 255.0;
            let g = ((n / 6) % 6) as f32 * 51.0 / 255.0;
            let b = (n % 6) as f32 * 51.0 / 255.0;
            // Boost slightly for visibility on dark bg
            vec4(r.max(0.02), g.max(0.02), b.max(0.02), 1.0)
        }
        // 256-color: 232-253 = grayscale ramp
        232..=253 => {
            let v = ((idx - 232) as f32 * 10.0 + 8.0) / 255.0;
            vec4(v, v, v, 1.0)
        }
    }
}

// ── Terminal Grid ──────────────────────────────────────────
struct TermGrid {
    cols: usize,
    rows: usize,
    cells: Vec<Vec<Cell>>,
    scrollback: Vec<Vec<Cell>>,
    max_scrollback: usize,
    cur_r: usize,
    cur_c: usize,
    cur_fg: u32,
    cur_bg: u32,
    cur_bold: bool,
    cur_underline: bool,
    // Modes
    mouse_reporting: bool,
    bracketed_paste: bool,
    app_cursor_keys: bool,
    // Response buffer (for DA, DSR replies sent back to PTY)
    response_buf: Vec<u8>,
    // Alternate screen buffer (for vim, htop, less, etc.)
    alt_cells: Option<Vec<Vec<Cell>>>,
    alt_cur_r: usize,
    alt_cur_c: usize,
    in_alt_screen: bool,
    // Saved cursor
    saved_cur_r: usize,
    saved_cur_c: usize,
    // Scroll region
    scroll_top: usize,
    scroll_bottom: usize,
    // Selection (mouse drag)
    sel_start: Option<(usize, usize)>, // (row, col)
    sel_end: Option<(usize, usize)>,
    // Window title from OSC
    title: String,
    // Bell flag
    bell: bool,
}

impl Default for TermGrid {
    fn default() -> Self { Self::new(110, 33) }
}

impl TermGrid {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols, rows,
            cells: vec![vec![Cell::default(); cols]; rows],
            scrollback: Vec::new(),
            max_scrollback: 5000,
            cur_r: 0, cur_c: 0, cur_fg: DEFAULT_FG, cur_bg: DEFAULT_BG, cur_bold: false, cur_underline: false,
            mouse_reporting: false, bracketed_paste: false, app_cursor_keys: false, response_buf: Vec::new(),
            alt_cells: None, alt_cur_r: 0, alt_cur_c: 0, in_alt_screen: false,
            saved_cur_r: 0, saved_cur_c: 0,
            scroll_top: 0, scroll_bottom: rows.saturating_sub(1),
            sel_start: None, sel_end: None,
            title: String::new(), bell: false,
        }
    }

    fn enter_alt_screen(&mut self) {
        if self.in_alt_screen { return; }
        self.in_alt_screen = true;
        self.alt_cur_r = self.cur_r;
        self.alt_cur_c = self.cur_c;
        self.alt_cells = Some(self.cells.clone());
        self.cells = vec![vec![Cell::default(); self.cols]; self.rows];
        self.cur_r = 0;
        self.cur_c = 0;
        self.scroll_top = 0;
        self.scroll_bottom = self.rows.saturating_sub(1);
    }

    fn leave_alt_screen(&mut self) {
        if !self.in_alt_screen { return; }
        self.in_alt_screen = false;
        if let Some(main) = self.alt_cells.take() {
            self.cells = main;
        }
        self.cur_r = self.alt_cur_r;
        self.cur_c = self.alt_cur_c;
        self.scroll_top = 0;
        self.scroll_bottom = self.rows.saturating_sub(1);
    }

    fn save_cursor(&mut self) {
        self.saved_cur_r = self.cur_r;
        self.saved_cur_c = self.cur_c;
    }

    fn restore_cursor(&mut self) {
        self.cur_r = self.saved_cur_r.min(self.rows.saturating_sub(1));
        self.cur_c = self.saved_cur_c.min(self.cols.saturating_sub(1));
    }

    fn put(&mut self, ch: char) {
        if self.cur_c >= self.cols { self.cur_c = 0; self.newline(); }
        if self.cur_r < self.rows {
            self.cells[self.cur_r][self.cur_c] = Cell { ch, fg: self.cur_fg, bg: self.cur_bg, bold: self.cur_bold, underline: self.cur_underline };
            self.cur_c += 1;
        }
    }

    fn newline(&mut self) {
        if self.cur_r == self.scroll_bottom {
            // Scroll within region
            let top = self.cells.remove(self.scroll_top);
            if self.scroll_top == 0 && !self.in_alt_screen {
                self.scrollback.push(top);
                if self.scrollback.len() > self.max_scrollback {
                    self.scrollback.remove(0);
                }
            }
            self.cells.insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
        } else if self.cur_r + 1 < self.rows {
            self.cur_r += 1;
        }
    }

    fn cr(&mut self) { self.cur_c = 0; }
    fn bs(&mut self) { if self.cur_c > 0 { self.cur_c -= 1; } }
    fn tab(&mut self) { self.cur_c = ((self.cur_c / 8) + 1) * 8; if self.cur_c >= self.cols { self.cur_c = self.cols - 1; } }

    fn clear_line_right(&mut self) { for c in self.cur_c..self.cols { self.cells[self.cur_r][c] = Cell::default(); } }
    fn clear_line_full(&mut self) { for c in 0..self.cols { self.cells[self.cur_r][c] = Cell::default(); } }
    fn clear_screen(&mut self) { for r in &mut self.cells { for c in r.iter_mut() { *c = Cell::default(); } } self.cur_r = 0; self.cur_c = 0; }
    fn clear_below(&mut self) { self.clear_line_right(); for r in (self.cur_r+1)..self.rows { for c in 0..self.cols { self.cells[r][c] = Cell::default(); } } }

    fn resize(&mut self, new_cols: usize, new_rows: usize) {
        let mut new_cells = vec![vec![Cell::default(); new_cols]; new_rows];
        let copy_r = self.rows.min(new_rows);
        let copy_c = self.cols.min(new_cols);
        for r in 0..copy_r {
            for c in 0..copy_c {
                new_cells[r][c] = self.cells[r][c];
            }
        }
        self.cells = new_cells;
        self.cols = new_cols;
        self.rows = new_rows;
        self.cur_r = self.cur_r.min(new_rows.saturating_sub(1));
        self.cur_c = self.cur_c.min(new_cols.saturating_sub(1));
    }

    /// Search scrollback + visible for text, return list of (abs_row, col) matches
    #[allow(dead_code)]
    fn search(&self, query: &str) -> Vec<(usize, usize)> {
        let mut results = Vec::new();
        if query.is_empty() { return results; }
        let q = query.to_lowercase();

        // Search scrollback
        for (r, row) in self.scrollback.iter().enumerate() {
            let line: String = row.iter().map(|c| c.ch.to_lowercase().next().unwrap_or(' ')).collect();
            let mut start = 0;
            while let Some(pos) = line[start..].find(&q) {
                results.push((r, start + pos));
                start += pos + 1;
            }
        }
        // Search visible
        let sb = self.scrollback.len();
        for r in 0..self.rows {
            let line: String = self.cells[r].iter().map(|c| c.ch.to_lowercase().next().unwrap_or(' ')).collect();
            let mut start = 0;
            while let Some(pos) = line[start..].find(&q) {
                results.push((sb + r, start + pos));
                start += pos + 1;
            }
        }
        results
    }

    fn start_select(&mut self, row: usize, col: usize) {
        self.sel_start = Some((row, col));
        self.sel_end = None;
    }

    fn update_select(&mut self, row: usize, col: usize) {
        self.sel_end = Some((row, col));
    }

    fn get_selection_text(&self) -> Option<String> {
        let (s, e) = match (self.sel_start, self.sel_end) {
            (Some(s), Some(e)) => if s.0 < e.0 || (s.0 == e.0 && s.1 <= e.1) { (s, e) } else { (e, s) },
            _ => return None,
        };
        let mut text = String::new();
        let all_rows: Vec<&Vec<Cell>> = self.scrollback.iter().chain(self.cells.iter()).collect();
        for r in s.0..=e.0.min(all_rows.len().saturating_sub(1)) {
            let cs = if r == s.0 { s.1 } else { 0 };
            let ce = if r == e.0 { e.1 } else { all_rows[r].len().saturating_sub(1) };
            for c in cs..=ce.min(all_rows[r].len().saturating_sub(1)) {
                let ch = all_rows[r][c].ch;
                text.push(if ch == ' ' || ch == '\0' { ' ' } else { ch });
            }
            if r < e.0 { text.push('\n'); }
        }
        let trimmed = text.trim_end().to_string();
        if trimmed.is_empty() { None } else { Some(trimmed) }
    }

    fn clear_select(&mut self) {
        self.sel_start = None;
        self.sel_end = None;
    }

    fn select_all(&mut self) {
        let sb = self.scrollback.len();
        self.sel_start = Some((0, 0));
        self.sel_end = Some((sb + self.rows - 1, self.cols - 1));
    }

    /// Find URLs in visible text (simple http/https detection)
    fn find_urls(&self) -> Vec<(usize, usize, usize, String)> {
        // Returns: (row, start_col, end_col, url)
        let mut urls = Vec::new();
        let all_rows: Vec<&Vec<Cell>> = self.scrollback.iter().chain(self.cells.iter()).collect();
        for (r, row) in all_rows.iter().enumerate() {
            let line: String = row.iter().map(|c| c.ch).collect();
            let mut search_from = 0;
            while let Some(pos) = line[search_from..].find("http") {
                let start = search_from + pos;
                // Find end of URL (space, ), ], or end of line)
                let end = line[start..].find(|c: char| c == ' ' || c == ')' || c == ']' || c == '\'' || c == '"')
                    .map(|e| start + e)
                    .unwrap_or(line.len());
                let url = line[start..end].trim().to_string();
                if url.starts_with("http://") || url.starts_with("https://") {
                    urls.push((r, start, end, url));
                }
                search_from = end;
            }
        }
        urls
    }

    fn is_selected(&self, abs_row: usize, col: usize) -> bool {
        let (s, e) = match (self.sel_start, self.sel_end) {
            (Some(s), Some(e)) => if s.0 < e.0 || (s.0 == e.0 && s.1 <= e.1) { (s, e) } else { (e, s) },
            _ => return false,
        };
        if abs_row < s.0 || abs_row > e.0 { return false; }
        if abs_row == s.0 && abs_row == e.0 { return col >= s.1 && col <= e.1; }
        if abs_row == s.0 { return col >= s.1; }
        if abs_row == e.0 { return col <= e.1; }
        true
    }

    fn render(&self) -> String {
        // Render all visible rows, trim trailing empty rows
        let mut out = String::with_capacity((self.cols + 1) * self.rows);
        // Find last row with content (at least up to cursor row)
        let last_row = self.cur_r.max(
            self.cells.iter().enumerate()
                .filter(|(_, row)| row.iter().any(|c| c.ch != ' '))
                .map(|(r, _)| r)
                .max()
                .unwrap_or(0)
        );
        for r in 0..=last_row {
            let mut last_col = 0;
            for (c, cell) in self.cells[r].iter().enumerate() {
                if cell.ch != ' ' { last_col = c + 1; }
            }
            // Always output at least empty line for cursor row
            for c in 0..last_col { out.push(self.cells[r][c].ch); }
            if r < last_row { out.push('\n'); }
        }
        out
    }

    fn process(&mut self, data: &[u8]) {
        let mut i = 0;
        while i < data.len() {
            match data[i] {
                0x1b => {
                    i += 1; if i >= data.len() { break; }
                    match data[i] {
                        b'[' => {
                            i += 1;
                            let mut params: Vec<usize> = Vec::new();
                            let mut num: i32 = -1;
                            let mut private = false;
                            while i < data.len() {
                                let c = data[i];
                                if c == b'?' { private = true; i += 1; continue; }
                                if c == b'>' || c == b'=' || c == b'!' { i += 1; continue; }
                                if (b'0'..=b'9').contains(&c) {
                                    if num < 0 { num = 0; }
                                    num = num * 10 + (c - b'0') as i32;
                                    i += 1; continue;
                                }
                                if c == b';' { params.push(if num < 0 { 0 } else { num as usize }); num = -1; i += 1; continue; }
                                if num >= 0 { params.push(num as usize); }
                                let p0 = params.first().copied().unwrap_or(0);
                                let p1 = params.get(1).copied().unwrap_or(0);
                                match c {
                                    b'A' => self.cur_r = self.cur_r.saturating_sub(p0.max(1)),
                                    b'B' => self.cur_r = (self.cur_r + p0.max(1)).min(self.rows - 1),
                                    b'C' => self.cur_c = (self.cur_c + p0.max(1)).min(self.cols - 1),
                                    b'D' => self.cur_c = self.cur_c.saturating_sub(p0.max(1)),
                                    b'H' | b'f' => { self.cur_r = p0.max(1).saturating_sub(1).min(self.rows-1); self.cur_c = p1.max(1).saturating_sub(1).min(self.cols-1); }
                                    b'G' => self.cur_c = p0.max(1).saturating_sub(1).min(self.cols-1),
                                    b'd' => self.cur_r = p0.max(1).saturating_sub(1).min(self.rows-1),
                                    b'J' => match p0 {
                                        0 => self.clear_below(),
                                        1 => {
                                            // Clear from start to cursor
                                            for r in 0..self.cur_r {
                                                for c in 0..self.cols { self.cells[r][c] = Cell::default(); }
                                            }
                                            for c in 0..=self.cur_c.min(self.cols.saturating_sub(1)) {
                                                self.cells[self.cur_r][c] = Cell::default();
                                            }
                                        }
                                        2 | 3 => self.clear_screen(),
                                        _ => {}
                                    },
                                    b'K' => match p0 {
                                        0 => self.clear_line_right(),
                                        1 => { // clear from start to cursor
                                            for c in 0..=self.cur_c.min(self.cols.saturating_sub(1)) {
                                                self.cells[self.cur_r][c] = Cell::default();
                                            }
                                        }
                                        2 => self.clear_line_full(),
                                        _ => {}
                                    },
                                    b'E' => { self.cur_c = 0; self.cur_r = (self.cur_r + p0.max(1)).min(self.rows-1); }
                                    b'F' => { self.cur_c = 0; self.cur_r = self.cur_r.saturating_sub(p0.max(1)); }
                                    b'm' => {
                                        // SGR — process colors
                                        if params.is_empty() { self.cur_fg = DEFAULT_FG; self.cur_bg = DEFAULT_BG; self.cur_bold = false; self.cur_underline = false; }
                                        else {
                                            let mut j = 0;
                                            while j < params.len() {
                                                match params[j] {
                                                    0 => { self.cur_fg = DEFAULT_FG; self.cur_bg = DEFAULT_BG; self.cur_bold = false; self.cur_underline = false; }
                                                    1 => self.cur_bold = true,
                                                    2 | 22 => self.cur_bold = false,
                                                    3 => {} // italic
                                                    4 => self.cur_underline = true,
                                                    24 => self.cur_underline = false,
                                                    7 => {
                                                        let fg = if self.cur_fg == DEFAULT_FG { 7 } else { self.cur_fg };
                                                        let bg = if self.cur_bg == DEFAULT_BG { 254 } else { self.cur_bg };
                                                        self.cur_fg = bg;
                                                        self.cur_bg = fg;
                                                    }
                                                    27 => {
                                                        let fg = if self.cur_fg == 254 { DEFAULT_FG } else { self.cur_fg };
                                                        let bg = if self.cur_bg == 7 { DEFAULT_BG } else { self.cur_bg };
                                                        self.cur_fg = fg;
                                                        self.cur_bg = bg;
                                                    }
                                                    30..=37 => self.cur_fg = (params[j] - 30) as u32,
                                                    39 => self.cur_fg = DEFAULT_FG,
                                                    40..=47 => self.cur_bg = (params[j] - 40) as u32,
                                                    49 => self.cur_bg = DEFAULT_BG,
                                                    90..=97 => self.cur_fg = (params[j] - 90 + 8) as u32,
                                                    100..=107 => self.cur_bg = (params[j] - 100 + 8) as u32,
                                                    38 => {
                                                        if j+2 < params.len() && params[j+1] == 5 {
                                                            self.cur_fg = params[j+2].min(255) as u32;
                                                            j += 2;
                                                        } else if j+4 < params.len() && params[j+1] == 2 {
                                                            // TRUE COLOR RGB
                                                            self.cur_fg = pack_rgb(params[j+2] as u8, params[j+3] as u8, params[j+4] as u8);
                                                            j += 4;
                                                        }
                                                    }
                                                    48 => {
                                                        if j+2 < params.len() && params[j+1] == 5 {
                                                            self.cur_bg = params[j+2].min(255) as u32;
                                                            j += 2;
                                                        } else if j+4 < params.len() && params[j+1] == 2 {
                                                            self.cur_bg = pack_rgb(params[j+2] as u8, params[j+3] as u8, params[j+4] as u8);
                                                            j += 4;
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                j += 1;
                                            }
                                        }
                                    }
                                    b'h' => {
                                        if private {
                                            for &p in &params {
                                                match p {
                                                    1049 | 47 | 1047 => self.enter_alt_screen(),
                                                    1000 | 1002 | 1003 | 1006 => self.mouse_reporting = true,
                                                    2004 => self.bracketed_paste = true,
                                                    1 => self.app_cursor_keys = true, // DECCKM
                                                    12 | 25 | 1004 | 1005 | 7 => {}
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    b'l' => {
                                        if private {
                                            for &p in &params {
                                                match p {
                                                    1049 | 47 | 1047 => self.leave_alt_screen(),
                                                    1000 | 1002 | 1003 | 1006 => self.mouse_reporting = false,
                                                    2004 => self.bracketed_paste = false,
                                                    1 => self.app_cursor_keys = false,
                                                    12 | 25 | 1004 | 1005 | 7 => {}
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    b'L' => {
                                        // IL — insert lines
                                        let n = p0.max(1);
                                        for _ in 0..n {
                                            if self.cur_r < self.rows {
                                                self.cells.insert(self.cur_r, vec![Cell::default(); self.cols]);
                                                self.cells.truncate(self.rows);
                                            }
                                        }
                                    }
                                    b'M' => {
                                        // DL — delete lines
                                        let n = p0.max(1);
                                        for _ in 0..n {
                                            if self.cur_r < self.rows {
                                                self.cells.remove(self.cur_r);
                                                self.cells.push(vec![Cell::default(); self.cols]);
                                            }
                                        }
                                    }
                                    b'P' => {
                                        // DCH — delete characters
                                        let n = p0.max(1);
                                        let row = &mut self.cells[self.cur_r];
                                        for _ in 0..n {
                                            if self.cur_c < self.cols {
                                                row.remove(self.cur_c);
                                                row.push(Cell::default());
                                            }
                                        }
                                    }
                                    b'@' => {
                                        // ICH — insert characters
                                        let n = p0.max(1);
                                        let row = &mut self.cells[self.cur_r];
                                        for _ in 0..n {
                                            if self.cur_c < self.cols {
                                                row.insert(self.cur_c, Cell::default());
                                                row.truncate(self.cols);
                                            }
                                        }
                                    }
                                    b'X' => {
                                        // ECH — erase characters
                                        let n = p0.max(1);
                                        for c in self.cur_c..(self.cur_c + n).min(self.cols) {
                                            self.cells[self.cur_r][c] = Cell::default();
                                        }
                                    }
                                    b'r' => {
                                        // DECSTBM — set scroll region
                                        let top = if p0 > 0 { p0 - 1 } else { 0 };
                                        let bot = if p1 > 0 { (p1 - 1).min(self.rows - 1) } else { self.rows - 1 };
                                        self.scroll_top = top;
                                        self.scroll_bottom = bot;
                                        self.cur_r = 0;
                                        self.cur_c = 0;
                                    }
                                    b'S' => {
                                        // SU — scroll up N lines
                                        let n = p0.max(1);
                                        for _ in 0..n {
                                            self.cells.remove(self.scroll_top);
                                            self.cells.insert(self.scroll_bottom, vec![Cell::default(); self.cols]);
                                        }
                                    }
                                    b'T' => {
                                        // SD — scroll down N lines
                                        let n = p0.max(1);
                                        for _ in 0..n {
                                            self.cells.remove(self.scroll_bottom);
                                            self.cells.insert(self.scroll_top, vec![Cell::default(); self.cols]);
                                        }
                                    }
                                    b'c' => {
                                        // DA — Report as xterm-256color compatible
                                        if private || p0 == 0 {
                                            self.response_buf.extend_from_slice(b"\x1b[?62;22c");
                                        }
                                    }
                                    b'n' => {
                                        if p0 == 6 {
                                            let resp = format!("\x1b[{};{}R", self.cur_r + 1, self.cur_c + 1);
                                            self.response_buf.extend_from_slice(resp.as_bytes());
                                        } else if p0 == 5 {
                                            self.response_buf.extend_from_slice(b"\x1b[0n");
                                        }
                                    }
                                    b't' => {
                                        // Window ops
                                        match p0 {
                                            14 => {
                                                // Report window size in pixels
                                                let pw = self.cols as u32 * 9;
                                                let ph = self.rows as u32 * 20;
                                                let resp = format!("\x1b[4;{};{}t", ph, pw);
                                                self.response_buf.extend_from_slice(resp.as_bytes());
                                            }
                                            18 => {
                                                // Report text area size in chars
                                                let resp = format!("\x1b[8;{};{}t", self.rows, self.cols);
                                                self.response_buf.extend_from_slice(resp.as_bytes());
                                            }
                                            22 => {} // push title
                                            23 => {} // pop title
                                            _ => {}
                                        }
                                    }
                                    b'q' => {
                                        // DECSCUSR — Set Cursor Style
                                        // 0,1=block blink, 2=block steady, 3=underline blink,
                                        // 4=underline steady, 5=beam blink, 6=beam steady
                                        // We store this for rendering
                                        match p0 {
                                            0 | 1 | 2 => {} // block (default)
                                            3 | 4 => {}      // underline
                                            5 | 6 => {}      // beam
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                                i += 1; break;
                            }
                        }
                        b']' => {
                            // OSC — parse title (OSC 0;title BEL or OSC 2;title BEL)
                            i += 1;
                            let mut osc_data = Vec::new();
                            while i < data.len() {
                                if data[i] == 0x07 { i += 1; break; }
                                if data[i] == 0x1b && i+1 < data.len() && data[i+1] == b'\\' { i += 2; break; }
                                osc_data.push(data[i]);
                                i += 1;
                            }
                            if let Ok(s) = std::str::from_utf8(&osc_data) {
                                if s.starts_with("0;") || s.starts_with("2;") {
                                    self.title = s[2..].to_string();
                                } else if s.starts_with("10;?") {
                                    // Query fg color → respond with #C5C8C6
                                    self.response_buf.extend_from_slice(b"\x1b]10;rgb:c5c5/c8c8/c6c6\x1b\\");
                                } else if s.starts_with("11;?") {
                                    // Query bg color → respond with #1E1E1E
                                    self.response_buf.extend_from_slice(b"\x1b]11;rgb:1e1e/1e1e/1e1e\x1b\\");
                                } else if s.starts_with("4;") && s.contains("?") {
                                    // Query color palette — respond with empty (skip)
                                }
                            }
                        }
                        b'P' | b'_' | b'^' => { i += 1; while i < data.len() { if data[i] == 0x1b { i += 2; break; } if data[i] == 0x07 { i += 1; break; } i += 1; } }
                        b'(' | b')' | b'*' | b'+' => { i += 1; if i < data.len() { i += 1; } }
                        b'7' => { self.save_cursor(); }     // DECSC
                        b'8' => { self.restore_cursor(); }  // DECRC
                        b'M' => {                           // RI — reverse index
                            if self.cur_r == self.scroll_top {
                                // Scroll down within region
                                if self.scroll_bottom < self.rows {
                                    self.cells.remove(self.scroll_bottom);
                                }
                                self.cells.insert(self.scroll_top, vec![Cell::default(); self.cols]);
                                if self.cells.len() > self.rows { self.cells.truncate(self.rows); }
                            } else if self.cur_r > 0 {
                                self.cur_r -= 1;
                            }
                        }
                        _ => { i += 1; }
                    }
                }
                b'\n' => { self.newline(); i += 1; }
                b'\r' => { self.cr(); i += 1; }
                b'\x08' => { self.bs(); i += 1; }
                b'\t' => { self.tab(); i += 1; }
                0x07 => { self.bell = true; i += 1; } // BEL
                0x00..=0x06 | 0x0e..=0x1a | 0x1c..=0x1f => { i += 1; }
                _ => {
                    if data[i] < 0x80 { self.put(data[i] as char); i += 1; }
                    else {
                        let len = if data[i] < 0xE0 { 2 } else if data[i] < 0xF0 { 3 } else { 4 };
                        let end = (i + len).min(data.len());
                        if let Ok(s) = std::str::from_utf8(&data[i..end]) {
                            for ch in s.chars() { self.put(ch); }
                            i = end;
                        } else { i += 1; }
                    }
                }
            }
        }
    }
}

fn rgb_to_ansi(r: u8, g: u8, b: u8) -> u8 {
    // Simple mapping to 16 basic colors
    let brightness = (r as u16 + g as u16 + b as u16) / 3;
    if brightness < 40 { return 0; }
    if r > 150 && g < 100 && b < 100 { return if brightness > 180 { 9 } else { 1 }; }
    if g > 150 && r < 100 && b < 100 { return if brightness > 180 { 10 } else { 2 }; }
    if r > 150 && g > 150 && b < 100 { return if brightness > 180 { 11 } else { 3 }; }
    if b > 150 && r < 100 && g < 100 { return if brightness > 180 { 12 } else { 4 }; }
    if r > 150 && b > 150 && g < 100 { return if brightness > 180 { 13 } else { 5 }; }
    if g > 150 && b > 150 && r < 100 { return if brightness > 180 { 14 } else { 6 }; }
    if brightness > 200 { 15 } else if brightness > 120 { 7 } else { 8 }
}

// ANSI color to Makepad vec4
// Foreground color
fn ansi_to_vec4(idx: u8) -> Vec4 {
    match idx {
        0  => vec4(0.08, 0.08, 0.10, 1.0),  // black — actually dark (visible on colored bg)
        1  => vec4(0.95, 0.30, 0.30, 1.0),  // red
        2  => vec4(0.30, 0.75, 0.35, 1.0),  // green
        3  => vec4(0.85, 0.72, 0.25, 1.0),  // yellow
        4  => vec4(0.40, 0.62, 0.95, 1.0),  // blue
        5  => vec4(0.75, 0.45, 0.95, 1.0),  // magenta
        6  => vec4(0.35, 0.80, 0.85, 1.0),  // cyan
        7  => vec4(0.88, 0.90, 0.94, 1.0),  // white (bright enough for bg text)
        8  => vec4(0.50, 0.55, 0.62, 1.0),  // bright black (comments)
        9  => vec4(1.00, 0.45, 0.45, 1.0),  // bright red
        10 => vec4(0.40, 0.88, 0.48, 1.0),  // bright green
        11 => vec4(0.96, 0.86, 0.38, 1.0),  // bright yellow
        12 => vec4(0.55, 0.76, 1.00, 1.0),  // bright blue
        13 => vec4(0.88, 0.65, 1.00, 1.0),  // bright magenta
        14 => vec4(0.50, 0.93, 0.98, 1.0),  // bright cyan
        15 => vec4(0.95, 0.97, 1.00, 1.0),  // bright white
        254 => vec4(0.12, 0.12, 0.12, 1.0), // default bg (reverse)
        _  => vec4(0.77, 0.79, 0.82, 1.0),  // default fg
    }
}

// ── TermView: colored terminal grid widget ─────────────────

#[derive(Live, LiveHook, Widget)]
pub struct TermView {
    #[redraw] #[live] draw_text: DrawText,
    #[live] draw_bg: DrawColor,
    #[live] draw_cursor: DrawColor,
    #[live] draw_cell_bg: DrawColor,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[rust] grid_ref: Option<Arc<Mutex<TermGrid>>>,
    #[rust] scroll_offset: i64,
    #[rust] blink_on: bool,
    #[rust] blink_counter: u32,
    #[rust] cw: f64,
    #[rust] ch: f64,
    #[rust] cursor_block: bool,
    #[rust] search_highlights: Vec<(usize, usize, usize)>, // (abs_row, start_col, len)
    #[rust] is_focused: bool,
    // Touch state (Android): (start_x, start_y, scroll_offset at touch start)
    #[rust] touch_start: Option<(f64, f64, i64)>,
    #[rust] touch_moved: bool,
}

impl Widget for TermView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::Scroll(se) => {
                let lines = (-se.scroll.y / 20.0) as i64;
                self.scroll_offset = (self.scroll_offset + lines).max(0);
                self.redraw(cx);
            }
            // Touch (Android): one-finger drag scrolls scrollback; a tap raises the
            // soft keyboard. Desktop uses mouse events above; phones only get these.
            Event::TouchUpdate(e) => {
                if let Some(t) = e.touches.first() {
                    match t.state {
                        TouchState::Start => {
                            self.touch_start = Some((t.abs.x, t.abs.y, self.scroll_offset));
                            self.touch_moved = false;
                            cx.set_key_focus(self.draw_bg.area());
                        }
                        TouchState::Move | TouchState::Stable => {
                            if let Some((_sx, sy, s_off)) = self.touch_start {
                                let ch = if self.ch > 0.0 { self.ch } else { 20.0 };
                                let dy = t.abs.y - sy;
                                if dy.abs() > 6.0 { self.touch_moved = true; }
                                // Drag down (dy>0) reveals older output (scroll back).
                                let lines = (dy / ch) as i64;
                                self.scroll_offset = (s_off + lines).max(0);
                                self.redraw(cx);
                            }
                        }
                        TouchState::Stop => {
                            if !self.touch_moved {
                                // Tap: focus + show the on-screen keyboard.
                                cx.set_key_focus(self.draw_bg.area());
                                #[cfg(target_os = "android")]
                                cx.show_text_ime(self.draw_bg.area(), dvec2(0.0, 0.0));
                            }
                            self.touch_start = None;
                        }
                    }
                }
            }
            Event::MouseDown(me) => {
                let cw = if self.cw > 0.0 { self.cw } else { 9.2 };
                let ch = if self.ch > 0.0 { self.ch } else { 20.0 };
                let col = ((me.abs.x - 12.0) / cw).max(0.0) as usize;
                let screen_row = ((me.abs.y - 8.0) / ch).max(0.0) as usize;

                if let Some(grid) = &self.grid_ref {
                    let mut g = grid.lock().unwrap_or_else(|e| e.into_inner());
                    let sb = g.scrollback.len();
                    let start = (sb + g.rows).saturating_sub(g.rows + self.scroll_offset as usize);
                    let abs_row = (start + screen_row).min(sb + g.rows - 1);
                    let max_col = g.cols.saturating_sub(1);

                    // Ctrl+click = open URL
                    if me.modifiers.control {
                        let urls = g.find_urls();
                        for (r, cs, ce, url) in &urls {
                            if *r == abs_row && col >= *cs && col <= *ce {
                                #[cfg(not(target_os = "android"))]
                                { let _ = std::process::Command::new("xdg-open").arg(url).spawn(); }
                                #[cfg(target_os = "android")]
                                { let _ = url; }
                                return;
                            }
                        }
                    }

                    g.start_select(abs_row, col.min(max_col));
                }
                self.redraw(cx);
            }
            Event::MouseMove(me) => {
                let cw = if self.cw > 0.0 { self.cw } else { 9.2 };
                let ch = if self.ch > 0.0 { self.ch } else { 20.0 };
                let col = ((me.abs.x - 12.0) / cw).max(0.0) as usize;
                let screen_row = ((me.abs.y - 8.0) / ch).max(0.0) as usize;
                if let Some(grid) = &self.grid_ref {
                    let mut g = grid.lock().unwrap_or_else(|e| e.into_inner());
                    let sb = g.scrollback.len();
                    let start = (sb + g.rows).saturating_sub(g.rows + self.scroll_offset as usize);
                    let abs_row = (start + screen_row).min(sb + g.rows - 1);
                    let max_col = g.cols.saturating_sub(1);
                    g.update_select(abs_row, col.min(max_col));
                }
                self.redraw(cx);
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);

        let grid = match &self.grid_ref {
            Some(g) => g.lock().unwrap_or_else(|e| e.into_inner()),
            None => return DrawStep::done(),
        };

        let cw = if self.cw > 0.0 { self.cw } else { 9.2 };
        let ch = if self.ch > 0.0 { self.ch } else { 20.0 };
        let pad_x = 12.0;
        let pad_y = 8.0;

        // Build all rows: scrollback + visible
        let sb = &grid.scrollback;
        let sb_len = sb.len();
        let vis_last = grid.cur_r;
        let total_rows = sb_len + vis_last + 1;

        // How many rows fit in window
        let view_rows = ((rect.size.y - pad_y * 2.0) / ch) as usize;

        // Reset scroll if at bottom
        if self.scroll_offset > 0 {
            let max_scroll = total_rows.saturating_sub(view_rows) as i64;
            self.scroll_offset = self.scroll_offset.min(max_scroll);
        }

        // Which rows to show (from bottom - scroll_offset)
        let end_row = total_rows.saturating_sub(self.scroll_offset as usize);
        let start_row = end_row.saturating_sub(view_rows);

        let px = rect.pos.x + pad_x;
        let py = rect.pos.y + pad_y;
        let mut char_buf = [0u8; 4];
        let mut screen_row: usize = 0;

        for abs_row in start_row..end_row {
            let y = py + (screen_row as f64) * ch;
            if y > rect.pos.y + rect.size.y { break; }

            // Get row data from scrollback or visible grid
            let row_cells: &[Cell] = if abs_row < sb_len {
                &sb[abs_row]
            } else {
                let grid_row = abs_row - sb_len;
                if grid_row < grid.rows { &grid.cells[grid_row] } else { screen_row += 1; continue; }
            };

            for (c, cell) in row_cells.iter().enumerate() {
                let x = px + (c as f64) * cw;
                if x > rect.pos.x + rect.size.x { break; }

                // PASS 1: backgrounds only
                let has_bg = cell.bg != DEFAULT_BG;
                if has_bg {
                    self.draw_cell_bg.color = color_to_vec4(cell.bg);
                    self.draw_cell_bg.draw_abs(cx, Rect { pos: dvec2(x, y), size: dvec2(cw, ch) });
                }
                let selected = grid.is_selected(abs_row, c);
                if selected {
                    self.draw_cell_bg.color = vec4(0.20, 0.40, 0.65, 0.5);
                    self.draw_cell_bg.draw_abs(cx, Rect { pos: dvec2(x, y), size: dvec2(cw, ch) });
                }
                // Search highlight
                for &(hr, hc, hlen) in &self.search_highlights {
                    if abs_row == hr && c >= hc && c < hc + hlen {
                        self.draw_cell_bg.color = vec4(0.60, 0.50, 0.10, 0.6);
                        self.draw_cell_bg.draw_abs(cx, Rect { pos: dvec2(x, y), size: dvec2(cw, ch) });
                    }
                }
            }
            screen_row += 1;
        }

        // PASS 2: text on top of backgrounds
        screen_row = 0;
        for abs_row in start_row..end_row {
            let y = py + (screen_row as f64) * ch;
            if y > rect.pos.y + rect.size.y { break; }

            let row_cells: &[Cell] = if abs_row < sb_len {
                &sb[abs_row]
            } else {
                let grid_row = abs_row - sb_len;
                if grid_row < grid.rows { &grid.cells[grid_row] } else { screen_row += 1; continue; }
            };

            for (c, cell) in row_cells.iter().enumerate() {
                if cell.ch == ' ' { continue; }
                let x = px + (c as f64) * cw;
                if x > rect.pos.x + rect.size.x { break; }

                let fg = if cell.bold && cell.fg < 8 && !is_truecolor(cell.fg) { cell.fg + 8 } else { cell.fg };
                self.draw_text.color = color_to_vec4(fg);
                let s = cell.ch.encode_utf8(&mut char_buf);
                self.draw_text.draw_abs(cx, dvec2(x, y), s);

                if cell.underline {
                    self.draw_cell_bg.color = color_to_vec4(fg);
                    self.draw_cell_bg.draw_abs(cx, Rect { pos: dvec2(x, y + ch - 2.0), size: dvec2(cw, 1.0) });
                }
            }
            screen_row += 1;
        }

        // Blinking cursor
        self.blink_counter += 1;
        if self.blink_counter % 15 == 0 { self.blink_on = !self.blink_on; }

        if self.scroll_offset == 0 && self.blink_on {
            let cursor_y = py + (screen_row.saturating_sub(1usize) as f64) * ch;
            let cursor_x = px + (grid.cur_c as f64) * cw;
            self.draw_cursor.color = vec4(0.345, 0.608, 0.976, 0.8);
            let cursor_w = if self.cursor_block { cw } else { 2.0 };
            self.draw_cursor.draw_abs(cx, Rect { pos: dvec2(cursor_x, cursor_y), size: dvec2(cursor_w, ch) });
        }

        // Focus border indicator (thin colored line on focused pane edge)
        if self.is_focused {
            self.draw_cursor.color = vec4(0.345, 0.608, 0.976, 0.6);
            // Top edge
            self.draw_cursor.draw_abs(cx, Rect { pos: dvec2(rect.pos.x, rect.pos.y), size: dvec2(rect.size.x, 2.0) });
        }

        DrawStep::done()
    }
}

impl TermViewRef {
    fn set_grid(&self, grid: Arc<Mutex<TermGrid>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.grid_ref = Some(grid);
            inner.scroll_offset = 0;
        }
    }
    fn set_cell_size(&self, cw: f64, ch: f64, cursor_block: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.cw = cw;
            inner.ch = ch;
            inner.cursor_block = cursor_block;
        }
    }
    fn reset_scroll(&self) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.scroll_offset = 0;
        }
    }
    fn set_visible(&self, cx: &mut Cx, visible: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.walk = if visible {
                Walk { width: Size::Fill, height: Size::Fill, ..inner.walk }
            } else {
                Walk { width: Size::Fixed(0.0), height: Size::Fixed(0.0), ..inner.walk }
            };
            inner.redraw(cx);
        }
    }
    fn set_search_highlights(&self, highlights: Vec<(usize, usize, usize)>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.search_highlights = highlights;
        }
    }
    fn set_focused(&self, focused: bool) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.is_focused = focused;
        }
    }
}

// ── Makepad App ────────────────────────────────────────────

live_design! {
    use link::theme::*;
    use link::widgets::*;

    TermView = {{TermView}} {
        width: Fill, height: Fill
        draw_bg: { color: #x1E1E1E, fn pixel(self) -> vec4 { return self.color; } }
        draw_cursor: { color: #x58A6FF, fn pixel(self) -> vec4 { return self.color; } }
        draw_cell_bg: { fn pixel(self) -> vec4 { return self.color; } }
        draw_text: {
            color: #xC5C8C6
            text_style: {
                font_size: 12.0
                line_spacing: 1.3
                font_family: {
                    // JetBrains Mono Nerd Font: powerline/devicon glyphs live in the
                    // Private Use Area, which LiberationMono lacks — p10k prompts and the
                    // tmux status bar render as boxes without it.
                    latin = font("makepad/leuwi_panjang/resources/JetBrainsMonoNerdFont-Regular.ttf", 0.0, 0.0)
                }
            }
        }
    }

    // One key on the modifier bar. Sized to stay tappable across 8 keys on a phone.
    KeyBtn = <Button> {
        text: ""
        width: Fill, height: 36
        draw_text: { color: #xC5C8C6, text_style: { font_size: 10.5 } }
        draw_bg: { color: #x30363D, fn pixel(self) -> vec4 { return mix(self.color, #x484F58, self.hover); } }
        padding: { left: 2, right: 2, top: 4, bottom: 4 }
    }

    // A tappable row in the burger menu. Wide and tall enough for a thumb.
    MenuRow = <Button> {
        text: ""
        width: Fill, height: 34
        align: { x: 0.0, y: 0.5 }
        draw_text: { color: #xC5C8C6, text_style: { font_size: 10.5 } }
        draw_bg: { color: #x21262D, fn pixel(self) -> vec4 { return mix(self.color, #x30363D, self.hover); } }
        padding: { left: 10, right: 8 }
    }

    FormLabel = <Label> {
        width: Fill
        text: ""
        draw_text: { color: #x6E7681, text_style: { font_size: 8.5 } }
    }

    FormInput = <TextInput> {
        width: Fill, height: 30
        draw_text: { color: #xE6EDF3, text_style: { font_size: 10.0 } }
        draw_bg: { color: #x0D1117 }
    }

    FormBtn = <Button> {
        text: ""
        width: Fill, height: 34
        draw_text: { color: #xE6EDF3, text_style: { font_size: 10.5 } }
        draw_bg: { color: #x238636, fn pixel(self) -> vec4 { return mix(self.color, #x2EA043, self.hover); } }
        margin: { top: 6 }
    }

    // Vertical tab entry (Zen-browser style sidebar). Full-width and tall enough to
    // stay comfortably tappable when many tabs are open, instead of cramming them
    // into a horizontal strip.
    VTabBtn = <Button> {
        text: ""
        width: Fill, height: 38
        align: { x: 0.0, y: 0.5 }
        draw_text: { color: #xC5C8C6, text_style: { font_size: 10.0 } }
        draw_bg: { color: #x1E1E1E, fn pixel(self) -> vec4 { return mix(self.color, #x30363D80, self.hover); } }
        padding: { left: 8, right: 6, top: 4, bottom: 4 }
    }

    // One touch-tappable tab button (Termius-style). Text + active marker are set
    // at runtime in update_tab_label; unused slots are hidden.
    TabBtn = <Button> {
        text: ""
        draw_text: { color: #xC5C8C6, text_style: { font_size: 9.5 } }
        draw_bg: { color: #x1E1E1E, fn pixel(self) -> vec4 { return mix(self.color, #x30363D80, self.hover); } }
        padding: { left: 12, right: 12, top: 4, bottom: 4 }
        margin: { right: 2 }
    }

    App = {{App}} {
        ui: <Window> {
            window: { title: "Leuwi Panjang", inner_size: vec2(1100, 700) }
            show_bg: true
            draw_bg: { color: #x1E1E1E }

            caption_bar = <SolidView> {
                visible: true, flow: Right, height: 32
                draw_bg: { color: #x181818 }
                caption_label = <View> { visible: false, width: 0, height: 0 }
                tabs = <View> {
                    width: Fill, height: Fill, flow: Right, align: { y: 0.5 }, padding: { left: 6 }
                    <View> { width: Fill, height: Fill }
                    sidebar_btn = <Button> {
                        // U+258C: the UI font lacks U+25A4, which rendered as blank.
                        text: "▌", width: 30
                        draw_text: { color: #x6E7681, text_style: { font_size: 13.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                    }
                    close_btn = <Button> {
                        text: "×", width: 26
                        draw_text: { color: #x6E7681, text_style: { font_size: 15.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #xE0303060, self.hover); } }
                    }
                    plus_btn = <Button> {
                        text: "+", width: 28
                        draw_text: { color: #x6E7681, text_style: { font_size: 13.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                    }
                    menu_btn = <Button> {
                        text: "≡", width: 32
                        draw_text: { color: #x6E7681, text_style: { font_size: 14.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                    }
                }
                windows_buttons = <View> {
                    visible: true, width: Fit, height: Fit, align: { y: 0.5 }
                    min = <DesktopButton> { draw_bg: { button_type: WindowsMin } }
                    max = <DesktopButton> { draw_bg: { button_type: WindowsMax } }
                    close = <DesktopButton> { draw_bg: { button_type: WindowsClose } }
                }
            }
            window_menu = <WindowMenu> { main = Main { items: [] } }
            body = <View> {
                width: Fill, height: Fill, flow: Down

                // Terminal panes area
                panes = <View> {
                    width: Fill, height: Fill, flow: Right
                    show_bg: true
                    draw_bg: { color: #x1E1E1E }
                    tab_sidebar = <View> {
                        width: 150, height: Fill, flow: Down
                        show_bg: true
                        draw_bg: { color: #x181818 }
                        padding: { left: 5, right: 5, top: 6 }
                        spacing: 4
                        tab0 = <VTabBtn> {}
                        tab1 = <VTabBtn> {}
                        tab2 = <VTabBtn> {}
                        tab3 = <VTabBtn> {}
                        tab4 = <VTabBtn> {}
                    }
                    // Wrapped so the config panel can hide the whole terminal area at
                    // once: set_visible has no effect on the TermView widget itself.
                    term_area = <View> {
                        width: Fill, height: Fill, flow: Right
                        terminal = <TermView> {}
                        split_bar = <View> {
                            visible: false
                            width: 2, height: Fill
                            show_bg: true
                            draw_bg: { color: #x444444 }
                        }
                        terminal2 = <TermView> {
                            width: 0, height: 0
                        }
                    }
                // Takes over the whole screen while open so the forms have room, and
                // scrolls because they are taller than a phone. Closing restores the terminal.
                // Full-screen settings page: vertical nav on the left picks the
                // config type, the right column holds that section's form and scrolls.
                menu_panel = <View> {
                    visible: false
                    width: Fill, height: Fill
                    flow: Right
                    show_bg: true
                    draw_bg: { color: #x161B22 }

                    cfg_nav = <View> {
                        width: 132, height: Fill, flow: Down
                        show_bg: true
                        draw_bg: { color: #x0D1117 }
                        padding: { top: 10, left: 6, right: 6 }
                        spacing: 4
                        menu_title = <Label> {
                            text: "Konfigurasi"
                            draw_text: { color: #x58A6FF, text_style: { font_size: 11.0 } }
                        }
                        menu_ver = <Label> {
                            text: "v0.1.0-dev"
                            draw_text: { color: #x6E7681, text_style: { font_size: 8.5 } }
                            margin: { bottom: 8 }
                        }
                        m_profile = <MenuRow> { text: "Profil" }
                        m_sshkey  = <MenuRow> { text: "SSH key" }
                        m_keymap  = <MenuRow> { text: "Keymap" }
                        <View> { width: Fill, height: Fill }
                        btn_close = <MenuRow> { text: "< Tutup" }
                    }

                    cfg_body = <ScrollYView> {
                        width: Fill, height: Fill, flow: Down
                        padding: { top: 10, left: 12, right: 12 }
                        spacing: 4

                        sec_profile = <View> {
                            visible: false
                            width: Fill, height: Fit, flow: Down, spacing: 3
                            <FormLabel> { text: "Nama perintah" }
                            in_name = <FormInput> { empty_text: "nvgpu" }
                            <FormLabel> { text: "Host / IP" }
                            in_host = <FormInput> { empty_text: "10.0.0.1" }
                            <FormLabel> { text: "Port" }
                            in_port = <FormInput> { empty_text: "22" }
                            <FormLabel> { text: "Username" }
                            in_user = <FormInput> { empty_text: "user" }
                            <FormLabel> { text: "Tmux session" }
                            in_session = <FormInput> { empty_text: "main" }
                            <FormLabel> { text: "Password (kosong = pakai SSH key)" }
                            in_pass = <FormInput> { is_password: true, empty_text: "" }
                            btn_save = <FormBtn> { text: "Simpan" }
                            form_msg = <Label> {
                                width: Fill
                                text: ""
                                draw_text: { color: #x6E7681, text_style: { font_size: 9.0 } }
                            }
                        }

                        sec_sshkey = <View> {
                            visible: false
                            width: Fill, height: Fit, flow: Down, spacing: 4
                            key_info = <Label> {
                                width: Fill
                                text: ""
                                draw_text: { color: #xC5C8C6, text_style: { font_size: 9.0 } }
                            }
                            btn_keygen = <FormBtn> { text: "Generate key baru" }
                            btn_showpub = <FormBtn> { text: "Tampilkan public key" }
                            key_pub = <Label> {
                                width: Fill
                                text: ""
                                draw_text: { color: #x7EE787, text_style: { font_size: 8.0 } }
                            }
                        }

                        sec_keymap = <View> {
                            visible: false
                            width: Fill, height: Fit, flow: Down
                            menu_content = <Label> {
                                width: Fill
                                text: ""
                                draw_text: { color: #xC5C8C6, text_style: { font_size: 9.5 } }
                            }
                        }
                    }
                }

                // Search bar (hidden by default)
                search_bar = <View> {
                    visible: false
                    width: Fill, height: 28, flow: Right
                    show_bg: true
                    draw_bg: { color: #x252526 }
                    align: { y: 0.5 }
                    padding: { left: 10, right: 10 }
                    search_icon = <Label> {
                        text: "🔍 "
                        draw_text: { color: #x6E7681, text_style: { font_size: 9.0 } }
                    }
                    search_label = <Label> {
                        text: ""
                        draw_text: { color: #xE0E0E0, text_style: { font_size: 10.0 } }
                    }
                    <View> { width: Fill, height: Fill }
                    search_info = <Label> {
                        text: ""
                        draw_text: { color: #x6E7681, text_style: { font_size: 9.0 } }
                    }
                }

                // Modifier/navigation key bar. The Android soft keyboard has no Esc,
                // Tab, Ctrl or arrows, which makes tmux and vim unusable; this supplies
                // them. Ctrl/Alt are sticky: tap to arm, then the next character is sent
                // with the modifier applied. Hidden on desktop, which has a real keyboard.
                key_bar = <View> {
                    width: Fill, height: 44, flow: Right
                    show_bg: true
                    draw_bg: { color: #x252526 }
                    align: { y: 0.5 }
                    padding: { left: 3, right: 3 }
                    spacing: 3
                    k_esc = <KeyBtn> { text: "Esc" }
                    k_tab = <KeyBtn> { text: "Tab" }
                    k_ctrl = <KeyBtn> { text: "Ctrl" }
                    k_alt = <KeyBtn> { text: "Alt" }
                    k_left = <KeyBtn> { text: "←" }
                    k_down = <KeyBtn> { text: "↓" }
                    k_up = <KeyBtn> { text: "↑" }
                    k_right = <KeyBtn> { text: "→" }
                }

                // Status bar
                status = <View> {
                    width: Fill, height: 22, flow: Right
                    show_bg: true
                    draw_bg: { color: #x181818 }
                    align: { y: 0.5 }
                    padding: { left: 10, right: 10 }
                    status_left = <Label> {
                        text: ""
                        draw_text: { color: #x6E7681, text_style: { font_size: 8.5 } }
                    }
                    <View> { width: Fill, height: Fill }
                    status_right = <Label> {
                        text: ""
                        draw_text: { color: #x6E7681, text_style: { font_size: 8.5 } }
                    }
                }

                }
            }
        }
    }
}

// ── App ────────────────────────────────────────────────────

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] tabs: Vec<TermTab>,
    #[rust] active_tab: usize,
    #[rust] started: bool,
    #[rust] boot_frames: u32,
    #[rust] tab_counter: usize,
    #[rust] split_active: bool,
    #[rust] config: Config,
    #[rust] theme: Theme,
    #[rust] key_handled: bool,
    #[rust] menu_open: bool,
    // Search state
    #[rust] search_open: bool,
    #[rust] search_query: String,
    #[rust] search_matches: Vec<(usize, usize)>,  // (abs_row, col)
    #[rust] search_idx: usize,
    // Sticky modifiers armed from the on-screen key bar (Android).
    // Vertical tab sidebar can be collapsed so it does not eat phone screen.
    #[rust] sidebar_hidden: bool,
    #[rust] ctrl_pending: bool,
    #[rust] alt_pending: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl App {
    fn init(&mut self, cx: &mut Cx) {
        if self.started { return; }
        self.started = true;
        self.config = Config::load();
        self.theme = Theme::load(&self.config.theme);
        self.tab_counter = 1;
        self.tabs.push(TermTab::spawn(1, &self.config));
        self.active_tab = 0;
        self.ui.term_view(id!(terminal)).set_grid(self.tabs[0].grid.clone());
        let block = self.config.cursor_style == "block";
        self.ui.term_view(id!(terminal)).set_cell_size(self.config.cell_width, self.config.cell_height, block);
        self.ui.term_view(id!(terminal2)).set_cell_size(self.config.cell_width, self.config.cell_height, block);
        self.update_tab_label(cx);
        cx.start_interval(0.033);
        // NOTE: do NOT call show_text_ime here — invoking the IME during Startup
        // (before the view/area exists) can crash on Android. The keyboard is
        // raised on the first tap of the terminal instead (see TermView touch).
    }

    fn new_tab(&mut self, cx: &mut Cx) {
        if self.tabs.len() >= 5 { return; }
        self.tab_counter += 1;
        self.tabs.push(TermTab::spawn(self.tab_counter, &self.config));
        self.active_tab = self.tabs.len() - 1;
        self.switch_to_active(cx);
    }

    fn close_active_tab(&mut self, cx: &mut Cx) {
        if self.tabs.len() <= 1 { return; }
        self.tabs.remove(self.active_tab);
        if self.active_tab >= self.tabs.len() { self.active_tab = self.tabs.len() - 1; }
        self.switch_to_active(cx);
    }

    fn next_tab(&mut self, cx: &mut Cx) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
        self.switch_to_active(cx);
    }

    fn switch_to_active(&mut self, cx: &mut Cx) {
        self.ui.term_view(id!(terminal)).set_grid(self.tabs[self.active_tab].grid.clone());
        self.show_tab_split(cx);
        self.update_tab_label(cx);
    }

    fn update_tab_label(&mut self, cx: &mut Cx) {
        // One discrete, tappable button per tab (up to 5); hide the unused slots.
        macro_rules! set_tab {
            ($id:tt, $i:expr) => {{
                if $i < self.tabs.len() {
                    let name = self.tabs[$i].dynamic_title();
                    let split = if self.tabs[$i].split.is_some() { " ⫽" } else { "" };
                    let txt = if $i == self.active_tab {
                        format!(" ▌ {}{} ", name, split)
                    } else {
                        format!("  {}{} ", name, split)
                    };
                    self.ui.button(id!($id)).set_text(cx, &txt);
                    self.ui.widget(id!($id)).set_visible(cx, true);
                } else {
                    self.ui.widget(id!($id)).set_visible(cx, false);
                }
            }};
        }
        set_tab!(tab0, 0);
        set_tab!(tab1, 1);
        set_tab!(tab2, 2);
        set_tab!(tab3, 3);
        set_tab!(tab4, 4);
    }

    /// Show which sticky modifiers are armed, so the bar reflects real state.
    fn update_modifier_labels(&mut self, cx: &mut Cx) {
        let ctrl = if self.ctrl_pending { "Ctrl*" } else { "Ctrl" };
        let alt = if self.alt_pending { "Alt*" } else { "Alt" };
        self.ui.button(id!(k_ctrl)).set_text(cx, ctrl);
        self.ui.button(id!(k_alt)).set_text(cx, alt);
        self.ui.redraw(cx);
    }

    /// Fold any armed sticky modifier into freshly typed text, then disarm it.
    fn apply_sticky_modifiers(&mut self, cx: &mut Cx, input: &str) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        if self.ctrl_pending {
            // Ctrl-<key> is the character with the upper bits cleared (a -> 0x01).
            let mut chars = input.chars();
            if let Some(c) = chars.next() {
                out.push((c.to_ascii_uppercase() as u8) & 0x1f);
            }
            out.extend_from_slice(chars.as_str().as_bytes());
        } else {
            out.extend_from_slice(input.as_bytes());
        }
        if self.alt_pending {
            // Alt-<key> is conventionally sent as ESC followed by the key.
            out.insert(0, 0x1b);
        }
        if self.ctrl_pending || self.alt_pending {
            self.ctrl_pending = false;
            self.alt_pending = false;
            self.update_modifier_labels(cx);
        }
        out
    }

    /// Switch to tab `i` by touch (or Alt+N). No-op if out of range.
    fn goto_tab(&mut self, cx: &mut Cx, i: usize) {
        if i < self.tabs.len() && i != self.active_tab {
            self.active_tab = i;
            self.switch_to_active(cx);
        }
    }

    fn do_split(&mut self, cx: &mut Cx, dir: SplitDir) {
        let tab = &self.tabs[self.active_tab];
        if tab.split.is_some() { return; }
        self.tab_counter += 1;
        let split = TermTab::spawn(self.tab_counter, &self.config);
        let block = self.config.cursor_style == "block";
        self.ui.term_view(id!(terminal2)).set_grid(split.grid.clone());
        self.ui.term_view(id!(terminal2)).set_cell_size(self.config.cell_width, self.config.cell_height, block);
        self.tabs[self.active_tab].split = Some(Box::new(split));
        self.tabs[self.active_tab].split_dir = dir;
        self.split_active = false; // focus stays on main pane
        // Set panes layout direction
        self.apply_split_layout(cx, dir);
        self.ui.view(id!(split_bar)).set_visible(cx, true);
        self.ui.term_view(id!(terminal2)).set_visible(cx, true);
    }

    fn apply_split_layout(&self, _cx: &mut Cx, _dir: SplitDir) {
        // Makepad flow direction is set at live_design time (Right).
        // For horizontal split we'd need Down flow — but since Makepad
        // live_design is static, we handle this via terminal2 walk sizing
        // in handle_resize instead. Both directions use the same two TermViews.
    }

    fn close_split(&mut self, cx: &mut Cx) {
        self.tabs[self.active_tab].split = None;
        self.split_active = false;
        self.ui.view(id!(split_bar)).set_visible(cx, false);
        self.ui.term_view(id!(terminal2)).set_visible(cx, false);
    }

    fn toggle_split_focus(&mut self) {
        if self.tabs[self.active_tab].split.is_some() {
            self.split_active = !self.split_active;
        }
    }

    fn show_tab_split(&mut self, cx: &mut Cx) {
        let tab = &self.tabs[self.active_tab];
        if let Some(split) = &tab.split {
            self.ui.term_view(id!(terminal2)).set_grid(split.grid.clone());
            self.ui.view(id!(split_bar)).set_visible(cx, true);
            self.ui.term_view(id!(terminal2)).set_visible(cx, true);
        } else {
            self.ui.view(id!(split_bar)).set_visible(cx, false);
            self.ui.term_view(id!(terminal2)).set_visible(cx, false);
            self.split_active = false;
        }
    }

    fn copy_to_clipboard(&self) {
        if let Some(tab) = self.tabs.get(self.active_tab) {
            let grid = tab.grid.lock().unwrap_or_else(|e| e.into_inner());
            // Try selection first, fallback to all visible text
            let text = grid.get_selection_text().unwrap_or_else(|| grid.render());
            drop(grid);
            if !text.is_empty() {
                clip::set(&text);
            }
        }
    }

    fn paste_from_clipboard(&mut self) {
        let text = clip::get();
        if text.is_empty() { return; }
        if self.split_active {
            if let Some(split) = &mut self.tabs[self.active_tab].split {
                split.write(b"\x1b[200~");
                split.write(text.as_bytes());
                split.write(b"\x1b[201~");
            }
        } else if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.write(b"\x1b[200~");
            tab.write(text.as_bytes());
            tab.write(b"\x1b[201~");
        }
    }

    // ── Search ──────────────────────────────────────────────
    fn open_search(&mut self, cx: &mut Cx) {
        self.search_open = true;
        self.search_query.clear();
        self.search_matches.clear();
        self.search_idx = 0;
        self.ui.view(id!(search_bar)).set_visible(cx, true);
        self.ui.label(id!(search_label)).set_text(cx, "│");
        self.ui.label(id!(search_info)).set_text(cx, "Type to search · Esc to close");
        self.ui.redraw(cx);
    }

    fn close_search(&mut self, cx: &mut Cx) {
        self.search_open = false;
        self.search_query.clear();
        self.search_matches.clear();
        self.ui.view(id!(search_bar)).set_visible(cx, false);
        self.ui.redraw(cx);
    }

    fn update_search(&mut self, cx: &mut Cx) {
        if self.search_query.is_empty() {
            self.search_matches.clear();
            self.search_idx = 0;
            self.ui.label(id!(search_label)).set_text(cx, "│");
            self.ui.label(id!(search_info)).set_text(cx, "Type to search · Esc to close");
            return;
        }
        if let Some(tab) = self.tabs.get(self.active_tab) {
            let grid = tab.grid.lock().unwrap_or_else(|e| e.into_inner());
            self.search_matches = grid.search(&self.search_query);
        }
        let n = self.search_matches.len();
        if n > 0 {
            self.search_idx = self.search_idx.min(n - 1);
            let info = format!("{}/{} · Enter↓ Shift+Enter↑ · Esc close", self.search_idx + 1, n);
            self.ui.label(id!(search_info)).set_text(cx, &info);
            // Scroll to current match
            self.scroll_to_match(cx);
        } else {
            self.ui.label(id!(search_info)).set_text(cx, "No matches · Esc close");
        }
        self.ui.label(id!(search_label)).set_text(cx, &format!("{}│", self.search_query));
    }

    fn search_next(&mut self, cx: &mut Cx) {
        if self.search_matches.is_empty() { return; }
        self.search_idx = (self.search_idx + 1) % self.search_matches.len();
        let n = self.search_matches.len();
        let info = format!("{}/{} · Enter↓ Shift+Enter↑ · Esc close", self.search_idx + 1, n);
        self.ui.label(id!(search_info)).set_text(cx, &info);
        self.scroll_to_match(cx);
    }

    fn search_prev(&mut self, cx: &mut Cx) {
        if self.search_matches.is_empty() { return; }
        self.search_idx = if self.search_idx == 0 { self.search_matches.len() - 1 } else { self.search_idx - 1 };
        let n = self.search_matches.len();
        let info = format!("{}/{} · Enter↓ Shift+Enter↑ · Esc close", self.search_idx + 1, n);
        self.ui.label(id!(search_info)).set_text(cx, &info);
        self.scroll_to_match(cx);
    }

    fn scroll_to_match(&self, _cx: &mut Cx) {
        if let Some(&(abs_row, _col)) = self.search_matches.get(self.search_idx) {
            if let Some(inner) = self.ui.term_view(id!(terminal)).borrow_mut() {
                let grid = match &inner.grid_ref {
                    Some(g) => g.lock().unwrap_or_else(|e| e.into_inner()),
                    None => return,
                };
                let sb = grid.scrollback.len();
                let total = sb + grid.rows;
                let from_bottom = total.saturating_sub(abs_row);
                drop(grid);
                drop(inner);
                // Set scroll offset to show the match
                if let Some(mut inner) = self.ui.term_view(id!(terminal)).borrow_mut() {
                    inner.scroll_offset = from_bottom.saturating_sub(5) as i64;
                }
            }
        }
    }

    // ── Git branch detection ───────────────────────────────
    fn detect_git_branch() -> String {
        // Try to read .git/HEAD in CWD or parents
        let mut dir = std::env::current_dir().unwrap_or_default();
        for _ in 0..10 {
            let head = dir.join(".git/HEAD");
            if head.exists() {
                if let Ok(content) = std::fs::read_to_string(&head) {
                    let content = content.trim();
                    if let Some(branch) = content.strip_prefix("ref: refs/heads/") {
                        return format!(" {}", branch);
                    }
                    // Detached HEAD
                    return format!(" {}", &content[..7.min(content.len())]);
                }
            }
            if !dir.pop() { break; }
        }
        String::new()
    }

    /// Open or close the config panel. It covers the terminal so the forms are
    /// readable on a phone; closing brings the terminal back exactly as it was.
    fn set_menu_open(&mut self, cx: &mut Cx, open: bool) {
        self.menu_open = open;
        self.ui.view(id!(menu_panel)).set_visible(cx, open);
        // Hiding the whole panes row (terminal + tab sidebar) lets the settings
        // page own the screen; the key bar goes too since there is nothing to type into.
        self.ui.view(id!(panes)).set_visible(cx, !open);
        self.ui.view(id!(key_bar)).set_visible(cx, !open && cfg!(target_os = "android"));
        self.ui.widget(id!(tab_sidebar)).set_visible(cx, !open && !self.sidebar_hidden);
        if open {
            self.show_menu_section(cx, "profile");
        }
        self.ui.redraw(cx);
    }

    /// Swap which burger-menu section is on screen and fill it with current values.
    fn show_menu_section(&mut self, cx: &mut Cx, which: &str) {
        self.ui.view(id!(sec_profile)).set_visible(cx, which == "profile");
        self.ui.view(id!(sec_sshkey)).set_visible(cx, which == "sshkey");
        self.ui.view(id!(sec_keymap)).set_visible(cx, which == "keymap");

        match which {
            "profile" => {
                let host = self.config.ssh_host.clone();
                let port = self.config.ssh_port.to_string();
                let user = self.config.ssh_user.clone();
                let session = self.config.ssh_session.clone();
                let pass = self.config.ssh_password.clone();
                self.ui.widget(id!(in_name)).set_text(cx, "nvgpu");
                self.ui.widget(id!(in_host)).set_text(cx, &host);
                self.ui.widget(id!(in_port)).set_text(cx, &port);
                self.ui.widget(id!(in_user)).set_text(cx, &user);
                self.ui.widget(id!(in_session)).set_text(cx, &session);
                self.ui.widget(id!(in_pass)).set_text(cx, &pass);
                self.ui.widget(id!(form_msg)).set_text(cx, "");
            }
            "sshkey" => {
                let path = Self::ssh_key_path_display();
                let exists = std::path::Path::new(&path).is_file();
                let info = format!(
                    "key: {}\nstatus: {}",
                    path,
                    if exists { "ada" } else { "belum ada — tekan Generate" }
                );
                self.ui.widget(id!(key_info)).set_text(cx, &info);
                self.ui.widget(id!(key_pub)).set_text(cx, "");
            }
            _ => self.show_menu(cx),
        }
        self.ui.redraw(cx);
    }

    /// Where the SSH key lives, as text for the key section.
    fn ssh_key_path_display() -> String {
        #[cfg(target_os = "android")]
        {
            ssh::default_key_path().display().to_string()
        }
        #[cfg(not(target_os = "android"))]
        {
            "~/.ssh/id_rsa".to_string()
        }
    }

    /// Persist what the user typed in the connection form.
    fn save_profile_form(&mut self, cx: &mut Cx) {
        let host = self.ui.widget(id!(in_host)).text();
        let port = self.ui.widget(id!(in_port)).text();
        let user = self.ui.widget(id!(in_user)).text();
        let session = self.ui.widget(id!(in_session)).text();
        let pass = self.ui.widget(id!(in_pass)).text();

        if !port.trim().is_empty() {
            match port.trim().parse::<u16>() {
                Ok(p) => self.config.ssh_port = p,
                Err(_) => {
                    self.ui.widget(id!(form_msg)).set_text(cx, "port harus angka 1-65535");
                    self.ui.redraw(cx);
                    return;
                }
            }
        }
        if !host.trim().is_empty() { self.config.ssh_host = host.trim().to_string(); }
        if !user.trim().is_empty() { self.config.ssh_user = user.trim().to_string(); }
        if !session.trim().is_empty() { self.config.ssh_session = session.trim().to_string(); }
        self.config.ssh_password = pass;

        let msg = match self.config.save() {
            Ok(()) => "tersimpan — berlaku untuk tab baru".to_string(),
            Err(e) => format!("gagal: {e}"),
        };
        self.ui.widget(id!(form_msg)).set_text(cx, &msg);
        self.ui.redraw(cx);
    }

    /// Generate a fresh keypair and show the public half to paste into the server.
    fn run_keygen(&mut self, cx: &mut Cx) {
        #[cfg(target_os = "android")]
        {
            let path = ssh::default_key_path();
            let text = match ssh::generate_key(&path, "leuwi-panjang-android") {
                Ok(pubkey) => format!("{}\n\nsalin ke ~/.ssh/authorized_keys di server", pubkey),
                Err(e) => format!("gagal: {e}"),
            };
            self.ui.widget(id!(key_pub)).set_text(cx, &text);
            self.show_menu_section(cx, "sshkey");
            self.ui.widget(id!(key_pub)).set_text(cx, &text);
        }
        #[cfg(not(target_os = "android"))]
        {
            self.ui.widget(id!(key_pub)).set_text(cx, "keygen hanya di Android");
        }
        self.ui.redraw(cx);
    }

    /// Show the existing public key without regenerating anything.
    fn show_public_key(&mut self, cx: &mut Cx) {
        let path = Self::ssh_key_path_display();
        let pub_path = format!("{}.pub", path);
        let text = match std::fs::read_to_string(&pub_path) {
            Ok(k) => k.trim().to_string(),
            Err(e) => format!("tidak bisa baca {}: {}", pub_path, e),
        };
        self.ui.widget(id!(key_pub)).set_text(cx, &text);
        self.ui.redraw(cx);
    }

    fn show_menu(&self, cx: &mut Cx) {
        let menu = format!(
"━━━ KEY MAP ━━━━━━━━━━━━━━━━━━━━

 TABS  (each tab = its own nvgpu tmux session)
  New Tab           tap +  /  Ctrl+Shift+T
  Switch Tab        tap tab / Alt+1..5
  Close Tab         tap ×  /  Ctrl+Shift+W
  Next Tab          Ctrl+Tab

 SPLIT SCREEN
  Split Vertical    Ctrl+Shift+D
  Split Horizontal  Ctrl+Shift+E
  Switch Pane       Alt+Left/Right
  Close Split       Ctrl+Shift+W

 SEARCH
  Find              Ctrl+Shift+F
  Next Match        Enter
  Prev Match        Shift+Enter
  Close Search      Esc

 CLIPBOARD
  Copy (selection)  Ctrl+Shift+C
  Paste             Ctrl+Shift+V
  Select All        Ctrl+Shift+A

 NAVIGATION
  Scroll (phone)    drag 1 finger
  Show keyboard     tap terminal
  Scroll Up/Down    Mouse Wheel
  Select Text       Mouse Drag
  Open URL          Ctrl+Click

 MENU
  Open/Close Menu   Ctrl+Shift+M

 WINDOW
  Close App         Alt+F4
  Cancel/Interrupt  Ctrl+C
  Clear Screen      Ctrl+L

━━━ CONFIG ━━━━━━━━━━━━━━━━━━━━━
  Shell: {}
  Theme: {}
  Cell: {}x{} | Grid: {}x{}
  Scrollback: {} | Cursor: {}
  ~/.config/leuwi-panjang/config.toml

━━━ ABOUT ━━━━━━━━━━━━━━━━━━━━━━
  Leuwi Panjang Terminal v0.1.0-dev.20
  Rust + Makepad | GPL-3.0
  github.com/situkangsayur/
    leuwi-panjang

  Esc: close menu",
            self.config.shell, self.config.theme,
            self.config.cell_width, self.config.cell_height,
            self.config.cols, self.config.rows,
            self.config.scrollback, self.config.cursor_style,
        );
        self.ui.label(id!(menu_content)).set_text(cx, &menu);
    }

    fn write_to_active(&mut self, data: &[u8]) {
        if self.split_active {
            if let Some(split) = &mut self.tabs[self.active_tab].split {
                split.write(data);
            }
        } else {
            let active = self.active_tab;
            // On Android a tab sits at the built-in prompt until a session starts,
            // so route keystrokes there first.
            #[cfg(target_os = "android")]
            {
                let cfg = &mut self.config;
                if let Some(tab) = self.tabs.get_mut(active) {
                    if tab.repl_active() {
                        tab.repl_feed(data, cfg, active);
                        return;
                    }
                }
            }
            if let Some(tab) = self.tabs.get_mut(active) {
                tab.write(data);
            }
        }
    }

    fn handle_resize(&mut self, width: f64, height: f64) {
        let chrome_h = 32.0 + 22.0; // caption + status bar
        let search_h = if self.search_open { 28.0 } else { 0.0 };
        // The vertical tab sidebar and (on Android) the modifier key bar take space away
        // from the grid. Without accounting for them the terminal reflows wider/taller
        // than its pane and the text runs off the edge.
        let sidebar_w = if self.sidebar_hidden { 0.0 } else { 150.0 };
        let key_bar_h = if cfg!(target_os = "android") { 44.0 } else { 0.0 };
        let cw = self.config.cell_width;
        let ch = self.config.cell_height;
        let avail_w = width - 24.0 - sidebar_w;
        let avail_h = height - chrome_h - search_h - key_bar_h;
        let full_cols = (avail_w / cw).max(20.0) as usize;
        let full_rows = (avail_h / ch).max(5.0) as usize;

        // Update config with actual computed size
        self.config.cols = full_cols;
        self.config.rows = full_rows;

        for tab in &mut self.tabs {
            let has_split = tab.split.is_some();
            let dir = tab.split_dir;
            match (has_split, dir) {
                (true, SplitDir::Vertical) => {
                    let half = (full_cols.saturating_sub(1)) / 2;
                    // Resize main grid directly
                    tab.grid.lock().unwrap_or_else(|e| e.into_inner()).resize(half, full_rows);
                    tab.grid.lock().unwrap_or_else(|e| e.into_inner()).scroll_bottom = full_rows.saturating_sub(1);
                    if let Some(ssh) = &tab.ssh { ssh.resize(half as u16, full_rows as u16); }
                    #[cfg(not(target_os = "android"))]
                    if let Some(m) = &tab.master {
                        let _ = m.resize(portable_pty::PtySize { rows: full_rows as u16, cols: half as u16, pixel_width: 0, pixel_height: 0 });
                    }
                    if let Some(s) = &mut tab.split { s.resize(half, full_rows); }
                }
                (true, SplitDir::Horizontal) => {
                    let half = (full_rows.saturating_sub(1)) / 2;
                    tab.grid.lock().unwrap_or_else(|e| e.into_inner()).resize(full_cols, half);
                    tab.grid.lock().unwrap_or_else(|e| e.into_inner()).scroll_bottom = half.saturating_sub(1);
                    if let Some(ssh) = &tab.ssh { ssh.resize(full_cols as u16, half as u16); }
                    #[cfg(not(target_os = "android"))]
                    if let Some(m) = &tab.master {
                        let _ = m.resize(portable_pty::PtySize { rows: half as u16, cols: full_cols as u16, pixel_width: 0, pixel_height: 0 });
                    }
                    if let Some(s) = &mut tab.split { s.resize(full_cols, half); }
                }
                _ => {
                    tab.resize(full_cols, full_rows);
                }
            }
        }
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Touch: tap a tab to switch (Termius-style tab bar).
        if self.ui.button(id!(tab0)).clicked(&actions) { self.goto_tab(cx, 0); }
        if self.ui.button(id!(tab1)).clicked(&actions) { self.goto_tab(cx, 1); }
        if self.ui.button(id!(tab2)).clicked(&actions) { self.goto_tab(cx, 2); }
        if self.ui.button(id!(tab3)).clicked(&actions) { self.goto_tab(cx, 3); }
        if self.ui.button(id!(tab4)).clicked(&actions) { self.goto_tab(cx, 4); }

        // On-screen modifier bar: Esc/Tab/arrows send immediately; Ctrl and Alt arm a
        // sticky modifier that is applied to the next character typed.
        // Burger menu is a real navigator now: each row swaps the section below it.
        if self.ui.button(id!(btn_close)).clicked(&actions) { self.set_menu_open(cx, false); }
        if self.ui.button(id!(m_profile)).clicked(&actions) { self.show_menu_section(cx, "profile"); }
        if self.ui.button(id!(m_sshkey)).clicked(&actions) { self.show_menu_section(cx, "sshkey"); }
        if self.ui.button(id!(m_keymap)).clicked(&actions) { self.show_menu_section(cx, "keymap"); }
        if self.ui.button(id!(btn_save)).clicked(&actions) { self.save_profile_form(cx); }
        if self.ui.button(id!(btn_keygen)).clicked(&actions) { self.run_keygen(cx); }
        if self.ui.button(id!(btn_showpub)).clicked(&actions) { self.show_public_key(cx); }
        if self.ui.button(id!(sidebar_btn)).clicked(&actions) {
            self.sidebar_hidden = !self.sidebar_hidden;
            self.ui.widget(id!(tab_sidebar)).set_visible(cx, !self.sidebar_hidden);
            self.ui.redraw(cx);
        }
        if self.ui.button(id!(k_esc)).clicked(&actions) { self.write_to_active(b"\x1b"); }
        if self.ui.button(id!(k_tab)).clicked(&actions) { self.write_to_active(b"\t"); }
        if self.ui.button(id!(k_left)).clicked(&actions) { self.write_to_active(b"\x1b[D"); }
        if self.ui.button(id!(k_down)).clicked(&actions) { self.write_to_active(b"\x1b[B"); }
        if self.ui.button(id!(k_up)).clicked(&actions) { self.write_to_active(b"\x1b[A"); }
        if self.ui.button(id!(k_right)).clicked(&actions) { self.write_to_active(b"\x1b[C"); }
        if self.ui.button(id!(k_ctrl)).clicked(&actions) {
            self.ctrl_pending = !self.ctrl_pending;
            self.update_modifier_labels(cx);
        }
        if self.ui.button(id!(k_alt)).clicked(&actions) {
            self.alt_pending = !self.alt_pending;
            self.update_modifier_labels(cx);
        }
        // Touch: × closes the active tab (keeps at least one open).
        if self.ui.button(id!(close_btn)).clicked(&actions) {
            if self.tabs.len() > 1 { self.close_active_tab(cx); }
        }
        if self.ui.button(id!(plus_btn)).clicked(&actions) {
            self.new_tab(cx);
        }
        if self.ui.button(id!(menu_btn)).clicked(&actions) {
            let open = !self.menu_open;
            self.set_menu_open(cx, open);
            self.ui.redraw(cx);
        }
    }
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());
        match event {
            Event::Startup => { self.init(cx); }
            Event::Timer(_) => {
                // Start deferred SSH connections ~0.5s after launch (off the
                // startup/first-paint path). New tabs connect on the next tick.
                self.boot_frames = self.boot_frames.saturating_add(1);
                #[cfg(not(target_os = "android"))]
                if self.boot_frames == 1 {
                    self.ui.widget(id!(key_bar)).set_visible(cx, false);
                }
                if self.boot_frames > 15 {
                    for tab in &mut self.tabs {
                        tab.start_pending_ssh();
                    }
                }
                // A finished SSH session hands the tab back to the built-in prompt.
                #[cfg(target_os = "android")]
                for tab in &mut self.tabs {
                    tab.reap_finished_ssh();
                }
                // Flush response buffer (DA, DSR replies to PTY)
                if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                    let mut g = tab.grid.lock().unwrap_or_else(|e| e.into_inner());
                    if !g.response_buf.is_empty() {
                        let resp = g.response_buf.drain(..).collect::<Vec<u8>>();
                        drop(g);
                        tab.write(&resp);
                    }
                }
                // Visual bell check
                if let Some(tab) = self.tabs.get(self.active_tab) {
                    let mut g = tab.grid.lock().unwrap_or_else(|e| e.into_inner());
                    if g.bell {
                        g.bell = false;
                        // Flash effect — briefly change status bar
                        self.ui.label(id!(status_left)).set_text(cx, "🔔 BELL");
                    }
                }
                self.update_tab_label(cx);
                // Status bar — left: git + CWD + split info
                let git = App::detect_git_branch();
                let cwd = std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();
                let cwd_short = if let Some(home) = dirs::home_dir() {
                    cwd.replace(&home.to_string_lossy().to_string(), "~")
                } else { cwd };
                let split_info = match self.tabs.get(self.active_tab) {
                    Some(t) if t.split.is_some() => match t.split_dir {
                        SplitDir::Vertical => " ⫽V",
                        SplitDir::Horizontal => " ⫽H",
                    },
                    _ => "",
                };
                let focus = if self.split_active { "→2" } else if self.tabs.get(self.active_tab).map(|t| t.split.is_some()).unwrap_or(false) { "→1" } else { "" };
                let left = format!("{} {}  Tab {}/{}{}  {}",
                    git, cwd_short,
                    self.active_tab + 1, self.tabs.len(),
                    split_info, focus,
                );
                self.ui.label(id!(status_left)).set_text(cx, left.trim());
                // Status bar — right: cols x rows + clock
                let now = {
                    let t = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    let h = (t / 3600) % 24;
                    let m = (t / 60) % 60;
                    // Adjust for local timezone offset (approximate)
                    format!("{:02}:{:02}", h, m)
                };
                let right = format!("{}×{}  {}", self.config.cols, self.config.rows, now);
                self.ui.label(id!(status_right)).set_text(cx, &right);
                // Update pane focus indicators
                let has_split = self.tabs.get(self.active_tab).map(|t| t.split.is_some()).unwrap_or(false);
                if has_split {
                    self.ui.term_view(id!(terminal)).set_focused(!self.split_active);
                    self.ui.term_view(id!(terminal2)).set_focused(self.split_active);
                } else {
                    self.ui.term_view(id!(terminal)).set_focused(false);
                    self.ui.term_view(id!(terminal2)).set_focused(false);
                }
                // Update search highlights on terminal
                if self.search_open && !self.search_matches.is_empty() {
                    let qlen = self.search_query.len();
                    let highlights: Vec<(usize, usize, usize)> = self.search_matches.iter().map(|&(r, c)| (r, c, qlen)).collect();
                    self.ui.term_view(id!(terminal)).set_search_highlights(highlights);
                } else {
                    self.ui.term_view(id!(terminal)).set_search_highlights(Vec::new());
                }
                self.ui.redraw(cx);
            }
            // Button actions are handled in MatchEvent::handle_actions (see above)
            // via self.match_event(cx, event); no duplicate handling here.
            Event::KeyDown(ke) => {
                // Escape: close search or menu if open, otherwise send to PTY
                if ke.key_code == KeyCode::Escape {
                    if self.search_open {
                        self.close_search(cx);
                        return;
                    }
                    if self.menu_open {
                        self.set_menu_open(cx, false);
                        self.ui.redraw(cx);
                        return;
                    }
                    // Send ESC to PTY (vim mode switch)
                    self.write_to_active(&[0x1b]);
                    self.key_handled = true;
                    return;
                }
                // Search mode key handling
                if self.search_open {
                    match ke.key_code {
                        KeyCode::ReturnKey => {
                            if ke.modifiers.shift { self.search_prev(cx); } else { self.search_next(cx); }
                            self.key_handled = true;
                            return;
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.update_search(cx);
                            self.key_handled = true;
                            return;
                        }
                        _ => {} // printable chars handled in TextInput
                    }
                }
                // Ctrl+Shift shortcuts
                if ke.modifiers.control && ke.modifiers.shift {
                    match ke.key_code {
                        KeyCode::KeyT => { self.new_tab(cx); return; }
                        KeyCode::KeyW => {
                            if self.tabs[self.active_tab].split.is_some() && self.split_active {
                                self.close_split(cx);
                            } else if self.tabs.len() > 1 {
                                self.close_active_tab(cx);
                            }
                            // Never close the last tab — use Alt+F4 to close app
                            return;
                        }
                        KeyCode::KeyC => { self.copy_to_clipboard(); return; }
                        KeyCode::KeyV => { self.paste_from_clipboard(); return; }
                        KeyCode::KeyD => { self.do_split(cx, SplitDir::Vertical); return; }
                        KeyCode::KeyE => { self.do_split(cx, SplitDir::Horizontal); return; }
                        KeyCode::KeyA => {
                            // Select all
                            if let Some(tab) = self.tabs.get(self.active_tab) {
                                tab.grid.lock().unwrap_or_else(|e| e.into_inner()).select_all();
                            }
                            self.ui.redraw(cx);
                            return;
                        }
                        KeyCode::KeyM => {
                            // Toggle menu
                            let open = !self.menu_open;
                            self.set_menu_open(cx, open);
                            self.ui.redraw(cx);
                            return;
                        }
                        KeyCode::KeyF => {
                            if self.search_open { self.close_search(cx); } else { self.open_search(cx); }
                            return;
                        }
                        _ => {}
                    }
                }
                // Ctrl+Tab = next tab
                if ke.modifiers.control && ke.key_code == KeyCode::Tab {
                    self.next_tab(cx);
                    return;
                }
                // F11 = fullscreen
                if ke.key_code == KeyCode::F11 {
                    // Makepad doesn't have direct fullscreen toggle
                    // but we can maximize
                    return;
                }
                // Alt+F4 = close window
                if ke.modifiers.alt && ke.key_code == KeyCode::F4 {
                    cx.quit();
                    return;
                }
                // Alt shortcuts
                if ke.modifiers.alt {
                    match ke.key_code {
                        // Alt+Arrow = switch panes in split
                        KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                            self.toggle_split_focus();
                            return;
                        }
                        // Alt+1-5 = switch to tab N
                        KeyCode::Key1 => { if self.tabs.len() > 0 { self.active_tab = 0; self.switch_to_active(cx); } return; }
                        KeyCode::Key2 => { if self.tabs.len() > 1 { self.active_tab = 1; self.switch_to_active(cx); } return; }
                        KeyCode::Key3 => { if self.tabs.len() > 2 { self.active_tab = 2; self.switch_to_active(cx); } return; }
                        KeyCode::Key4 => { if self.tabs.len() > 3 { self.active_tab = 3; self.switch_to_active(cx); } return; }
                        KeyCode::Key5 => { if self.tabs.len() > 4 { self.active_tab = 4; self.switch_to_active(cx); } return; }
                        _ => {}
                    }
                }
                // Forward ONLY special/control keys via KeyDown
                // Printable chars come via TextInput (no double-send)
                // Clear selection AFTER processing Ctrl+Shift+C (copy needs selection)
                self.key_handled = false;
                let app_cur = self.tabs.get(self.active_tab)
                    .map(|t| t.grid.lock().unwrap_or_else(|e| e.into_inner()).app_cursor_keys).unwrap_or(false);
                let b = key_to_special_bytes(ke, app_cur);
                if !b.is_empty() {
                    self.key_handled = true;
                    // Clear selection when typing (not for Ctrl+Shift combos which are handled above)
                    if let Some(tab) = self.tabs.get(self.active_tab) {
                        tab.grid.lock().unwrap_or_else(|e| e.into_inner()).clear_select();
                    }
                    self.write_to_active(&b);
                    self.ui.term_view(id!(terminal)).reset_scroll();
                    if self.split_active {
                        self.ui.term_view(id!(terminal2)).reset_scroll();
                    }
                }
            }
            Event::TextInput(te) => {
                // ALL printable input comes here (handles shift, layout, etc.)
                if self.key_handled {
                    self.key_handled = false;
                } else if self.search_open {
                    // Send chars to search bar
                    if !te.input.is_empty() && !te.was_paste {
                        self.search_query.push_str(&te.input);
                        self.update_search(cx);
                    }
                } else if !te.input.is_empty() && !te.was_paste {
                    let input = te.input.clone();
                    let bytes = self.apply_sticky_modifiers(cx, &input);
                    self.write_to_active(&bytes);
                    self.ui.term_view(id!(terminal)).reset_scroll();
                }
            }
            Event::MouseDown(me) => {
                // Detect clicks in caption bar area (y < 32)
                if me.abs.y < 32.0 {
                    let win_w = 1100.0; // approximate, TODO: get actual
                    // ≡ button area: right side before window controls (~120px from right)
                    if me.abs.x > win_w - 180.0 && me.abs.x < win_w - 140.0 {
                        let open = !self.menu_open;
                        self.set_menu_open(cx, open);
                        self.ui.redraw(cx);
                    }
                    // + button area
                    if me.abs.x > win_w - 220.0 && me.abs.x < win_w - 185.0 {
                        self.new_tab(cx);
                    }
                } else if self.menu_open {
                    self.set_menu_open(cx, false);
                    self.ui.redraw(cx);
                } else {
                    // Send mouse click to PTY if mouse reporting enabled
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        let g = tab.grid.lock().unwrap_or_else(|e| e.into_inner());
                        if g.mouse_reporting {
                            let cw = self.config.cell_width;
                            let ch = self.config.cell_height;
                            let col = ((me.abs.x - 12.0) / cw).max(0.0) as usize + 1; // 1-based
                            let row = ((me.abs.y - 32.0) / ch).max(0.0) as usize + 1;
                            drop(g);
                            // SGR mouse: ESC [ < 0 ; col ; row M
                            let seq = format!("\x1b[<0;{};{}M", col, row);
                            tab.write(seq.as_bytes());
                        }
                    }
                }
            }
            Event::MouseUp(me) => {
                // Send mouse release to PTY if reporting
                if me.abs.y > 32.0 {
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        let g = tab.grid.lock().unwrap_or_else(|e| e.into_inner());
                        if g.mouse_reporting {
                            let cw = self.config.cell_width;
                            let ch = self.config.cell_height;
                            let col = ((me.abs.x - 12.0) / cw).max(0.0) as usize + 1;
                            let row = ((me.abs.y - 32.0) / ch).max(0.0) as usize + 1;
                            drop(g);
                            let seq = format!("\x1b[<0;{};{}m", col, row); // lowercase m = release
                            tab.write(seq.as_bytes());
                        }
                    }
                }
            }
            Event::WindowGeomChange(ev) => {
                let size = ev.new_geom.inner_size;
                self.handle_resize(size.x, size.y);
            }
            _ => {}
        }
    }
}

/// Only handle special/control keys — printable chars come via TextInput
fn key_to_special_bytes(ke: &KeyEvent, app_cursor: bool) -> Vec<u8> {
    if ke.modifiers.control {
        if let Some(c) = kc_char(&ke.key_code) {
            let c = c.to_ascii_lowercase();
            if ('a'..='z').contains(&c) { return vec![(c as u8) - b'a' + 1]; }
        }
    }
    // Arrow keys: ESC [ X (normal) or ESC O X (application mode / DECCKM)
    let arrow_prefix = if app_cursor { b'O' } else { b'[' };
    match ke.key_code {
        KeyCode::ReturnKey => vec![13],
        KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![9],
        KeyCode::Escape => vec![27],
        KeyCode::ArrowUp => vec![27, arrow_prefix, b'A'],
        KeyCode::ArrowDown => vec![27, arrow_prefix, b'B'],
        KeyCode::ArrowRight => vec![27, arrow_prefix, b'C'],
        KeyCode::ArrowLeft => vec![27, arrow_prefix, b'D'],
        KeyCode::Home => vec![27, b'[', b'H'],
        KeyCode::End => vec![27, b'[', b'F'],
        KeyCode::PageUp => vec![27, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![27, b'[', b'6', b'~'],
        KeyCode::Delete => vec![27, b'[', b'3', b'~'],
        KeyCode::F1 => vec![27, b'O', b'P'],
        KeyCode::F2 => vec![27, b'O', b'Q'],
        KeyCode::F3 => vec![27, b'O', b'R'],
        KeyCode::F4 => vec![27, b'O', b'S'],
        KeyCode::F5 => vec![27, b'[', b'1', b'5', b'~'],
        KeyCode::F6 => vec![27, b'[', b'1', b'7', b'~'],
        KeyCode::F7 => vec![27, b'[', b'1', b'8', b'~'],
        KeyCode::F8 => vec![27, b'[', b'1', b'9', b'~'],
        KeyCode::F9 => vec![27, b'[', b'2', b'0', b'~'],
        KeyCode::F10 => vec![27, b'[', b'2', b'1', b'~'],
        KeyCode::F11 => vec![27, b'[', b'2', b'3', b'~'],
        KeyCode::F12 => vec![27, b'[', b'2', b'4', b'~'],
        // Don't handle printable chars — they come via TextInput
        _ => vec![],
    }
}

fn kc_char(kc: &KeyCode) -> Option<char> {
    match kc {
        KeyCode::KeyA => Some('a'), KeyCode::KeyB => Some('b'), KeyCode::KeyC => Some('c'),
        KeyCode::KeyD => Some('d'), KeyCode::KeyE => Some('e'), KeyCode::KeyF => Some('f'),
        KeyCode::KeyG => Some('g'), KeyCode::KeyH => Some('h'), KeyCode::KeyI => Some('i'),
        KeyCode::KeyJ => Some('j'), KeyCode::KeyK => Some('k'), KeyCode::KeyL => Some('l'),
        KeyCode::KeyM => Some('m'), KeyCode::KeyN => Some('n'), KeyCode::KeyO => Some('o'),
        KeyCode::KeyP => Some('p'), KeyCode::KeyQ => Some('q'), KeyCode::KeyR => Some('r'),
        KeyCode::KeyS => Some('s'), KeyCode::KeyT => Some('t'), KeyCode::KeyU => Some('u'),
        KeyCode::KeyV => Some('v'), KeyCode::KeyW => Some('w'), KeyCode::KeyX => Some('x'),
        KeyCode::KeyY => Some('y'), KeyCode::KeyZ => Some('z'),
        KeyCode::Key0 => Some('0'), KeyCode::Key1 => Some('1'), KeyCode::Key2 => Some('2'),
        KeyCode::Key3 => Some('3'), KeyCode::Key4 => Some('4'), KeyCode::Key5 => Some('5'),
        KeyCode::Key6 => Some('6'), KeyCode::Key7 => Some('7'), KeyCode::Key8 => Some('8'),
        KeyCode::Key9 => Some('9'),
        KeyCode::Minus => Some('-'), KeyCode::Equals => Some('='),
        KeyCode::LBracket => Some('['), KeyCode::RBracket => Some(']'),
        KeyCode::Backslash => Some('\\'), KeyCode::Semicolon => Some(';'),
        KeyCode::Quote => Some('\''), KeyCode::Comma => Some(','),
        KeyCode::Period => Some('.'), KeyCode::Slash => Some('/'),
        KeyCode::Backtick => Some('`'), _ => None,
    }
}

#[allow(dead_code)]
fn shift_char(c: char) -> char {
    match c {
        'a'..='z' => c.to_ascii_uppercase(),
        '0'=>')', '1'=>'!', '2'=>'@', '3'=>'#', '4'=>'$',
        '5'=>'%', '6'=>'^', '7'=>'&', '8'=>'*', '9'=>'(',
        '-'=>'_', '='=>'+', '['=>'{', ']'=>'}', '\\'=>'|',
        ';'=>':', '\''=>'"', ','=>'<', '.'=>'>', '/'=>'?', '`'=>'~', c=>c,
    }
}

// `app_main!` generates `pub fn app_main()` on desktop and the JNI `android_main`
// entry (in the cdylib) on Android. The desktop binary (src/main.rs) calls
// `app_main()`; Android's Java shim loads the cdylib and calls `android_main`.
app_main!(App);

// ── Tests ──────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    fn new_grid(cols: usize, rows: usize) -> TermGrid {
        TermGrid::new(cols, rows)
    }

    // ── Grid basics ──
    #[test]
    fn test_grid_new() {
        let g = new_grid(80, 24);
        assert_eq!(g.cols, 80);
        assert_eq!(g.rows, 24);
        assert_eq!(g.cur_r, 0);
        assert_eq!(g.cur_c, 0);
    }

    #[test]
    fn test_grid_put_char() {
        let mut g = new_grid(80, 24);
        g.put('A');
        assert_eq!(g.cells[0][0].ch, 'A');
        assert_eq!(g.cur_c, 1);
        g.put('B');
        assert_eq!(g.cells[0][1].ch, 'B');
        assert_eq!(g.cur_c, 2);
    }

    #[test]
    fn test_grid_newline() {
        let mut g = new_grid(80, 24);
        g.put('A');
        g.newline();
        assert_eq!(g.cur_r, 1);
        // newline does NOT reset cur_c (only CR does)
    }

    #[test]
    fn test_grid_scroll() {
        let mut g = new_grid(80, 3);
        g.cur_r = 2; // bottom row
        g.newline();  // should scroll
        assert_eq!(g.scrollback.len(), 1);
        assert_eq!(g.cur_r, 2);
    }

    #[test]
    fn test_grid_clear_screen() {
        let mut g = new_grid(80, 24);
        g.put('X');
        g.clear_screen();
        assert_eq!(g.cells[0][0].ch, ' ');
        assert_eq!(g.cur_r, 0);
        assert_eq!(g.cur_c, 0);
    }

    #[test]
    fn test_grid_wrap() {
        let mut g = new_grid(5, 3);
        for c in "ABCDE".chars() { g.put(c); }
        assert_eq!(g.cur_c, 5);
        g.put('F'); // wraps
        assert_eq!(g.cur_r, 1);
        assert_eq!(g.cur_c, 1);
        assert_eq!(g.cells[1][0].ch, 'F');
    }

    #[test]
    fn test_grid_resize() {
        let mut g = new_grid(80, 24);
        g.put('A');
        g.resize(40, 12);
        assert_eq!(g.cols, 40);
        assert_eq!(g.rows, 12);
        assert_eq!(g.cells[0][0].ch, 'A');
    }

    // ── VT parser ──
    #[test]
    fn test_process_plain_text() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello");
        assert_eq!(g.cells[0][0].ch, 'H');
        assert_eq!(g.cells[0][4].ch, 'o');
        assert_eq!(g.cur_c, 5);
    }

    #[test]
    fn test_process_newline() {
        let mut g = new_grid(80, 24);
        g.process(b"A\r\nB"); // CR+LF to move to col 0 next line
        assert_eq!(g.cells[0][0].ch, 'A');
        assert_eq!(g.cells[1][0].ch, 'B');
    }

    #[test]
    fn test_process_carriage_return() {
        let mut g = new_grid(80, 24);
        g.process(b"ABC\rX");
        assert_eq!(g.cells[0][0].ch, 'X');
        assert_eq!(g.cells[0][1].ch, 'B');
    }

    #[test]
    fn test_process_backspace() {
        let mut g = new_grid(80, 24);
        g.process(b"AB\x08C");
        assert_eq!(g.cells[0][0].ch, 'A');
        assert_eq!(g.cells[0][1].ch, 'C');
    }

    #[test]
    fn test_process_tab() {
        let mut g = new_grid(80, 24);
        g.process(b"A\tB");
        assert_eq!(g.cells[0][0].ch, 'A');
        assert_eq!(g.cur_c, 9); // tab to 8 + 'B' at 8, cur at 9
        assert_eq!(g.cells[0][8].ch, 'B');
    }

    #[test]
    fn test_csi_cursor_home() {
        let mut g = new_grid(80, 24);
        g.cur_r = 5; g.cur_c = 10;
        g.process(b"\x1b[H"); // cursor home
        assert_eq!(g.cur_r, 0);
        assert_eq!(g.cur_c, 0);
    }

    #[test]
    fn test_csi_cursor_position() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[5;10H"); // row 5, col 10 (1-based)
        assert_eq!(g.cur_r, 4); // 0-based
        assert_eq!(g.cur_c, 9);
    }

    #[test]
    fn test_csi_cursor_up_down() {
        let mut g = new_grid(80, 24);
        g.cur_r = 10;
        g.process(b"\x1b[3A"); // up 3
        assert_eq!(g.cur_r, 7);
        g.process(b"\x1b[5B"); // down 5
        assert_eq!(g.cur_r, 12);
    }

    #[test]
    fn test_csi_cursor_forward_back() {
        let mut g = new_grid(80, 24);
        g.cur_c = 10;
        g.process(b"\x1b[3D"); // back 3
        assert_eq!(g.cur_c, 7);
        g.process(b"\x1b[5C"); // forward 5
        assert_eq!(g.cur_c, 12);
    }

    #[test]
    fn test_csi_erase_display() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello");
        g.process(b"\x1b[2J"); // clear screen
        assert_eq!(g.cells[0][0].ch, ' ');
    }

    #[test]
    fn test_csi_erase_line() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello World");
        g.cur_c = 5;
        g.process(b"\x1b[K"); // erase from cursor to end
        assert_eq!(g.cells[0][4].ch, 'o');
        assert_eq!(g.cells[0][5].ch, ' ');
    }

    // ── SGR colors ──
    #[test]
    fn test_sgr_fg_color() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[31mR"); // red fg
        assert_eq!(g.cells[0][0].fg, 1);
        g.process(b"\x1b[32mG"); // green
        assert_eq!(g.cells[0][1].fg, 2);
    }

    #[test]
    fn test_sgr_bg_color() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[41mR"); // red bg
        assert_eq!(g.cells[0][0].bg, 1);
    }

    #[test]
    fn test_sgr_reset() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[31;42mX\x1b[0mY");
        assert_eq!(g.cells[0][0].fg, 1);
        assert_eq!(g.cells[0][0].bg, 2);
        assert_eq!(g.cells[0][1].fg, 255); // reset
        assert_eq!(g.cells[0][1].bg, 255);
    }

    #[test]
    fn test_sgr_bright_colors() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[91mX"); // bright red
        assert_eq!(g.cells[0][0].fg, 9);
        g.process(b"\x1b[102mY"); // bright green bg
        assert_eq!(g.cells[0][1].bg, 10);
    }

    #[test]
    fn test_sgr_256_color() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[38;5;196mR"); // 256-color fg
        assert_eq!(g.cells[0][0].fg, 196);
    }

    #[test]
    fn test_sgr_bold() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[1mB");
        assert!(g.cells[0][0].bold);
        g.process(b"\x1b[0mN");
        assert!(!g.cells[0][1].bold);
    }

    #[test]
    fn test_sgr_reverse() {
        let mut g = new_grid(80, 24);
        // Default fg=255, bg=255 → reverse: fg=254(dark bg), bg=7(white)
        g.process(b"\x1b[7mX");
        assert_eq!(g.cells[0][0].fg, 254); // default bg as fg
        assert_eq!(g.cells[0][0].bg, 7);   // white as bg (was default fg)
    }

    // ── Alt screen ──
    #[test]
    fn test_alt_screen() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello");
        assert_eq!(g.cells[0][0].ch, 'H');

        g.process(b"\x1b[?1049h"); // enter alt screen
        assert!(g.in_alt_screen);
        assert_eq!(g.cells[0][0].ch, ' '); // alt screen is blank

        g.process(b"Alt");
        assert_eq!(g.cells[0][0].ch, 'A');

        g.process(b"\x1b[?1049l"); // leave alt screen
        assert!(!g.in_alt_screen);
        assert_eq!(g.cells[0][0].ch, 'H'); // main screen restored
    }

    // ── Scroll region ──
    #[test]
    fn test_scroll_region() {
        let mut g = new_grid(80, 10);
        g.process(b"\x1b[3;7r"); // scroll region rows 3-7 (1-based)
        assert_eq!(g.scroll_top, 2);
        assert_eq!(g.scroll_bottom, 6);
    }

    // ── Save/restore cursor ──
    #[test]
    fn test_save_restore_cursor() {
        let mut g = new_grid(80, 24);
        g.cur_r = 5; g.cur_c = 10;
        g.save_cursor();
        g.cur_r = 0; g.cur_c = 0;
        g.restore_cursor();
        assert_eq!(g.cur_r, 5);
        assert_eq!(g.cur_c, 10);
    }

    // ── Search ──
    #[test]
    fn test_search() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello World\nFoo Bar\nHello Again");
        let results = g.search("hello");
        assert_eq!(results.len(), 2);
    }

    // ── Render ──
    #[test]
    fn test_render() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello\nWorld");
        let text = g.render();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    // ── OSC / DCS skip ──
    #[test]
    fn test_osc_skip() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b]0;Title\x07Hello");
        assert_eq!(g.cells[0][0].ch, 'H');
    }

    #[test]
    fn test_dcs_skip() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1bPsomething\x1b\\Hello");
        assert_eq!(g.cells[0][0].ch, 'H');
    }

    // ── UTF-8 ──
    #[test]
    fn test_utf8() {
        let mut g = new_grid(80, 24);
        g.process("日本語".as_bytes());
        assert_eq!(g.cells[0][0].ch, '日');
        assert_eq!(g.cells[0][1].ch, '本');
        assert_eq!(g.cells[0][2].ch, '語');
    }

    // ── Insert/Delete lines ──
    #[test]
    fn test_insert_line() {
        let mut g = new_grid(80, 5);
        g.process(b"A\r\nB\r\nC");
        g.cur_r = 1;
        g.process(b"\x1b[L"); // insert line at row 1
        assert_eq!(g.cells[1][0].ch, ' '); // inserted blank line
    }

    #[test]
    fn test_delete_line() {
        let mut g = new_grid(80, 5);
        g.process(b"A\r\nB\r\nC");
        g.cur_r = 1;
        g.process(b"\x1b[M"); // delete line at row 1
        assert_eq!(g.cells[1][0].ch, 'C'); // C moved up
    }

    // ── Scrollback ──
    #[test]
    fn test_scrollback_limit() {
        let mut g = new_grid(10, 3);
        g.max_scrollback = 5;
        for i in 0..20 {
            g.process(format!("{}\n", i).as_bytes());
        }
        assert!(g.scrollback.len() <= 5);
    }

    // ── Config ──
    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert_eq!(cfg.font_size, 12.0);
        assert_eq!(cfg.scrollback, 5000);
        assert_eq!(cfg.opacity, 0.97);
        assert!(!cfg.bg_color.is_empty());
    }

    // ── Color mapping ──
    #[test]
    fn test_ansi_to_vec4_all() {
        for i in 0..=15 {
            let v = ansi_to_vec4(i);
            assert!(v.x >= 0.0 && v.x <= 1.0);
            assert!(v.y >= 0.0 && v.y <= 1.0);
        }
        let def = ansi_to_vec4(255);
        assert!(def.x > 0.5); // default fg is light
    }

    #[test]
    fn test_rgb_to_ansi_basic() {
        assert_eq!(rgb_to_ansi(0, 0, 0), 0);       // black
        assert!(rgb_to_ansi(255, 0, 0) <= 9);       // red
        assert!(rgb_to_ansi(0, 255, 0) <= 10);      // green
        assert!(rgb_to_ansi(255, 255, 255) >= 7);   // white
    }

    // ── Key mapping ──
    #[test]
    fn test_shift_char() {
        assert_eq!(shift_char('a'), 'A');
        assert_eq!(shift_char('z'), 'Z');
        assert_eq!(shift_char('1'), '!');
        assert_eq!(shift_char(';'), ':');
        assert_eq!(shift_char('-'), '_');
        assert_eq!(shift_char('['), '{');
        assert_eq!(shift_char('\\'), '|');
        assert_eq!(shift_char('`'), '~');
    }

    // ── Background colors ──
    #[test]
    fn test_sgr_bg_256() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[48;5;100mX");
        assert_eq!(g.cells[0][0].bg, 100);
    }

    #[test]
    fn test_sgr_bright_bg() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[103mX"); // bright yellow bg
        assert_eq!(g.cells[0][0].bg, 11);
    }

    // ── Scrollback rendering ──
    #[test]
    fn test_scrollback_content() {
        let mut g = new_grid(10, 3);
        g.process(b"A\r\nB\r\nC\r\nD\r\nE");
        assert!(g.scrollback.len() >= 2);
        assert_eq!(g.scrollback[0][0].ch, 'A');
    }

    // ── Erase characters ──
    #[test]
    fn test_erase_chars() {
        let mut g = new_grid(80, 24);
        g.process(b"ABCDE");
        g.cur_c = 1;
        g.process(b"\x1b[2X"); // erase 2 chars from pos 1
        assert_eq!(g.cells[0][0].ch, 'A');
        assert_eq!(g.cells[0][1].ch, ' ');
        assert_eq!(g.cells[0][2].ch, ' ');
        assert_eq!(g.cells[0][3].ch, 'D');
    }

    // ── Cursor column absolute ──
    #[test]
    fn test_csi_cha() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[15G"); // cursor to column 15
        assert_eq!(g.cur_c, 14); // 0-based
    }

    // ── Cursor vertical absolute ──
    #[test]
    fn test_csi_vpa() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[10d"); // cursor to row 10
        assert_eq!(g.cur_r, 9);
    }

    // ── Newline in scroll region ──
    #[test]
    fn test_newline_in_scroll_region() {
        let mut g = new_grid(80, 10);
        g.scroll_top = 2;
        g.scroll_bottom = 5;
        g.cur_r = 5; // at bottom of scroll region
        g.process(b"X");
        g.newline(); // should scroll within region
        assert_eq!(g.cur_r, 5); // stays at bottom
    }

    // ── Multiple SGR params ──
    #[test]
    fn test_sgr_multiple_params() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[1;31;42mX"); // bold + red fg + green bg
        assert!(g.cells[0][0].bold);
        assert_eq!(g.cells[0][0].fg, 1);
        assert_eq!(g.cells[0][0].bg, 2);
    }

    // ── Cursor next/prev line ──
    #[test]
    fn test_csi_cnl_cpl() {
        let mut g = new_grid(80, 24);
        g.cur_r = 5; g.cur_c = 10;
        g.process(b"\x1b[2E"); // CNL: next 2 lines, col 0
        assert_eq!(g.cur_r, 7);
        assert_eq!(g.cur_c, 0);
        g.process(b"\x1b[3F"); // CPL: prev 3 lines, col 0
        assert_eq!(g.cur_r, 4);
        assert_eq!(g.cur_c, 0);
    }

    // ── Tab stop ──
    #[test]
    fn test_tab_alignment() {
        let mut g = new_grid(80, 24);
        g.process(b"\tX");
        assert_eq!(g.cur_c, 9); // tab to 8, X at 8, cur at 9
    }

    // ── Config toml ──
    // ── Selection ──
    #[test]
    fn test_selection() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello World");
        g.start_select(0, 0);
        g.update_select(0, 4);
        assert!(g.is_selected(0, 2));
        assert!(!g.is_selected(0, 6));
        let text = g.get_selection_text().unwrap();
        assert_eq!(text, "Hello");
    }

    #[test]
    fn test_select_all() {
        let mut g = new_grid(10, 3);
        g.process(b"ABC");
        g.select_all();
        assert!(g.is_selected(0, 0));
        assert!(g.is_selected(2, 9));
    }

    #[test]
    fn test_clear_select() {
        let mut g = new_grid(80, 24);
        g.start_select(0, 0);
        g.update_select(0, 5);
        g.clear_select();
        assert!(!g.is_selected(0, 0));
    }

    // ── URL detection ──
    #[test]
    fn test_find_urls() {
        let mut g = new_grid(80, 24);
        g.process(b"Visit https://github.com/test for info");
        let urls = g.find_urls();
        assert_eq!(urls.len(), 1);
        assert!(urls[0].3.starts_with("https://"));
    }

    #[test]
    fn test_no_urls() {
        let mut g = new_grid(80, 24);
        g.process(b"No urls here");
        assert!(g.find_urls().is_empty());
    }

    // ── OSC title ──
    #[test]
    fn test_osc_title() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b]0;My Terminal Title\x07");
        assert_eq!(g.title, "My Terminal Title");
    }

    #[test]
    fn test_osc_title_2() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b]2;Window Title\x07Hello");
        assert_eq!(g.title, "Window Title");
        assert_eq!(g.cells[0][0].ch, 'H');
    }

    // ── Bell ──
    #[test]
    fn test_bell() {
        let mut g = new_grid(80, 24);
        assert!(!g.bell);
        g.process(b"\x07");
        assert!(g.bell);
    }

    // ── Bold bright ──
    #[test]
    fn test_bold_bright_color() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[1;31mX"); // bold red
        assert!(g.cells[0][0].bold);
        assert_eq!(g.cells[0][0].fg, 1);
        // Bold red should render as bright red (9) — tested in rendering
    }

    // ── Multi-line selection ──
    #[test]
    fn test_multiline_selection() {
        let mut g = new_grid(80, 24);
        g.process(b"Line1\r\nLine2\r\nLine3");
        g.start_select(0, 0);
        g.update_select(2, 4);
        assert!(g.is_selected(0, 0));
        assert!(g.is_selected(1, 3));
        assert!(g.is_selected(2, 4));
        assert!(!g.is_selected(2, 5));
    }

    #[test]
    fn test_config_toml_parse() {
        let toml_str = "shell = \"/bin/bash\"\nfont_size = 14.0\ncols = 100\nrows = 30\nscrollback = 3000\n";
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.shell, "/bin/bash");
        assert_eq!(cfg.font_size, 14.0);
        assert_eq!(cfg.cols, 100);
        assert_eq!(cfg.scrollback, 3000);
    }

    // ── Edge cases ──
    #[test]
    fn test_empty_grid_render() {
        let g = new_grid(80, 24);
        let text = g.render();
        assert!(text.is_empty() || text.chars().all(|c| c == ' ' || c == '\n'));
    }

    #[test]
    fn test_cursor_at_boundary() {
        let mut g = new_grid(5, 3);
        g.cur_c = 4;
        g.put('X');
        assert_eq!(g.cur_c, 5); // at boundary
        g.put('Y'); // wraps
        assert_eq!(g.cur_r, 1);
    }

    #[test]
    fn test_backspace_at_zero() {
        let mut g = new_grid(80, 24);
        g.cur_c = 0;
        g.bs();
        assert_eq!(g.cur_c, 0); // doesn't go negative
    }

    #[test]
    fn test_cursor_up_at_top() {
        let mut g = new_grid(80, 24);
        g.cur_r = 0;
        g.process(b"\x1b[5A"); // up 5 from row 0
        assert_eq!(g.cur_r, 0); // stays at 0
    }

    #[test]
    fn test_cursor_right_at_end() {
        let mut g = new_grid(10, 5);
        g.process(b"\x1b[999C"); // forward 999
        assert_eq!(g.cur_c, 9); // clamped
    }

    #[test]
    fn test_clear_below() {
        let mut g = new_grid(80, 5);
        g.process(b"AAA\r\nBBB\r\nCCC");
        g.cur_r = 1; g.cur_c = 0;
        g.clear_below();
        assert_eq!(g.cells[0][0].ch, 'A'); // above: untouched
        assert_eq!(g.cells[1][0].ch, ' '); // cleared
        assert_eq!(g.cells[2][0].ch, ' '); // cleared
    }

    #[test]
    fn test_tab_at_end_of_line() {
        let mut g = new_grid(10, 3);
        g.cur_c = 9;
        g.tab();
        assert_eq!(g.cur_c, 9); // clamped to cols-1
    }

    #[test]
    fn test_utf8_emoji() {
        let mut g = new_grid(80, 24);
        g.process("🎉".as_bytes());
        assert_eq!(g.cells[0][0].ch, '🎉');
    }

    #[test]
    fn test_multiple_colors_one_line() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[31mR\x1b[32mG\x1b[34mB\x1b[0mN");
        assert_eq!(g.cells[0][0].fg, 1); // red
        assert_eq!(g.cells[0][1].fg, 2); // green
        assert_eq!(g.cells[0][2].fg, 4); // blue
        assert_eq!(g.cells[0][3].fg, 255); // reset
    }

    #[test]
    fn test_reverse_then_reset() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[7mR\x1b[0mN");
        // R should have reversed colors
        assert_ne!(g.cells[0][0].fg, 255);
        // N should be default
        assert_eq!(g.cells[0][1].fg, 255);
    }

    #[test]
    fn test_alt_screen_preserves_scrollback() {
        let mut g = new_grid(10, 3);
        g.process(b"A\r\nB\r\nC\r\nD"); // D causes scroll, A goes to scrollback
        let sb_before = g.scrollback.len();
        g.enter_alt_screen();
        g.leave_alt_screen();
        assert_eq!(g.scrollback.len(), sb_before); // scrollback preserved
    }

    #[test]
    fn test_selection_empty_grid() {
        let g = new_grid(80, 24);
        assert!(g.get_selection_text().is_none());
    }

    #[test]
    fn test_config_defaults_valid() {
        let cfg = Config::default();
        assert!(cfg.cell_width > 0.0);
        assert!(cfg.cell_height > 0.0);
        assert!(cfg.cols > 0);
        assert!(cfg.rows > 0);
        assert!(!cfg.cursor_style.is_empty());
    }

    #[test]
    fn test_find_urls_multiple() {
        let mut g = new_grid(200, 24);
        g.process(b"Go to https://example.com and https://rust-lang.org for info");
        let urls = g.find_urls();
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn test_osc_no_crash_on_invalid() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b]999;garbage\x07OK");
        assert_eq!(g.cells[0][0].ch, 'O');
    }

    #[test]
    fn test_csi_no_crash_on_large_params() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[99999;99999HX");
        assert_eq!(g.cells[g.rows-1][g.cols-1].ch, 'X');
    }

    #[test]
    fn test_resize_preserves_content() {
        let mut g = new_grid(80, 24);
        g.process(b"Hello World");
        g.resize(40, 12);
        assert_eq!(g.cells[0][0].ch, 'H');
        assert_eq!(g.cols, 40);
        assert_eq!(g.rows, 12);
    }

    // ── DA/DSR responses ──
    #[test]
    fn test_da_response() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[c"); // DA query
        assert!(!g.response_buf.is_empty());
        let resp = String::from_utf8_lossy(&g.response_buf);
        assert!(resp.contains("?62"));
    }

    #[test]
    fn test_dsr_cursor_position() {
        let mut g = new_grid(80, 24);
        g.cur_r = 5; g.cur_c = 10;
        g.process(b"\x1b[6n"); // DSR cursor pos
        let resp = String::from_utf8_lossy(&g.response_buf);
        assert!(resp.contains("6;11")); // 1-based
    }

    #[test]
    fn test_dsr_status() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[5n");
        assert_eq!(&g.response_buf, b"\x1b[0n");
    }

    // ── Window ops ──
    #[test]
    fn test_window_size_report() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[18t"); // query text size
        let resp = String::from_utf8_lossy(&g.response_buf);
        assert!(resp.contains("8;24;80"));
    }

    // ── Erase modes ──
    #[test]
    fn test_erase_line_left() {
        let mut g = new_grid(80, 24);
        g.process(b"ABCDE");
        g.cur_c = 2;
        g.process(b"\x1b[1K"); // erase from start to cursor
        assert_eq!(g.cells[0][0].ch, ' ');
        assert_eq!(g.cells[0][2].ch, ' ');
        assert_eq!(g.cells[0][3].ch, 'D');
    }

    #[test]
    fn test_erase_display_above() {
        let mut g = new_grid(80, 5);
        g.process(b"AAA\r\nBBB\r\nCCC");
        g.cur_r = 1; g.cur_c = 1;
        g.process(b"\x1b[1J"); // erase from start to cursor
        assert_eq!(g.cells[0][0].ch, ' '); // row 0 cleared
        assert_eq!(g.cells[1][0].ch, ' '); // row 1 col 0 cleared
        assert_eq!(g.cells[1][1].ch, ' '); // row 1 col 1 cleared
        assert_eq!(g.cells[2][0].ch, 'C'); // row 2 untouched
    }

    // ── App cursor keys ──
    #[test]
    fn test_app_cursor_keys() {
        let mut g = new_grid(80, 24);
        assert!(!g.app_cursor_keys);
        g.process(b"\x1b[?1h"); // enable DECCKM
        assert!(g.app_cursor_keys);
        g.process(b"\x1b[?1l"); // disable
        assert!(!g.app_cursor_keys);
    }

    // ── OSC color queries ──
    #[test]
    fn test_osc_fg_query() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b]10;?\x07");
        let resp = String::from_utf8_lossy(&g.response_buf);
        assert!(resp.contains("10;rgb:"));
    }

    #[test]
    fn test_osc_bg_query() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b]11;?\x07");
        let resp = String::from_utf8_lossy(&g.response_buf);
        assert!(resp.contains("11;rgb:"));
    }

    // ── Truecolor ──
    #[test]
    fn test_truecolor_fg() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[38;2;255;128;0mX");
        assert!(is_truecolor(g.cells[0][0].fg));
        let v = color_to_vec4(g.cells[0][0].fg);
        assert!((v.x - 1.0).abs() < 0.01); // r=255
        assert!((v.y - 0.502).abs() < 0.01); // g=128
    }

    #[test]
    fn test_truecolor_bg() {
        let mut g = new_grid(80, 24);
        g.process(b"\x1b[48;2;0;100;200mX");
        assert!(is_truecolor(g.cells[0][0].bg));
    }

    // ── 256 color cube ──
    #[test]
    fn test_256_color_cube() {
        // Color 196 = red (5,0,0) in 6x6x6 cube
        let v = color_to_vec4(196);
        assert!(v.x > 0.9); // very red
        assert!(v.y < 0.1); // no green
    }

    #[test]
    fn test_256_grayscale() {
        // 232 = darkest grey, 255 would be lightest but 255=default
        let v = color_to_vec4(232);
        assert!(v.x < 0.1); // very dark
        let v2 = color_to_vec4(250);
        assert!(v2.x > 0.5); // lighter
    }

    // ── Theme tests ──
    #[test]
    fn test_parse_hex_color() {
        let v = parse_hex_color("#FF8000");
        assert!((v.x - 1.0).abs() < 0.01);
        assert!((v.y - 0.502).abs() < 0.01);
        assert!((v.z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_hex_color_no_hash() {
        let v = parse_hex_color("1E1E1E");
        assert!((v.x - 0.118).abs() < 0.01);
    }

    #[test]
    fn test_theme_default() {
        let t = Theme::default();
        assert_eq!(t.name, "leuwi-dark");
        assert_eq!(t.ansi.len(), 16);
        assert_eq!(t.bg, "#1E1E1E");
    }

    #[test]
    fn test_theme_ansi_color() {
        let t = Theme::default();
        let v = t.ansi_color(1); // red
        assert!(v.x > 0.8); // very red
    }

    #[test]
    fn test_theme_hex_to_vec4() {
        let t = Theme::default();
        let v = t.hex_to_vec4("#58A6FF");
        assert!(v.z > 0.9); // very blue
    }

    // ── Search tests ──
    #[test]
    fn test_search_case_insensitive() {
        let mut g = new_grid(80, 5);
        g.process(b"Hello World\r\nhello again");
        let results = g.search("hello");
        assert_eq!(results.len(), 2); // both lines match
    }

    #[test]
    fn test_search_multiple_per_line() {
        let mut g = new_grid(80, 5);
        g.process(b"abcabc");
        let results = g.search("abc");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (0, 0));
        assert_eq!(results[1], (0, 3));
    }

    #[test]
    fn test_search_in_scrollback() {
        let mut g = new_grid(80, 3);
        g.process(b"line1\r\nline2\r\nline3\r\nline4\r\nline5");
        let results = g.search("line");
        assert!(results.len() >= 3); // some in scrollback, some visible
    }

    #[test]
    fn test_search_empty_query() {
        let g = new_grid(80, 5);
        let results = g.search("");
        assert!(results.is_empty());
    }

    // ── Split direction ──
    #[test]
    fn test_split_dir_enum() {
        assert_eq!(SplitDir::Vertical, SplitDir::Vertical);
        assert_ne!(SplitDir::Vertical, SplitDir::Horizontal);
    }

    // ── Config with theme ──
    #[test]
    fn test_config_has_theme() {
        let cfg = Config::default();
        assert_eq!(cfg.theme, "leuwi-dark");
    }
}
