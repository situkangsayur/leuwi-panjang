use makepad_widgets::*;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

// ── Terminal Grid ──────────────────────────────────────────
struct TermGrid {
    cols: usize,
    rows: usize,
    cells: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
}

impl Default for TermGrid {
    fn default() -> Self { Self::new(140, 45) }
}

impl TermGrid {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols, rows,
            cells: vec![vec![' '; cols]; rows],
            cursor_row: 0, cursor_col: 0,
        }
    }

    fn put_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.newline();
        }
        if self.cursor_row < self.rows {
            self.cells[self.cursor_row][self.cursor_col] = c;
            self.cursor_col += 1;
        }
    }

    fn newline(&mut self) {
        if self.cursor_row + 1 >= self.rows {
            // Scroll up
            self.cells.remove(0);
            self.cells.push(vec![' '; self.cols]);
        } else {
            self.cursor_row += 1;
        }
    }

    fn carriage_return(&mut self) { self.cursor_col = 0; }
    fn backspace(&mut self) { if self.cursor_col > 0 { self.cursor_col -= 1; } }
    fn tab(&mut self) { self.cursor_col = ((self.cursor_col / 8) + 1) * 8; if self.cursor_col >= self.cols { self.cursor_col = self.cols - 1; } }

    fn clear_line_from_cursor(&mut self) {
        for c in self.cursor_col..self.cols { self.cells[self.cursor_row][c] = ' '; }
    }
    fn clear_screen(&mut self) {
        for row in &mut self.cells { for c in row.iter_mut() { *c = ' '; } }
        self.cursor_row = 0; self.cursor_col = 0;
    }
    fn clear_screen_from_cursor(&mut self) {
        self.clear_line_from_cursor();
        for r in (self.cursor_row + 1)..self.rows {
            for c in 0..self.cols { self.cells[r][c] = ' '; }
        }
    }

    fn cursor_up(&mut self, n: usize) { self.cursor_row = self.cursor_row.saturating_sub(n); }
    fn cursor_down(&mut self, n: usize) { self.cursor_row = (self.cursor_row + n).min(self.rows - 1); }
    fn cursor_forward(&mut self, n: usize) { self.cursor_col = (self.cursor_col + n).min(self.cols - 1); }
    fn cursor_back(&mut self, n: usize) { self.cursor_col = self.cursor_col.saturating_sub(n); }
    fn cursor_to(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
    }
    fn cursor_to_col(&mut self, col: usize) { self.cursor_col = col.min(self.cols - 1); }

    fn render(&self) -> String {
        let mut out = String::with_capacity((self.cols + 1) * self.rows);
        let mut last_content_row = 0;
        for (r, row) in self.cells.iter().enumerate() {
            if row.iter().any(|&c| c != ' ') { last_content_row = r; }
        }
        for r in 0..=last_content_row {
            let mut last_non_space = 0;
            for (c, &ch) in self.cells[r].iter().enumerate() {
                if ch != ' ' { last_non_space = c + 1; }
            }
            for c in 0..last_non_space {
                out.push(self.cells[r][c]);
            }
            if r < last_content_row { out.push('\n'); }
        }
        out
    }

    /// Process raw PTY bytes through a simple VT parser
    fn process(&mut self, data: &[u8]) {
        let mut i = 0;
        while i < data.len() {
            let b = data[i];
            match b {
                0x1b => {
                    i += 1;
                    if i >= data.len() { break; }
                    match data[i] {
                        b'[' => {
                            // CSI sequence
                            i += 1;
                            let mut params = Vec::new();
                            let mut num: i32 = -1;
                            let mut private = false;

                            while i < data.len() {
                                let c = data[i];
                                if c == b'?' { private = true; i += 1; continue; }
                                if c == b'>' || c == b'=' || c == b'!' { i += 1; continue; }
                                if c >= b'0' && c <= b'9' {
                                    if num < 0 { num = 0; }
                                    num = num * 10 + (c - b'0') as i32;
                                    i += 1; continue;
                                }
                                if c == b';' {
                                    params.push(if num < 0 { 0 } else { num as usize });
                                    num = -1; i += 1; continue;
                                }
                                // Final byte
                                if num >= 0 { params.push(num as usize); }
                                let p0 = params.first().copied().unwrap_or(1);
                                let p1 = params.get(1).copied().unwrap_or(1);
                                match c {
                                    b'A' => self.cursor_up(p0.max(1)),
                                    b'B' => self.cursor_down(p0.max(1)),
                                    b'C' => self.cursor_forward(p0.max(1)),
                                    b'D' => self.cursor_back(p0.max(1)),
                                    b'H' | b'f' => self.cursor_to(p0.saturating_sub(1), p1.saturating_sub(1)),
                                    b'G' => self.cursor_to_col(p0.saturating_sub(1)),
                                    b'd' => { self.cursor_row = p0.saturating_sub(1).min(self.rows - 1); }
                                    b'J' => {
                                        match p0 { 0 => self.clear_screen_from_cursor(), 2 | 3 => self.clear_screen(), _ => {} }
                                    }
                                    b'K' => {
                                        match params.first().copied().unwrap_or(0) {
                                            0 => self.clear_line_from_cursor(),
                                            2 => { for c in 0..self.cols { self.cells[self.cursor_row][c] = ' '; } }
                                            _ => {}
                                        }
                                    }
                                    b'E' => { self.cursor_col = 0; self.cursor_down(p0.max(1)); }
                                    b'F' => { self.cursor_col = 0; self.cursor_up(p0.max(1)); }
                                    // m (SGR), h, l, r, etc — silently ignore
                                    _ => {}
                                }
                                i += 1; break;
                            }
                        }
                        b']' => {
                            // OSC — skip until BEL or ST
                            i += 1;
                            while i < data.len() {
                                if data[i] == 0x07 { i += 1; break; }
                                if data[i] == 0x1b && i + 1 < data.len() && data[i+1] == b'\\' { i += 2; break; }
                                i += 1;
                            }
                        }
                        b'P' | b'_' | b'^' => {
                            // DCS, APC, PM — skip until ST
                            i += 1;
                            while i < data.len() {
                                if data[i] == 0x1b && i + 1 < data.len() && data[i+1] == b'\\' { i += 2; break; }
                                if data[i] == 0x07 { i += 1; break; }
                                i += 1;
                            }
                        }
                        b'(' | b')' | b'*' | b'+' => { i += 1; } // charset — skip next byte
                        b'7' | b'8' | b'c' | b'D' | b'E' | b'M' => {} // save/restore/reset — ignore
                        _ => {}
                    }
                }
                b'\n' => { self.newline(); i += 1; }
                b'\r' => { self.carriage_return(); i += 1; }
                b'\x08' => { self.backspace(); i += 1; }
                b'\t' => { self.tab(); i += 1; }
                0x07 => { i += 1; } // BEL
                0x00..=0x06 | 0x0e..=0x1a | 0x1c..=0x1f => { i += 1; } // skip control chars
                _ => {
                    // UTF-8 character
                    let start = i;
                    if b < 0x80 { self.put_char(b as char); i += 1; }
                    else {
                        let len = if b < 0xE0 { 2 } else if b < 0xF0 { 3 } else { 4 };
                        let end = (start + len).min(data.len());
                        if let Ok(s) = std::str::from_utf8(&data[start..end]) {
                            for ch in s.chars() { self.put_char(ch); }
                            i = end;
                        } else { i += 1; }
                    }
                }
            }
        }
    }
}

