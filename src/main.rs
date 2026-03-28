use makepad_widgets::*;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

// ── Cell with color ────────────────────────────────────────
#[derive(Clone, Copy)]
struct Cell {
    ch: char,
    fg: u8,  // ANSI color index 0-15, 255=default
    bold: bool,
}

impl Default for Cell {
    fn default() -> Self { Self { ch: ' ', fg: 255, bold: false } }
}

// ── Terminal Grid ──────────────────────────────────────────
struct TermGrid {
    cols: usize,
    rows: usize,       // visible rows
    cells: Vec<Vec<Cell>>,
    scrollback: Vec<Vec<Cell>>,  // scrolled-off rows
    max_scrollback: usize,
    cur_r: usize,
    cur_c: usize,
    cur_fg: u8,
    cur_bold: bool,
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
            cur_r: 0, cur_c: 0, cur_fg: 255, cur_bold: false,
        }
    }

    fn put(&mut self, ch: char) {
        if self.cur_c >= self.cols { self.cur_c = 0; self.newline(); }
        if self.cur_r < self.rows {
            self.cells[self.cur_r][self.cur_c] = Cell { ch, fg: self.cur_fg, bold: self.cur_bold };
            self.cur_c += 1;
        }
    }

    fn newline(&mut self) {
        if self.cur_r + 1 >= self.rows {
            // Save top row to scrollback
            let top = self.cells.remove(0);
            self.scrollback.push(top);
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }
            self.cells.push(vec![Cell::default(); self.cols]);
        } else {
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
                            while i < data.len() {
                                let c = data[i];
                                if c == b'?' || c == b'>' || c == b'=' || c == b'!' { i += 1; continue; }
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
                                        if params.is_empty() { self.cur_fg = 255; self.cur_bold = false; }
                                        else {
                                            let mut j = 0;
                                            while j < params.len() {
                                                match params[j] {
                                                    0 => { self.cur_fg = 255; self.cur_bold = false; }
                                                    1 => self.cur_bold = true,
                                                    22 => self.cur_bold = false,
                                                    30..=37 => self.cur_fg = (params[j] - 30) as u8,
                                                    39 => self.cur_fg = 255,
                                                    90..=97 => self.cur_fg = (params[j] - 90 + 8) as u8,
                                                    38 => {
                                                        // 256-color: 38;5;N
                                                        if j+2 < params.len() && params[j+1] == 5 {
                                                            self.cur_fg = params[j+2].min(255) as u8;
                                                            j += 2;
                                                        } else if j+4 < params.len() && params[j+1] == 2 {
                                                            // RGB — map to nearest basic color
                                                            let r = params[j+2]; let g = params[j+3]; let b = params[j+4];
                                                            self.cur_fg = rgb_to_ansi(r as u8, g as u8, b as u8);
                                                            j += 4;
                                                        }
                                                    }
                                                    _ => {} // ignore bg, underline, etc for now
                                                }
                                                j += 1;
                                            }
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
}

impl Widget for TermView {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let grid = match &self.grid_ref {
            Some(g) => g.lock().unwrap(),
            None => {
                cx.walk_turtle(walk);
                return DrawStep::done();
            }
        };

        let cw = 9.5_f64;
        let ch = 20.0_f64;
        let pad = 12.0;

        // Total rows = scrollback + visible grid
        let sb_len = grid.scrollback.len();
        let vis_last = grid.cur_r.max(
            grid.cells.iter().enumerate()
                .filter(|(_, row)| row.iter().any(|c| c.ch != ' '))
                .map(|(r, _)| r).max().unwrap_or(0)
        );
        let total_rows = sb_len + vis_last + 2;
        let content_h = (total_rows as f64) * ch + pad * 2.0;
        let content_w = (grid.cols as f64) * cw + pad * 2.0;

        let sized_walk = Walk {
            width: Size::Fixed(content_w),
            height: Size::Fixed(content_h),
            ..walk
        };

        let rect = cx.walk_turtle(sized_walk);
        self.draw_bg.draw_abs(cx, rect);

        let px = rect.pos.x + pad;
        let py = rect.pos.y + pad;
        let mut char_buf = [0u8; 4];

        // Draw scrollback rows
        for (r, row) in grid.scrollback.iter().enumerate() {
            let y = py + (r as f64) * ch;
            for (c, cell) in row.iter().enumerate() {
                if cell.ch == ' ' { continue; }
                let x = px + (c as f64) * cw;
                self.draw_text.color = ansi_to_vec4(cell.fg);
                let s = cell.ch.encode_utf8(&mut char_buf);
                self.draw_text.draw_abs(cx, dvec2(x, y), s);
            }
        }

        // Draw visible grid rows
        for r in 0..=vis_last {
            let y = py + ((sb_len + r) as f64) * ch;
            for c in 0..grid.cols {
                let cell = &grid.cells[r][c];
                if cell.ch == ' ' { continue; }
                let x = px + (c as f64) * cw;
                self.draw_text.color = ansi_to_vec4(cell.fg);
                let s = cell.ch.encode_utf8(&mut char_buf);
                self.draw_text.draw_abs(cx, dvec2(x, y), s);
            }
        }

        // Cursor
        let cx_pos = px + (grid.cur_c as f64) * cw;
        let cy_pos = py + ((sb_len + grid.cur_r) as f64) * ch;
        self.draw_cursor.draw_abs(cx, Rect { pos: dvec2(cx_pos, cy_pos), size: dvec2(2.0, ch) });

        DrawStep::done()
    }
}

impl TermViewRef {
    fn set_grid(&self, grid: Arc<Mutex<TermGrid>>) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.grid_ref = Some(grid);
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
            body = <ScrollXYView> {
                width: Fill, height: Fill
                show_bg: true
                draw_bg: { color: #x1E1E1E }
                scroll_bars: <ScrollBars> {
                    show_scroll_x: false
                    show_scroll_y: true
                }
                terminal = <TermView> {}
            }
        }
    }
}

