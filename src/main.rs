use makepad_widgets::*;
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};

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
                    width: Fill, height: Fill
                    flow: Right
                    align: { y: 0.5 }
                    padding: { left: 8 }

                    tab1 = <Button> {
                        text: " Terminal 1 "
                        draw_text: { color: #xC9D1D9, text_style: { font_size: 9.0 } }
                        draw_bg: { color: #x161B22, fn pixel(self) -> vec4 { return self.color; } }
                        padding: { left: 12, right: 12, top: 4, bottom: 4 }
                    }

                    <View> { width: Fill, height: Fill }

                    plus_btn = <Button> {
                        text: "+"
                        draw_text: { color: #x8B949E, text_style: { font_size: 13.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                        width: 28, padding: { left: 6, right: 6 }
                    }
                    menu_btn = <Button> {
                        text: "≡"
                        draw_text: { color: #x8B949E, text_style: { font_size: 14.0 } }
                        draw_bg: { color: #x00000000, fn pixel(self) -> vec4 { return mix(self.color, #x30363D60, self.hover); } }
                        width: 32, padding: { left: 6, right: 6 }
                    }
                }

                windows_buttons = <View> {
                    visible: true
                    width: Fit, height: Fit
                    align: { y: 0.5 }
                    min = <DesktopButton> { draw_bg: { button_type: WindowsMin } }
                    max = <DesktopButton> { draw_bg: { button_type: WindowsMax } }
                    close = <DesktopButton> { draw_bg: { button_type: WindowsClose } }
                }
            }

            window_menu = <WindowMenu> { main = Main { items: [] } }

            body = <View> {
                width: Fill, height: Fill
                flow: Down
                show_bg: true
                draw_bg: { color: #x161B22 }
                padding: { top: 6, left: 10, right: 10, bottom: 4 }

                output = <Label> {
                    width: Fill
                    text: ""
                    draw_text: {
                        color: #xC9D1D9
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
    #[rust] pty_output: Arc<Mutex<String>>,
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

        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(portable_pty::PtySize {
            rows: 40, cols: 120, pixel_width: 0, pixel_height: 0,
        }).unwrap();

        let shell = std::env::var("SHELL").unwrap_or("/bin/bash".into());
        let mut cmd = portable_pty::CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        for v in &["HOME","USER","PATH","LANG","DISPLAY","WAYLAND_DISPLAY","XDG_RUNTIME_DIR"] {
            if let Ok(val) = std::env::var(v) { cmd.env(v, &val); }
        }
        pair.slave.spawn_command(cmd).unwrap();
        drop(pair.slave);

        let reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();
        self.pty_writer = Some(writer);

        let output = self.pty_output.clone();
        std::thread::spawn(move || {
            let mut reader = reader;
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buf[..n]);
                        // Strip common escape sequences
                        let clean = strip_escapes(&text);
                        let mut out = output.lock().unwrap();
                        out.push_str(&clean);
                        // Keep last 8000 chars
                        if out.len() > 8000 {
                            let start = out.len() - 6000;
                            *out = out[start..].to_string();
                        }
                    }
                }
            }
        });

        cx.start_interval(0.033); // 30fps
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
                let text = self.pty_output.lock().unwrap().clone();
                self.ui.label(id!(output)).set_text(cx, &text);
                self.ui.redraw(cx);
            }
            Event::KeyDown(ke) => {
                if let Some(writer) = &mut self.pty_writer {
                    let bytes = key_to_bytes(ke);
                    if !bytes.is_empty() { let _ = writer.write_all(&bytes); let _ = writer.flush(); }
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
        KeyCode::Delete => vec![27, b'[', b'3', b'~'],
        _ => {
            if let Some(c) = kc_char(&ke.key_code) {
                let c = if ke.modifiers.shift { shift_char(c) } else { c };
                let mut b = [0u8; 4];
                c.encode_utf8(&mut b);
                return b[..c.len_utf8()].to_vec();
            }
            match ke.key_code {
                KeyCode::Space => vec![32],
                _ => vec![],
            }
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
        '`' => '~',
        c => c,
    }
}

/// Strip ANSI escape sequences for clean display
fn strip_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    // Skip until letter
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc.is_ascii_alphabetic() || nc == '~' || nc == '@' { break; }
                    }
                }
                Some(']') => {
                    chars.next();
                    // Skip until BEL or ST
                    while let Some(&nc) = chars.peek() {
                        chars.next();
                        if nc == '\x07' { break; }
                        if nc == '\x1b' { chars.next(); break; }
                    }
                }
                Some('(' | ')' | '*' | '+') => { chars.next(); chars.next(); }
                _ => { chars.next(); }
            }
        } else if c == '\r' {
            // Carriage return — move to beginning of line
            if let Some(last_newline) = out.rfind('\n') {
                out.truncate(last_newline + 1);
            } else {
                out.clear();
            }
        } else if c == '\x08' {
            // Backspace
            out.pop();
        } else if c >= ' ' || c == '\n' || c == '\t' {
            out.push(c);
        }
        // Skip other control chars
    }
    out
}

app_main!(App);

fn main() {
    app_main()
}
