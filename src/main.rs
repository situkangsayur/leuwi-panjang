use makepad_widgets::*;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use arboard::Clipboard;
use serde::{Deserialize, Serialize};

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
}

fn default_shell() -> String { std::env::var("SHELL").unwrap_or("/bin/zsh".into()) }
fn default_font_size() -> f64 { 12.0 }
fn default_cols() -> usize { 125 }
fn default_rows() -> usize { 36 }
fn default_scrollback() -> usize { 5000 }
fn default_bg() -> String { "#1E1E1E".into() }
fn default_fg() -> String { "#C5C8C6".into() }
fn default_prompt() -> String { "%n@%m %~ %# ".into() }
fn default_opacity() -> f64 { 0.97 }

impl Default for Config {
    fn default() -> Self {
        Self {
            shell: default_shell(), font_size: default_font_size(),
            cols: default_cols(), rows: default_rows(),
            scrollback: default_scrollback(),
            bg_color: default_bg(), fg_color: default_fg(),
            prompt: default_prompt(), opacity: default_opacity(),
        }
    }
}

impl Config {
    fn load() -> Self {
        let path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("~/.config"))
            .join("leuwi-panjang/config.toml");
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

// ── Terminal Tab ───────────────────────────────────────────
struct TermTab {
    grid: Arc<Mutex<TermGrid>>,
    writer: Option<Box<dyn Write + Send>>,
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    title: String,
    split: Option<Box<TermTab>>,
    split_focused: bool,
}

impl TermTab {
    fn spawn(id: usize, cfg: &Config) -> Self {
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
                    Ok(n) => { g.lock().unwrap().process(&buf[..n]); }
                }
            }
        });

        Self { grid, writer: Some(writer), master: Some(master), title: format!("Terminal {}", id), split: None, split_focused: false }
    }

    fn write(&mut self, data: &[u8]) {
        if let Some(w) = &mut self.writer {
            let _ = w.write_all(data);
            let _ = w.flush();
        }
    }

    fn get_selected_text(&self) -> String {
        let grid = self.grid.lock().unwrap();
        grid.render()
    }

    fn resize(&mut self, cols: usize, rows: usize) {
        let mut grid = self.grid.lock().unwrap();
        grid.resize(cols, rows);
        grid.scroll_bottom = rows.saturating_sub(1);
        drop(grid);
        // Resize PTY — triggers SIGWINCH so vim/htop adapt
        if let Some(master) = &self.master {
            let _ = master.resize(portable_pty::PtySize {
                rows: rows as u16, cols: cols as u16,
                pixel_width: 0, pixel_height: 0,
            });
        }
    }

    /// Get dynamic title from current prompt line (last line with content)
    fn dynamic_title(&self) -> String {
        let grid = self.grid.lock().unwrap();
        // Find current prompt line — extract CWD from it
        for r in (0..grid.rows).rev() {
            let line: String = grid.cells[r].iter().map(|c| if c.ch == ' ' { ' ' } else { c.ch }).collect();
            let trimmed = line.trim();
            if trimmed.is_empty() { continue; }
            // Look for pattern: user@host /path %
            if let Some(at_pos) = trimmed.find('@') {
                if let Some(space_after) = trimmed[at_pos..].find(' ') {
                    let after = &trimmed[at_pos + space_after + 1..];
                    let path = after.split(' ').next().unwrap_or("~");
                    // Get last component of path
                    let name = path.rsplit('/').next().unwrap_or(path);
                    if name.is_empty() || name == "%" || name == "$" {
                        return path.to_string();
                    }
                    return name.to_string();
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
    fg: u8,   // ANSI color index 0-15, 255=default
    bg: u8,   // ANSI bg color, 255=default (transparent)
    bold: bool,
}

impl Default for Cell {
    fn default() -> Self { Self { ch: ' ', fg: 255, bg: 255, bold: false } }
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
    cur_fg: u8,
    cur_bg: u8,
    cur_bold: bool,
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
            cur_r: 0, cur_c: 0, cur_fg: 255, cur_bg: 255, cur_bold: false,
            alt_cells: None, alt_cur_r: 0, alt_cur_c: 0, in_alt_screen: false,
            saved_cur_r: 0, saved_cur_c: 0,
            scroll_top: 0, scroll_bottom: rows.saturating_sub(1),
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
            self.cells[self.cur_r][self.cur_c] = Cell { ch, fg: self.cur_fg, bg: self.cur_bg, bold: self.cur_bold };
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
                                    b'J' => match p0 { 0 => self.clear_below(), 2|3 => self.clear_screen(), _ => {} },
                                    b'K' => match p0 { 0 => self.clear_line_right(), 2 => self.clear_line_full(), _ => {} },
                                    b'E' => { self.cur_c = 0; self.cur_r = (self.cur_r + p0.max(1)).min(self.rows-1); }
                                    b'F' => { self.cur_c = 0; self.cur_r = self.cur_r.saturating_sub(p0.max(1)); }
                                    b'm' => {
                                        // SGR — process colors
                                        if params.is_empty() { self.cur_fg = 255; self.cur_bg = 255; self.cur_bold = false; }
                                        else {
                                            let mut j = 0;
                                            while j < params.len() {
                                                match params[j] {
                                                    0 => { self.cur_fg = 255; self.cur_bg = 255; self.cur_bold = false; }
                                                    1 => self.cur_bold = true,
                                                    2 | 22 => self.cur_bold = false,
                                                    3 => {} // italic
                                                    4 => {} // underline
                                                    7 => { std::mem::swap(&mut self.cur_fg, &mut self.cur_bg); } // reverse
                                                    27 => { std::mem::swap(&mut self.cur_fg, &mut self.cur_bg); } // reverse off
                                                    30..=37 => self.cur_fg = (params[j] - 30) as u8,
                                                    39 => self.cur_fg = 255,
                                                    40..=47 => self.cur_bg = (params[j] - 40) as u8,
                                                    49 => self.cur_bg = 255,
                                                    90..=97 => self.cur_fg = (params[j] - 90 + 8) as u8,
                                                    100..=107 => self.cur_bg = (params[j] - 100 + 8) as u8,
                                                    38 => {
                                                        if j+2 < params.len() && params[j+1] == 5 {
                                                            self.cur_fg = params[j+2].min(255) as u8;
                                                            j += 2;
                                                        } else if j+4 < params.len() && params[j+1] == 2 {
                                                            let r = params[j+2]; let g = params[j+3]; let b = params[j+4];
                                                            self.cur_fg = rgb_to_ansi(r as u8, g as u8, b as u8);
                                                            j += 4;
                                                        }
                                                    }
                                                    48 => {
                                                        if j+2 < params.len() && params[j+1] == 5 {
                                                            self.cur_bg = params[j+2].min(255) as u8;
                                                            j += 2;
                                                        } else if j+4 < params.len() && params[j+1] == 2 {
                                                            let r = params[j+2]; let g = params[j+3]; let b = params[j+4];
                                                            self.cur_bg = rgb_to_ansi(r as u8, g as u8, b as u8);
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
                                        // DECSET — enable modes
                                        if private {
                                            for &p in &params {
                                                match p {
                                                    1049 => self.enter_alt_screen(), // alt screen
                                                    1 => {} // app cursor keys
                                                    25 => {} // show cursor
                                                    2004 => {} // bracketed paste
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                    b'l' => {
                                        // DECRST — disable modes
                                        if private {
                                            for &p in &params {
                                                match p {
                                                    1049 => self.leave_alt_screen(), // leave alt screen
                                                    1 => {} // normal cursor keys
                                                    25 => {} // hide cursor
                                                    2004 => {} // disable bracketed paste
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
                                    _ => {} // silently ignore other CSI
                                }
                                i += 1; break;
                            }
                        }
                        b']' => { i += 1; while i < data.len() { if data[i] == 0x07 { i += 1; break; } if data[i] == 0x1b { i += 2; break; } i += 1; } }
                        b'P' | b'_' | b'^' => { i += 1; while i < data.len() { if data[i] == 0x1b { i += 2; break; } if data[i] == 0x07 { i += 1; break; } i += 1; } }
                        b'(' | b')' | b'*' | b'+' => { i += 1; if i < data.len() { i += 1; } }
                        b'7' => { self.save_cursor(); }     // DECSC
                        b'8' => { self.restore_cursor(); }  // DECRC
                        b'M' => {                           // RI — reverse index (scroll down)
                            if self.cur_r == 0 {
                                self.cells.insert(0, vec![Cell::default(); self.cols]);
                                self.cells.truncate(self.rows);
                            } else {
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
                0x00..=0x06 | 0x07 | 0x0e..=0x1a | 0x1c..=0x1f => { i += 1; }
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
fn ansi_to_vec4(idx: u8) -> Vec4 {
    match idx {
        0  => vec4(0.20, 0.24, 0.28, 1.0),  // black
        1  => vec4(1.00, 0.33, 0.33, 1.0),  // red
        2  => vec4(0.25, 0.73, 0.31, 1.0),  // green
        3  => vec4(0.83, 0.69, 0.22, 1.0),  // yellow
        4  => vec4(0.35, 0.61, 0.98, 1.0),  // blue
        5  => vec4(0.74, 0.50, 0.98, 1.0),  // magenta
        6  => vec4(0.32, 0.83, 0.89, 1.0),  // cyan
        7  => vec4(0.79, 0.82, 0.89, 1.0),  // white
        8  => vec4(0.41, 0.46, 0.52, 1.0),  // bright black
        9  => vec4(1.00, 0.47, 0.47, 1.0),  // bright red
        10 => vec4(0.35, 0.83, 0.42, 1.0),  // bright green
        11 => vec4(0.93, 0.83, 0.32, 1.0),  // bright yellow
        12 => vec4(0.50, 0.74, 1.00, 1.0),  // bright blue
        13 => vec4(0.84, 0.64, 1.00, 1.0),  // bright magenta
        14 => vec4(0.44, 0.91, 0.97, 1.0),  // bright cyan
        15 => vec4(0.91, 0.93, 0.98, 1.0),  // bright white
        _  => vec4(0.90, 0.93, 0.96, 1.0),  // default fg
    }
}

// ── TermView: colored terminal grid widget ─────────────────

#[derive(Live, LiveHook, Widget)]
pub struct TermView {
    #[redraw] #[live] draw_text: DrawText,
    #[live] draw_bg: DrawColor,
    #[live] draw_cursor: DrawColor,
    #[walk] walk: Walk,
    #[layout] layout: Layout,
    #[rust] grid_ref: Option<Arc<Mutex<TermGrid>>>,
    #[rust] scroll_offset: i64,
    #[rust] blink_on: bool,
    #[rust] blink_counter: u32,
}

impl Widget for TermView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        // Mouse wheel = scroll
        if let Event::Scroll(se) = event {
            let lines = (-se.scroll.y / 20.0) as i64;  // natural scroll (inverted)
            self.scroll_offset = (self.scroll_offset + lines).max(0);
            self.redraw(cx);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);

        let grid = match &self.grid_ref {
            Some(g) => g.lock().unwrap(),
            None => return DrawStep::done(),
        };

        let cw = 8.5_f64;
        let ch = 18.0_f64;
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

                // Draw bg color if not default
                if cell.bg != 255 {
                    self.draw_cursor.color = ansi_to_vec4(cell.bg);
                    self.draw_cursor.draw_abs(cx, Rect { pos: dvec2(x, y), size: dvec2(cw, ch) });
                }

                if cell.ch == ' ' { continue; }
                self.draw_text.color = ansi_to_vec4(cell.fg);
                let s = cell.ch.encode_utf8(&mut char_buf);
                self.draw_text.draw_abs(cx, dvec2(x, y), s);
            }
            screen_row += 1;
        }

        // Blinking cursor
        self.blink_counter += 1;
        if self.blink_counter % 15 == 0 { self.blink_on = !self.blink_on; }

        if self.scroll_offset == 0 && self.blink_on {
            let cursor_y = py + (screen_row.saturating_sub(1usize) as f64) * ch;
            let cursor_x = px + (grid.cur_c as f64) * cw;
            // Block cursor with highlight color
            self.draw_cursor.color = vec4(0.345, 0.608, 0.976, 0.8);
            self.draw_cursor.draw_abs(cx, Rect { pos: dvec2(cursor_x, cursor_y), size: dvec2(cw, ch) });
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
}

// ── Makepad App ────────────────────────────────────────────

live_design! {
    use link::theme::*;
    use link::widgets::*;

    TermView = {{TermView}} {
        width: Fill, height: Fill
        draw_bg: { color: #x1E1E1E, fn pixel(self) -> vec4 { return self.color; } }
        draw_cursor: { color: #x58A6FF, fn pixel(self) -> vec4 { return self.color; } }
        draw_text: {
            color: #xC5C8C6
            text_style: {
                font_size: 12.0
                line_spacing: 1.3
                font_family: {
                    latin = font("crate://makepad-widgets/resources/LiberationMono-Regular.ttf", 0.0, 0.0)
                }
            }
        }
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
                    width: Fill, height: Fill, flow: Right, align: { y: 0.5 }, padding: { left: 8 }
                    tab1 = <Button> {
                        text: " Terminal 1 "
                        draw_text: { color: #xC5C8C6, text_style: { font_size: 9.5 } }
                        draw_bg: { color: #x1E1E1E, fn pixel(self) -> vec4 { return self.color; } }
                        padding: { left: 14, right: 14, top: 4, bottom: 4 }
                    }
                    <View> { width: Fill, height: Fill }
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
                    terminal = <TermView> {}
                    split_bar = <View> {
                        visible: false
                        width: 2, height: Fill
                        show_bg: true
                        draw_bg: { color: #x333333 }
                    }
                    terminal2 = <TermView> {
                        width: 0, height: 0
                    }
                }

                // Status bar
                status = <View> {
                    width: Fill, height: 20, flow: Right
                    show_bg: true
                    draw_bg: { color: #x181818 }
                    align: { y: 0.5 }
                    padding: { left: 10, right: 10 }
                    status_text = <Label> {
                        text: ""
                        draw_text: { color: #x6E7681, text_style: { font_size: 8.5 } }
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
    #[rust] tab_counter: usize,
    // Split is now per-tab (stored in TermTab.split)
    #[rust] split_active: bool,
    #[rust] config: Config,
    #[rust] key_handled: bool,  // prevent double-input from KeyDown+TextInput
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
        self.tab_counter = 1;
        self.tabs.push(TermTab::spawn(1, &self.config));
        self.active_tab = 0;
        self.ui.term_view(id!(terminal)).set_grid(self.tabs[0].grid.clone());
        self.update_tab_label(cx);
        cx.start_interval(0.033);
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
        let labels: Vec<String> = self.tabs.iter().enumerate().map(|(i, t)| {
            let name = t.dynamic_title();
            let split_indicator = if t.split.is_some() { "⫽" } else { "" };
            if i == self.active_tab {
                format!(" ▌{} {} ×", name, split_indicator)
            } else {
                format!("  {} {}  ", name, split_indicator)
            }
        }).collect();
        let text = labels.join("│");
        self.ui.button(id!(tab1)).set_text(cx, &text);
    }

    fn split_vertical(&mut self, cx: &mut Cx) {
        let tab = &self.tabs[self.active_tab];
        if tab.split.is_some() { return; }
        self.tab_counter += 1;
        let split = TermTab::spawn(self.tab_counter, &self.config);
        self.ui.term_view(id!(terminal2)).set_grid(split.grid.clone());
        self.tabs[self.active_tab].split = Some(Box::new(split));
        self.split_active = true;
        self.ui.view(id!(split_bar)).set_visible(cx, true);
        self.ui.term_view(id!(terminal2)).set_visible(cx, true);
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
            let text = tab.get_selected_text();
            if !text.is_empty() {
                if let Ok(mut cb) = Clipboard::new() { let _ = cb.set_text(&text); }
            }
        }
    }

    fn paste_from_clipboard(&mut self) {
        let text = match Clipboard::new() {
            Ok(mut cb) => cb.get_text().unwrap_or_default(),
            Err(_) => return,
        };
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

    /// Write bytes to the currently focused pane (main or split)
    fn write_to_active(&mut self, data: &[u8]) {
        if self.split_active {
            if let Some(split) = &mut self.tabs[self.active_tab].split {
                split.write(data);
            }
        } else if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            tab.write(data);
        }
    }

    fn handle_resize(&mut self, width: f64, height: f64) {
        let chrome_h = 32.0 + 20.0; // caption + status bar
        let cw = 8.5;
        let ch = 18.0;
        let cols = ((width - 24.0) / cw).max(20.0) as usize;
        let rows = ((height - chrome_h) / ch).max(5.0) as usize;
        for tab in &mut self.tabs {
            let has_split = tab.split.is_some();
            let tab_cols = if has_split { cols / 2 } else { cols };
            tab.resize(tab_cols, rows);
            if let Some(split) = &mut tab.split {
                split.resize(cols / 2, rows);
            }
        }
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(plus_btn)).clicked(actions) {
            self.new_tab(cx);
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
                self.update_tab_label(cx);
                // Status bar info
                let tab_info = format!(
                    "Tab {}/{}  {}  Cols:{}  Rows:{}",
                    self.active_tab + 1, self.tabs.len(),
                    if self.tabs[self.active_tab].split.is_some() { "Split" } else { "" },
                    self.config.cols, self.config.rows,
                );
                self.ui.label(id!(status_text)).set_text(cx, &tab_info);
                self.ui.redraw(cx);
            }
            Event::KeyDown(ke) => {
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
                        KeyCode::KeyD => { self.split_vertical(cx); return; }
                        KeyCode::KeyE => { self.split_vertical(cx); return; } // horizontal split (same impl for now)
                        KeyCode::KeyF => {
                            // TODO: open search UI overlay
                            // For now, search is via shell (Ctrl+R in zsh)
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
                // Alt+Arrow = switch panes
                if ke.modifiers.alt {
                    match ke.key_code {
                        KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                            self.toggle_split_focus();
                            return;
                        }
                        _ => {}
                    }
                }
                // Forward ONLY special/control keys via KeyDown
                // Printable chars come via TextInput (no double-send)
                self.key_handled = false;
                let b = key_to_special_bytes(ke);
                if !b.is_empty() {
                    self.key_handled = true;
                    self.write_to_active(&b);
                    self.ui.term_view(id!(terminal)).reset_scroll();
                    if self.split_active {
                        self.ui.term_view(id!(terminal2)).reset_scroll();
                    }
                }
            }
            Event::TextInput(te) => {
                // ALL printable input comes here (handles shift, layout, etc.)
                // Skip if KeyDown already handled this event (special keys)
                if self.key_handled {
                    self.key_handled = false;
                    // don't send — already sent via KeyDown
                } else if !te.input.is_empty() && !te.was_paste {
                    self.write_to_active(te.input.as_bytes());
                    self.ui.term_view(id!(terminal)).reset_scroll();
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
fn key_to_special_bytes(ke: &KeyEvent) -> Vec<u8> {
    // Ctrl+key combos
    if ke.modifiers.control {
        if let Some(c) = kc_char(&ke.key_code) {
            let c = c.to_ascii_lowercase();
            if ('a'..='z').contains(&c) { return vec![(c as u8) - b'a' + 1]; }
        }
    }
    match ke.key_code {
        KeyCode::ReturnKey => vec![13],
        KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![9],
        KeyCode::Escape => vec![27],
        KeyCode::ArrowUp => vec![27, b'[', b'A'],
        KeyCode::ArrowDown => vec![27, b'[', b'B'],
        KeyCode::ArrowRight => vec![27, b'[', b'C'],
        KeyCode::ArrowLeft => vec![27, b'[', b'D'],
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

fn shift_char(c: char) -> char {
    match c {
        'a'..='z' => c.to_ascii_uppercase(),
        '0'=>')', '1'=>'!', '2'=>'@', '3'=>'#', '4'=>'$',
        '5'=>'%', '6'=>'^', '7'=>'&', '8'=>'*', '9'=>'(',
        '-'=>'_', '='=>'+', '['=>'{', ']'=>'}', '\\'=>'|',
        ';'=>':', '\''=>'"', ','=>'<', '.'=>'>', '/'=>'?', '`'=>'~', c=>c,
    }
}

app_main!(App);
fn main() { app_main() }

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
        g.cur_fg = 1; g.cur_bg = 2;
        g.process(b"\x1b[7mX"); // reverse
        // After reverse, fg and bg should be swapped
        assert_eq!(g.cells[0][0].fg, 2);
        assert_eq!(g.cells[0][0].bg, 1);
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
}