// ── App ────────────────────────────────────────────────────

#[derive(Live, LiveHook)]
pub struct App {
    #[live] ui: WidgetRef,
    #[rust] pty_writer: Option<Box<dyn Write + Send>>,
    #[rust] grid: Arc<Mutex<TermGrid>>,
    #[rust] started: bool,
}

impl LiveRegister for App {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl App {
    fn start_pty(&mut self, cx: &mut Cx) {
        if self.started { return; }
        self.started = true;

        self.ui.term_view(id!(terminal)).set_grid(self.grid.clone());

        let pty_system = portable_pty::native_pty_system();
        let size = portable_pty::PtySize { rows: 33, cols: 110, pixel_width: 0, pixel_height: 0 };
        let pair = pty_system.openpty(size).unwrap();

        let mut cmd = portable_pty::CommandBuilder::new("/bin/zsh");
        cmd.args(["--no-globalrcs", "--no-rcs"]);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("PROMPT", "%n@%m %~ %# ");
        cmd.env("RPROMPT", "");
        cmd.env("LS_COLORS", "di=1;34:ln=1;36:so=1;35:pi=33:ex=1;32:bd=33;40:cd=33;40:*.tar=1;31:*.gz=1;31:*.zip=1;31:*.jpg=1;35:*.png=1;35:*.rs=33:*.go=36:*.py=33:*.js=33:*.ts=36:*.java=31:*.toml=33:*.json=33:*.md=37:*.sh=32");
        cmd.env("CLICOLOR", "1");
        for v in &["HOME","USER","PATH","LANG","DISPLAY","WAYLAND_DISPLAY","XDG_RUNTIME_DIR","DBUS_SESSION_BUS_ADDRESS","SSH_AUTH_SOCK","EDITOR"] {
            if let Ok(val) = std::env::var(v) { cmd.env(v, &val); }
        }
        pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().unwrap();
        self.pty_writer = Some(pair.master.take_writer().unwrap());

        // Send aliases for colored output
        {
            let w = self.pty_writer.as_mut().unwrap();
            let _ = w.write_all(b"alias ls='ls --color=auto'\nalias ll='ls -lah --color=auto'\nalias grep='grep --color=auto'\nclear\n");
            let _ = w.flush();
        }

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
        KeyCode::ReturnKey => vec![13], KeyCode::Backspace => vec![127],
        KeyCode::Tab => vec![9], KeyCode::Escape => vec![27],
        KeyCode::ArrowUp => vec![27,b'[',b'A'], KeyCode::ArrowDown => vec![27,b'[',b'B'],
        KeyCode::ArrowRight => vec![27,b'[',b'C'], KeyCode::ArrowLeft => vec![27,b'[',b'D'],
        KeyCode::Home => vec![27,b'[',b'H'], KeyCode::End => vec![27,b'[',b'F'],
        KeyCode::PageUp => vec![27,b'[',b'5',b'~'], KeyCode::PageDown => vec![27,b'[',b'6',b'~'],
        KeyCode::Delete => vec![27,b'[',b'3',b'~'],
        _ => {
            if let Some(c) = kc_char(&ke.key_code) {
                let c = if ke.modifiers.shift { shift_char(c) } else { c };
                let mut b = [0u8;4]; c.encode_utf8(&mut b);
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