// ── Makepad App ────────────────────────────────────────────

live_design! {
    use link::theme::*;
    use link::widgets::*;

    App = {{App}} {
        ui: <Window> {
            window: { title: "Leuwi Panjang", inner_size: vec2(1100, 700) }
            show_bg: true
            draw_bg: { color: #x161B22 }

            caption_bar = <SolidView> {
                visible: true
                flow: Right
                height: 32
                draw_bg: { color: #x0D1117 }
                caption_label = <View> { visible: false, width: 0, height: 0 }

                tabs = <View> {
                    width: Fill, height: Fill, flow: Right
                    align: { y: 0.5 }
                    padding: { left: 8 }

                    tab1 = <Button> {
                        text: " Terminal 1 "
                        draw_text: { color: #xC9D1D9, text_style: { font_size: 9.5 } }
                        draw_bg: { color: #x161B22, fn pixel(self) -> vec4 { return self.color; } }
                        padding: { left: 14, right: 14, top: 4, bottom: 4 }
                    }
                    <View> { width: Fill, height: Fill }
                    plus_btn = <Button> {
                        text: "+"
                        draw_text: { color: #x6E7681, text_style: { font_size: 13.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                        width: 28
                    }
                    menu_btn = <Button> {
                        text: "≡"
                        draw_text: { color: #x6E7681, text_style: { font_size: 14.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                        width: 32
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
                show_bg: true
                draw_bg: { color: #x161B22 }
                padding: { top: 8, left: 12, right: 12, bottom: 6 }

                output = <Label> {
                    width: Fill
                    text: ""
                    draw_text: {
                        color: #xE6EDF3
                        text_style: { font_size: 13.0 }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] pty_writer: Option<Box<dyn Write + Send>>,
    #[rust] grid: Arc<Mutex<TermGrid>>,
    #[rust] started: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) { makepad_widgets::live_design(cx); }
}

impl App {
    fn start_pty(&mut self, cx: &mut Cx) {
        if self.started { return; }
        self.started = true;

        let pty_system = portable_pty::native_pty_system();
        let size = portable_pty::PtySize { rows: 45, cols: 140, pixel_width: 0, pixel_height: 0 };
        let pair = pty_system.openpty(size).unwrap();

        let shell = std::env::var("SHELL").unwrap_or("/bin/bash".into());
        let mut cmd = portable_pty::CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        for v in &["HOME","USER","PATH","LANG","DISPLAY","WAYLAND_DISPLAY","XDG_RUNTIME_DIR","DBUS_SESSION_BUS_ADDRESS","SSH_AUTH_SOCK"] {
            if let Ok(val) = std::env::var(v) { cmd.env(v, &val); }
        }
        pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().unwrap();
        self.pty_writer = Some(pair.master.take_writer().unwrap());

        let grid = self.grid.clone();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => { grid.lock().unwrap().process(&buf[..n]); }
                }
            }
        });

        cx.start_interval(0.033);
    }
}

impl MatchEvent for App {
    fn handle_actions(&mut self, _cx: &mut Cx, _actions: &Actions) {}
}

impl AppMain for App {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());

        match event {
            Event::Startup => { self.start_pty(cx); }
            Event::Timer(_) => {
                let text = self.grid.lock().unwrap().render();
                self.ui.label(id!(output)).set_text(cx, &text);
                self.ui.redraw(cx);
            }
            Event::KeyDown(ke) => {
                if let Some(w) = &mut self.pty_writer {
                    let b = key_to_bytes(ke);
                    if !b.is_empty() { let _ = w.write_all(&b); let _ = w.flush(); }
                }
            }
            _ => {}
        }
    }
}

fn key_to_bytes(ke: &KeyEvent) -> Vec<u8> {
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
        _ => {
            if let Some(c) = kc_char(&ke.key_code) {
                let c = if ke.modifiers.shift { shift_char(c) } else { c };
                let mut b = [0u8; 4];
                c.encode_utf8(&mut b);
                return b[..c.len_utf8()].to_vec();
            }
            if ke.key_code == KeyCode::Space { return vec![32]; }
            vec![]
        }
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
        KeyCode::Backtick => Some('`'),
        _ => None,
    }
}

fn shift_char(c: char) -> char {
    match c {
        'a'..='z' => c.to_ascii_uppercase(),
        '0' => ')', '1' => '!', '2' => '@', '3' => '#', '4' => '$',
        '5' => '%', '6' => '^', '7' => '&', '8' => '*', '9' => '(',
        '-' => '_', '=' => '+', '[' => '{', ']' => '}', '\\' => '|',
        ';' => ':', '\'' => '"', ',' => '<', '.' => '>', '/' => '?',
        '`' => '~', c => c,
    }
}

app_main!(App);
fn main() { app_main() }
