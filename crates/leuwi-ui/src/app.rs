use makepad_widgets::*;
use leuwi_terminal::TerminalSession;
use arboard::Clipboard;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    // Leuwi Panjang Dark Green Theme
    LEUWI_BG      = #x0A1410FF   // deep dark green-black
    LEUWI_FG      = #xB8D4CCFF   // light green-tinted gray text
    LEUWI_GREEN   = #x00FF88FF   // bright neon green accent
    LEUWI_DIM_GRN = #x1B6B3FFF   // dim green
    LEUWI_TAB_BG  = #x060F0BFF   // tab bar background (darkest green-black)
    LEUWI_TAB_ACT = #x0A1410FF   // active tab (same as bg)
    LEUWI_BORDER  = #x1A3A28FF   // subtle green border
    LEUWI_DIM     = #x5C8A72FF   // dim green-gray text
    LEUWI_HOVER   = #x0D1F17FF   // hover state (slightly lighter green-dark)

    // Tab button with visible text
    LeuwiTab = <Button> {
        width: Fit,
        height: Fill,
        padding: { left: 16, right: 16, top: 0, bottom: 0 },
        margin: { right: 1 },
        draw_bg: {
            color: (LEUWI_TAB_ACT)
            instance hover: 0.0
            fn pixel(self) -> vec4 {
                return self.color;
            }
        }
        draw_text: {
            color: (LEUWI_FG)
            text_style: { font_size: 10.0 }
        }
    }

    // Small text button
    LeuwiSmallBtn = <Button> {
        width: 36,
        height: Fill,
        padding: { left: 6, right: 6 },
        draw_bg: {
            color: #x00000000
            fn pixel(self) -> vec4 {
                return mix(self.color, #x30363D40, self.hover);
            }
        }
        draw_text: {
            color: (LEUWI_DIM)
            text_style: { font_size: 12.0 }
        }
    }

    LeuwiApp = {{LeuwiApp}} {
        ui: <Window> {
            show_bg: true,
            window: {
                title: "Leuwi Panjang",
                inner_size: vec2(1200, 800),
                position: vec2(100, 100),
            },
            draw_bg: { color: (LEUWI_BG) }

            // Hide the default Makepad menu
            window_menu = <WindowMenu> {
                main = Main { items: [] }
            }

            // Caption bar = our tab bar (Chrome-style)
            // Override default to remove "Makepad" label
            caption_bar = <SolidView> {
                visible: true,
                flow: Right,
                height: 34,
                draw_bg: { color: (LEUWI_TAB_BG) }

                // Override: hide default "Makepad" label completely
                caption_label = <View> {
                    visible: false,
                    width: 0, height: 0,
                }

                // === Tabs (left) ===
                tabs_area = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Right,
                    align: { y: 1.0 },
                    padding: { left: 8, top: 4 },

                    tab1 = <LeuwiTab> {
                        text: "  Terminal 1  "
                    }

                    new_tab_btn = <LeuwiSmallBtn> {
                        width: 30,
                        text: "+"
                        draw_text: {
                            color: (LEUWI_DIM)
                            text_style: { font_size: 16.0 }
                        }
                    }
                }

                // === Right side: menu + window controls ===
                right_controls = <View> {
                    width: Fit,
                    height: Fill,
                    flow: Right,
                    align: { y: 0.5 },

                    menu_btn = <LeuwiSmallBtn> {
                        text: "≡"
                        draw_text: {
                            text_style: { font_size: 16.0 }
                        }
                    }

                    // Window control buttons
                    windows_buttons = <View> {
                        visible: true,
                        width: Fit, height: Fit,
                        align: { y: 0.5 },
                        min = <DesktopButton> {draw_bg: {button_type: WindowsMin}}
                        max = <DesktopButton> {draw_bg: {button_type: WindowsMax}}
                        close = <DesktopButton> {draw_bg: {button_type: WindowsClose}}
                    }
                }
            }

            body = <View> {
                width: Fill, height: Fill,
                flow: Down,
                spacing: 0,

                // Terminal pane container (will hold split panes)
                pane_container = <View> {
                    width: Fill,
                    height: Fill,
                    flow: Right,
                    spacing: 1,

                    // Primary terminal pane
                    pane1 = <View> {
                        width: Fill,
                        height: Fill,
                        flow: Down,
                        show_bg: true,
                        draw_bg: { color: (LEUWI_BG) }

                        // Terminal output with proper padding
                        terminal_scroll = <View> {
                            width: Fill,
                            height: Fill,
                            padding: { top: 8, bottom: 8, left: 12, right: 12 },

                            terminal_output = <Label> {
                                width: Fill,
                                text: "Starting Leuwi Panjang Terminal..."
                                draw_text: {
                                    color: (LEUWI_FG),
                                    text_style: {
                                        font_size: 13.0,
                                        line_spacing: 1.4,
                                    }
                                }
                            }
                        }
                    }

                    // Split divider (hidden by default, shown when split)
                    split_divider_v = <View> {
                        visible: false,
                        width: 2,
                        height: Fill,
                        show_bg: true,
                        draw_bg: { color: (LEUWI_BORDER) }
                    }

                    // Second pane (hidden, shown on vertical split)
                    pane2 = <View> {
                        visible: false,
                        width: Fill,
                        height: Fill,
                        flow: Down,
                        show_bg: true,
                        draw_bg: { color: (LEUWI_BG) }

                        terminal_scroll2 = <View> {
                            width: Fill,
                            height: Fill,
                            padding: { top: 8, bottom: 8, left: 12, right: 12 },

                            terminal_output2 = <Label> {
                                width: Fill,
                                text: ""
                                draw_text: {
                                    color: (LEUWI_FG),
                                    text_style: {
                                        font_size: 13.0,
                                        line_spacing: 1.4,
                                    }
                                }
                            }
                        }
                    }
                }

                // Status bar
                status_bar = <View> {
                    width: Fill,
                    height: 22,
                    flow: Right,
                    align: { y: 0.5 },
                    padding: { left: 12, right: 12 },
                    show_bg: true,
                    draw_bg: { color: (LEUWI_TAB_BG) }

                    // Green dot = connected
                    status_dot = <Label> {
                        text: "●"
                        draw_text: {
                            color: (LEUWI_GREEN),
                            text_style: { font_size: 8.0 }
                        }
                    }

                    status_text = <Label> {
                        margin: { left: 6 },
                        text: "leuwi-panjang v0.1.0"
                        draw_text: {
                            color: (LEUWI_DIM),
                            text_style: { font_size: 9.0 }
                        }
                    }

                    <View> { width: Fill, height: Fill }

                    clock_text = <Label> {
                        text: ""
                        draw_text: {
                            color: (LEUWI_DIM),
                            text_style: { font_size: 9.0 }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
pub struct LeuwiApp {
    #[live]
    ui: WidgetRef,
    // Pane 1 (always active)
    #[rust]
    session: Option<TerminalSession>,
    #[rust]
    pty: Option<leuwi_pty::Pty>,
    // Pane 2 (split)
    #[rust]
    session2: Option<TerminalSession>,
    #[rust]
    pty2: Option<leuwi_pty::Pty>,
    #[rust]
    split_active: bool,
    #[rust]
    active_pane: u8, // 0 = pane1, 1 = pane2
    #[rust]
    initialized: bool,
}

impl LiveRegister for LeuwiApp {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl LeuwiApp {
    fn initialize_terminal(&mut self, cx: &mut Cx) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        // Remove OS window decorations (title bar) on Linux
        // This runs xprop to set Motif hints — removes GNOME/KDE title bar
        #[cfg(target_os = "linux")]
        {
            std::thread::spawn(|| {
                // Small delay to let window appear
                std::thread::sleep(std::time::Duration::from_millis(200));
                let _ = std::process::Command::new("xprop")
                    .args([
                        "-name", "Leuwi Panjang",
                        "-f", "_MOTIF_WM_HINTS", "32c",
                        "-set", "_MOTIF_WM_HINTS", "0x2, 0x0, 0x0, 0x0, 0x0",
                    ])
                    .output();
            });
        }

        let config = leuwi_config::load_config();
        let cols: u16 = 120;
        let rows: u16 = 40;

        let mut session = TerminalSession::new(
            cols as usize,
            rows as usize,
            config.scrollback.lines as usize,
        );

        match leuwi_pty::Pty::spawn(&config.general.default_shell, cols, rows) {
            Ok(pty) => {
                session.start_reader(pty.clone_reader());
                self.pty = Some(pty);
                cx.start_interval(0.016);
            }
            Err(e) => {
                eprintln!("Failed to spawn PTY: {e}");
                session.process_bytes(
                    format!("Error: Failed to spawn shell: {e}\r\n").as_bytes(),
                );
            }
        }

        self.session = Some(session);
    }

    fn split_vertical(&mut self, cx: &mut Cx) {
        if self.split_active {
            return; // Already split
        }

        let config = leuwi_config::load_config();
        let cols: u16 = 60;
        let rows: u16 = 40;

        let mut session2 = TerminalSession::new(
            cols as usize,
            rows as usize,
            config.scrollback.lines as usize,
        );

        match leuwi_pty::Pty::spawn(&config.general.default_shell, cols, rows) {
            Ok(pty) => {
                session2.start_reader(pty.clone_reader());
                self.pty2 = Some(pty);
            }
            Err(e) => {
                eprintln!("Failed to spawn PTY for pane 2: {e}");
                return;
            }
        }

        self.session2 = Some(session2);
        self.split_active = true;
        self.active_pane = 1; // Focus new pane

        // Show pane2 and divider
        self.ui.view(id!(pane2)).set_visible(cx, true);
        self.ui.view(id!(split_divider_v)).set_visible(cx, true);
        self.ui.redraw(cx);
    }

    fn close_split(&mut self, cx: &mut Cx) {
        if !self.split_active {
            return;
        }
        if let Some(session2) = &self.session2 {
            session2.stop();
        }
        self.session2 = None;
        self.pty2 = None;
        self.split_active = false;
        self.active_pane = 0;

        self.ui.view(id!(pane2)).set_visible(cx, false);
        self.ui.view(id!(split_divider_v)).set_visible(cx, false);
        self.ui.redraw(cx);
    }

    fn toggle_active_pane(&mut self) {
        if self.split_active {
            self.active_pane = if self.active_pane == 0 { 1 } else { 0 };
        }
    }

    fn copy_selection(&self) {
        let session = if self.active_pane == 1 && self.split_active {
            self.session2.as_ref()
        } else {
            self.session.as_ref()
        };

        if let Some(session) = session {
            let grid = session.grid.lock().unwrap();
            let text = grid.get_selected_text()
                .unwrap_or_else(|| grid.get_all_text());

            if !text.is_empty() {
                if let Ok(mut clipboard) = Clipboard::new() {
                    let _ = clipboard.set_text(&text);
                }
            }
        }
    }

    fn paste_clipboard(&mut self) {
        let text = match Clipboard::new() {
            Ok(mut clipboard) => clipboard.get_text().unwrap_or_default(),
            Err(_) => return,
        };

        if text.is_empty() {
            return;
        }

        // Send paste with bracketed paste mode markers
        let bracketed_start = b"\x1b[200~";
        let bracketed_end = b"\x1b[201~";

        let pty = if self.active_pane == 1 && self.split_active {
            self.pty2.as_mut()
        } else {
            self.pty.as_mut()
        };

        if let Some(pty) = pty {
            let _ = pty.write(bracketed_start);
            let _ = pty.write(text.as_bytes());
            let _ = pty.write(bracketed_end);
        }
    }

    fn select_all_text(&self) {
        let session = if self.active_pane == 1 && self.split_active {
            self.session2.as_ref()
        } else {
            self.session.as_ref()
        };

        if let Some(session) = session {
            let mut grid = session.grid.lock().unwrap();
            grid.select_all();
        }
    }

    fn start_mouse_selection(&self, me: &MouseDownEvent) {
        // Convert mouse position to grid coordinates
        // Approximate: padding left=12, top=8, cell ~8x18 pixels
        let col = ((me.abs.x - 12.0) / 8.0).max(0.0) as usize;
        let row = ((me.abs.y - 42.0) / 18.0).max(0.0) as usize; // 42 = tab bar(34) + padding(8)

        let session = if self.active_pane == 1 && self.split_active {
            self.session2.as_ref()
        } else {
            self.session.as_ref()
        };

        if let Some(session) = session {
            let mut grid = session.grid.lock().unwrap();
            grid.start_selection(row, col);
        }
    }

    fn update_mouse_selection(&self, me: &MouseMoveEvent) {
        let col = ((me.abs.x - 12.0) / 8.0).max(0.0) as usize;
        let row = ((me.abs.y - 42.0) / 18.0).max(0.0) as usize;

        let session = if self.active_pane == 1 && self.split_active {
            self.session2.as_ref()
        } else {
            self.session.as_ref()
        };

        if let Some(session) = session {
            let mut grid = session.grid.lock().unwrap();
            grid.update_selection(row, col);
        }
    }

    fn render_grid_to_text(&self) -> String {
        let session = match &self.session {
            Some(s) => s,
            None => return "No session".to_string(),
        };

        let grid = session.grid.lock().unwrap();
        let mut output = String::with_capacity(grid.cols() * grid.rows() * 2);

        for row in 0..grid.rows() {
            for col in 0..grid.cols() {
                let cell = grid.cell(row, col);
                if cell.c == '\0' {
                    output.push(' ');
                } else {
                    output.push(cell.c);
                }
            }
            let trimmed = output.trim_end_matches(' ');
            output.truncate(trimmed.len());
            if row < grid.rows() - 1 {
                output.push('\n');
            }
        }

        output
    }

    fn render_grid_to_text_session(session: &TerminalSession) -> String {
        let grid = session.grid.lock().unwrap();
        let mut output = String::with_capacity(grid.cols() * grid.rows() * 2);

        for row in 0..grid.rows() {
            for col in 0..grid.cols() {
                let cell = grid.cell(row, col);
                if cell.c == '\0' {
                    output.push(' ');
                } else {
                    output.push(cell.c);
                }
            }
            let trimmed = output.trim_end_matches(' ');
            output.truncate(trimmed.len());
            if row < grid.rows() - 1 {
                output.push('\n');
            }
        }

        output
    }
}

impl MatchEvent for LeuwiApp {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        if self.ui.button(id!(new_tab_btn)).clicked(actions) {
            // TODO: new tab
        }
        if self.ui.button(id!(menu_btn)).clicked(actions) {
            // TODO: hamburger menu popup
        }
    }
}

impl AppMain for LeuwiApp {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());

        match event {
            Event::Startup => {
                self.initialize_terminal(cx);
            }
            Event::Timer(_te) => {
                // Update pane 1
                let text = self.render_grid_to_text();
                self.ui.label(id!(terminal_output)).set_text(cx, &text);

                // Update pane 2 if split
                if self.split_active {
                    if let Some(session2) = &self.session2 {
                        let text2 = Self::render_grid_to_text_session(session2);
                        self.ui.label(id!(terminal_output2)).set_text(cx, &text2);
                    }
                }

                self.ui.redraw(cx);
            }
            Event::KeyDown(ke) => {
                // Ctrl+Shift shortcuts (terminal special keys)
                if ke.modifiers.control && ke.modifiers.shift {
                    match ke.key_code {
                        // Ctrl+Shift+C = copy selected text
                        KeyCode::KeyC => {
                            self.copy_selection();
                            return;
                        }
                        // Ctrl+Shift+V = paste from clipboard
                        KeyCode::KeyV => {
                            self.paste_clipboard();
                            return;
                        }
                        // Ctrl+Shift+D = split vertical (like iTerm2)
                        KeyCode::KeyD => {
                            self.split_vertical(cx);
                            return;
                        }
                        // Ctrl+Shift+E = split horizontal
                        KeyCode::KeyE => {
                            // TODO: horizontal split
                            return;
                        }
                        // Ctrl+Shift+W = close active pane
                        KeyCode::KeyW => {
                            if self.split_active && self.active_pane == 1 {
                                self.close_split(cx);
                            }
                            return;
                        }
                        // Ctrl+Shift+T = new tab
                        KeyCode::KeyT => {
                            // TODO: new tab
                            return;
                        }
                        // Ctrl+Shift+A = select all
                        KeyCode::KeyA => {
                            self.select_all_text();
                            return;
                        }
                        _ => {}
                    }
                }

                // Alt+Arrow = switch between panes
                if ke.modifiers.alt {
                    match ke.key_code {
                        KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                            self.toggle_active_pane();
                            return;
                        }
                        _ => {}
                    }
                }

                // Forward keyboard input to active PTY
                let bytes = key_event_to_bytes(ke);
                if !bytes.is_empty() {
                    if self.active_pane == 1 && self.split_active {
                        if let Some(pty2) = &mut self.pty2 {
                            let _ = pty2.write(&bytes);
                        }
                    } else if let Some(pty) = &mut self.pty {
                        let _ = pty.write(&bytes);
                    }
                }
            }
            // Mouse selection for text block
            Event::MouseDown(me) => {
                // Start selection on mouse down in terminal area
                self.start_mouse_selection(me);
            }
            Event::MouseMove(me) => {
                // Update selection while dragging (left button held)
                self.update_mouse_selection(me);
            }
            Event::MouseUp(_me) => {
                // Selection complete — text is now selectable via Ctrl+Shift+C
            }
            _ => {}
        }
    }
}

fn key_event_to_bytes(ke: &KeyEvent) -> Vec<u8> {
    match ke.key_code {
        KeyCode::ReturnKey => vec![0x0d],
        KeyCode::Tab => vec![0x09],
        KeyCode::Escape => vec![0x1b],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::ArrowUp => vec![0x1b, b'[', b'A'],
        KeyCode::ArrowDown => vec![0x1b, b'[', b'B'],
        KeyCode::ArrowRight => vec![0x1b, b'[', b'C'],
        KeyCode::ArrowLeft => vec![0x1b, b'[', b'D'],
        KeyCode::Home => vec![0x1b, b'[', b'H'],
        KeyCode::End => vec![0x1b, b'[', b'F'],
        KeyCode::PageUp => vec![0x1b, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1b, b'[', b'6', b'~'],
        KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
        _ => {
            if let Some(c) = keycode_to_char(&ke.key_code, ke.modifiers.shift) {
                if ke.modifiers.control {
                    let c = c.to_ascii_lowercase();
                    if ('a'..='z').contains(&c) {
                        return vec![(c as u8) - b'a' + 1];
                    }
                }
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                return s.as_bytes().to_vec();
            }
            vec![]
        }
    }
}

fn keycode_to_char(kc: &KeyCode, shift: bool) -> Option<char> {
    match kc {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Key0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Key1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Key2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Key3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Key4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Key5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Key6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Key7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Key8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Key9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equals => Some(if shift { '+' } else { '=' }),
        KeyCode::LBracket => Some(if shift { '{' } else { '[' }),
        KeyCode::RBracket => Some(if shift { '}' } else { ']' }),
        KeyCode::Backslash => Some(if shift { '|' } else { '\\' }),
        KeyCode::Semicolon => Some(if shift { ':' } else { ';' }),
        KeyCode::Quote => Some(if shift { '"' } else { '\'' }),
        KeyCode::Comma => Some(if shift { '<' } else { ',' }),
        KeyCode::Period => Some(if shift { '>' } else { '.' }),
        KeyCode::Slash => Some(if shift { '?' } else { '/' }),
        KeyCode::Backtick => Some(if shift { '~' } else { '`' }),
        _ => None,
    }
}

app_main!(LeuwiApp);
