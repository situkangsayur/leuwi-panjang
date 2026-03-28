use makepad_widgets::*;
use arboard::Clipboard;

use crate::tab_manager::{TabManager, SplitDir};
use crate::terminal_view;
use crate::menu;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    // TerminalGrid will be added when custom Makepad draw pipeline is ready

    // Dark Green Theme
    LEUWI_BG      = #x0A1410D9   // 85% opacity (D9 = 217/255 = 0.85)
    LEUWI_FG      = #xB8D4CCFF
    LEUWI_GREEN   = #x00FF88FF
    LEUWI_TAB_BG  = #x060F0BFF
    LEUWI_TAB_ACT = #x0A1410FF
    LEUWI_BORDER  = #x1A3A28FF
    LEUWI_DIM     = #x5C8A72FF
    LEUWI_HOVER   = #x0D1F17FF

    TERM_STYLE = {
        font_size: 13.0,
        line_spacing: 1.5,
    }

    LeuwiTab = <Button> {
        width: Fit, height: Fill,
        padding: { left: 14, right: 14 },
        margin: { right: 1 },
        draw_bg: {
            color: (LEUWI_TAB_ACT)
            fn pixel(self) -> vec4 { return self.color; }
        }
        draw_text: { color: (LEUWI_FG), text_style: { font_size: 10.0 } }
    }

    LeuwiTabInactive = <Button> {
        width: Fit, height: Fill,
        padding: { left: 14, right: 14 },
        margin: { right: 1 },
        draw_bg: {
            color: (LEUWI_TAB_BG)
            fn pixel(self) -> vec4 { return self.color; }
        }
        draw_text: { color: (LEUWI_DIM), text_style: { font_size: 10.0 } }
    }

    LeuwiSmallBtn = <Button> {
        width: 36, height: Fill,
        padding: { left: 6, right: 6 },
        draw_bg: {
            color: #x00000000
            fn pixel(self) -> vec4 {
                return mix(self.color, #x30363D40, self.hover);
            }
        }
        draw_text: { color: (LEUWI_DIM), text_style: { font_size: 12.0 } }
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

            window_menu = <WindowMenu> { main = Main { items: [] } }

            caption_bar = <SolidView> {
                visible: true,
                flow: Right,
                height: 34,
                draw_bg: { color: (LEUWI_TAB_BG) }

                caption_label = <View> { visible: false, width: 0, height: 0 }

                tabs_area = <View> {
                    width: Fill, height: Fill,
                    flow: Right,
                    align: { y: 1.0 },
                    padding: { left: 8, top: 4 },

                    tab1 = <LeuwiTab> { text: " Terminal 1 " }
                    tab2 = <LeuwiTabInactive> { visible: false, text: " Terminal 2 " }
                    tab3 = <LeuwiTabInactive> { visible: false, text: " Terminal 3 " }
                    tab4 = <LeuwiTabInactive> { visible: false, text: " Terminal 4 " }
                    tab5 = <LeuwiTabInactive> { visible: false, text: " Terminal 5 " }

                    new_tab_btn = <LeuwiSmallBtn> {
                        width: 28,
                        text: "+"
                        draw_text: { color: (LEUWI_DIM), text_style: { font_size: 16.0 } }
                    }
                }

                right_controls = <View> {
                    width: Fit, height: Fill,
                    flow: Right,
                    align: { y: 0.5 },

                    menu_btn = <LeuwiSmallBtn> {
                        text: "≡"
                        draw_text: { text_style: { font_size: 16.0 } }
                    }

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
                flow: Down, spacing: 0,

                // Main terminal area
                pane_container = <View> {
                    width: Fill, height: Fill,
                    flow: Right, spacing: 0,

                    pane1_view = <View> {
                        width: Fill, height: Fill,
                        show_bg: true,
                        draw_bg: { color: (LEUWI_BG) }
                        padding: { top: 6, bottom: 4, left: 10, right: 10 },
                        terminal_output = <Label> {
                            width: Fill,
                            text: ""
                            draw_text: { color: (LEUWI_FG), text_style: (TERM_STYLE) }
                        }
                    }

                    split_divider = <View> {
                        visible: false,
                        width: 2, height: Fill,
                        show_bg: true,
                        draw_bg: { color: (LEUWI_BORDER) }
                    }

                    pane2_view = <View> {
                        visible: false,
                        width: Fill, height: Fill,
                        show_bg: true,
                        draw_bg: { color: (LEUWI_BG) }
                        padding: { top: 6, bottom: 4, left: 10, right: 10 },
                        terminal_output2 = <Label> {
                            width: Fill,
                            text: ""
                            draw_text: { color: (LEUWI_FG), text_style: (TERM_STYLE) }
                        }
                    }
                }

                // Menu panel (right side, hidden)
                menu_panel = <View> {
                    visible: false,
                    width: 300, height: Fill,
                    show_bg: true,
                    draw_bg: { color: (LEUWI_TAB_BG) }
                    padding: { top: 8, left: 0, right: 0 },
                    flow: Down,

                    menu_title = <Label> {
                        width: Fill,
                        padding: { left: 16, bottom: 8 },
                        text: "Leuwi Panjang"
                        draw_text: { color: (LEUWI_GREEN), text_style: { font_size: 13.0 } }
                    }
                    menu_content = <Label> {
                        width: Fill,
                        padding: { left: 16, right: 16 },
                        text: ""
                        draw_text: { color: (LEUWI_FG), text_style: { font_size: 11.0, line_spacing: 1.6 } }
                    }
                }

                // Status bar
                status_bar = <View> {
                    width: Fill, height: 22,
                    flow: Right, align: { y: 0.5 },
                    padding: { left: 12, right: 12 },
                    show_bg: true,
                    draw_bg: { color: (LEUWI_TAB_BG) }

                    status_dot = <Label> {
                        text: "●"
                        draw_text: { color: (LEUWI_GREEN), text_style: { font_size: 8.0 } }
                    }
                    status_text = <Label> {
                        margin: { left: 6 },
                        text: "leuwi-panjang v0.1.0"
                        draw_text: { color: (LEUWI_DIM), text_style: { font_size: 9.0 } }
                    }
                    <View> { width: Fill, height: Fill }
                    status_info = <Label> {
                        text: ""
                        draw_text: { color: (LEUWI_DIM), text_style: { font_size: 9.0 } }
                    }
                }
            }
        }
    }
}

#[derive(Live, LiveHook)]
pub struct LeuwiApp {
    #[live] ui: WidgetRef,
    #[rust] tabs: Option<TabManager>,
    #[rust] menu_open: bool,
    #[rust] initialized: bool,
    #[rust] last_width: f64,
    #[rust] last_height: f64,
}

impl LiveRegister for LeuwiApp {
    fn live_register(cx: &mut Cx) {
        makepad_widgets::live_design(cx);
    }
}

impl LeuwiApp {
    fn init_terminal(&mut self, cx: &mut Cx) {
        if self.initialized { return; }
        self.initialized = true;

        #[cfg(target_os = "linux")]
        {
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(200));
                let _ = std::process::Command::new("xprop")
                    .args(["-name", "Leuwi Panjang", "-f", "_MOTIF_WM_HINTS", "32c",
                           "-set", "_MOTIF_WM_HINTS", "0x2, 0x0, 0x0, 0x0, 0x0"])
                    .output();
            });
        }

        let config = leuwi_config::load_config();
        let cols: u16 = 120;
        let rows: u16 = 40;

        self.tabs = Some(TabManager::new(
            &config.general.default_shell, cols, rows, config.scrollback.lines as usize,
        ));

        cx.start_interval(0.016); // 60fps
    }

    fn update_tab_ui(&mut self, cx: &mut Cx) {
        let tabs = match &self.tabs { Some(t) => t, None => return };
        let tab_ids = [id!(tab1), id!(tab2), id!(tab3), id!(tab4), id!(tab5)];

        for (i, tab_id) in tab_ids.iter().enumerate() {
            if i < tabs.tab_count() {
                self.ui.button(*tab_id).set_visible(cx, true);
                let title = format!(" Terminal {} ", i + 1);
                self.ui.button(*tab_id).set_text(cx, &title);
            } else {
                self.ui.button(*tab_id).set_visible(cx, false);
            }
        }

        // Status info
        let tab = tabs.active_tab();
        let info = if tab.is_split() {
            format!("Tab {} | Pane {}/{} | Split", tabs.active_tab + 1, tab.active_pane + 1, tab.pane_count())
        } else {
            format!("Tab {}/{}", tabs.active_tab + 1, tabs.tab_count())
        };
        self.ui.label(id!(status_info)).set_text(cx, &info);
    }

    fn update_split_ui(&mut self, cx: &mut Cx) {
        let tabs = match &self.tabs { Some(t) => t, None => return };
        let tab = tabs.active_tab();
        let has_split = tab.is_split();

        self.ui.view(id!(split_divider)).set_visible(cx, has_split);
        self.ui.view(id!(pane2_view)).set_visible(cx, has_split);
    }

    fn toggle_menu(&mut self, cx: &mut Cx) {
        self.menu_open = !self.menu_open;
        self.ui.view(id!(menu_panel)).set_visible(cx, self.menu_open);
        if self.menu_open {
            let items = menu::build_main_menu();
            let text = menu::menu_to_text(&items, 0);
            self.ui.label(id!(menu_content)).set_text(cx, &text);
        }
        self.ui.redraw(cx);
    }

    fn copy_selection(&self) {
        let tabs = match &self.tabs { Some(t) => t, None => return };
        let text = tabs.active_tab().active_pane().copy_selected_or_all();
        if !text.is_empty() {
            if let Ok(mut cb) = Clipboard::new() {
                let _ = cb.set_text(&text);
            }
        }
    }

    fn paste_clipboard(&mut self) {
        let text = match Clipboard::new() {
            Ok(mut cb) => cb.get_text().unwrap_or_default(),
            Err(_) => return,
        };
        if text.is_empty() { return; }

        if let Some(tabs) = &mut self.tabs {
            // Bracketed paste
            tabs.write_active(b"\x1b[200~");
            tabs.write_active(text.as_bytes());
            tabs.write_active(b"\x1b[201~");
        }
    }

    fn handle_resize(&mut self, cx: &mut Cx, width: f64, height: f64) {
        if (width - self.last_width).abs() < 1.0 && (height - self.last_height).abs() < 1.0 {
            return;
        }
        self.last_width = width;
        self.last_height = height;

        // Calculate terminal grid size from window size
        // Approximate: cell ~8px wide, ~18px tall, minus chrome
        let chrome_height = 34.0 + 22.0; // tab bar + status bar
        let padding = 20.0;
        let cols = ((width - padding) / 8.0).max(20.0) as u16;
        let rows = ((height - chrome_height - padding) / 18.0).max(5.0) as u16;

        if let Some(tabs) = &mut self.tabs {
            tabs.resize(cols, rows);
        }
    }
}

impl MatchEvent for LeuwiApp {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        // Tab buttons
        let tab_ids = [id!(tab1), id!(tab2), id!(tab3), id!(tab4), id!(tab5)];
        for (i, tab_id) in tab_ids.iter().enumerate() {
            if self.ui.button(*tab_id).clicked(actions) {
                if let Some(tabs) = &mut self.tabs {
                    tabs.switch_tab(i);
                    self.update_split_ui(cx);
                    self.ui.redraw(cx);
                }
            }
        }

        if self.ui.button(id!(new_tab_btn)).clicked(actions) {
            if let Some(tabs) = &mut self.tabs {
                tabs.new_tab();
                self.update_tab_ui(cx);
                self.update_split_ui(cx);
                self.ui.redraw(cx);
            }
        }
        if self.ui.button(id!(menu_btn)).clicked(actions) {
            self.toggle_menu(cx);
        }
    }
}

impl AppMain for LeuwiApp {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.match_event(cx, event);
        self.ui.handle_event(cx, event, &mut Scope::empty());

        match event {
            Event::Startup => {
                self.init_terminal(cx);
            }
            Event::Timer(_) => {
                if let Some(tabs) = &self.tabs {
                    let tab = tabs.active_tab();

                    // Pane 1
                    let lines1 = tab.panes[0].render_colored_text();
                    let text1 = terminal_view::colored_lines_to_text(&lines1);
                    self.ui.label(id!(terminal_output)).set_text(cx, &text1);

                    // Pane 2 (if split)
                    if tab.is_split() && tab.panes.len() > 1 {
                        let lines2 = tab.panes[1].render_colored_text();
                        let text2 = terminal_view::colored_lines_to_text(&lines2);
                        self.ui.label(id!(terminal_output2)).set_text(cx, &text2);
                    }

                    self.update_tab_ui(cx);
                }
                self.ui.redraw(cx);
            }
            Event::WindowGeomChange(ev) => {
                let size = ev.new_geom.inner_size;
                self.handle_resize(cx, size.x, size.y);
            }
            Event::KeyDown(ke) => {
                // Escape closes menu
                if ke.key_code == KeyCode::Escape {
                    if self.menu_open {
                        self.toggle_menu(cx);
                        return;
                    }
                }

                // Ctrl+Shift combos
                if ke.modifiers.control && ke.modifiers.shift {
                    match ke.key_code {
                        KeyCode::KeyC => { self.copy_selection(); return; }
                        KeyCode::KeyV => { self.paste_clipboard(); return; }
                        KeyCode::KeyD => {
                            if let Some(tabs) = &mut self.tabs {
                                tabs.split_active(SplitDir::Vertical);
                                self.update_split_ui(cx);
                            }
                            return;
                        }
                        KeyCode::KeyE => {
                            if let Some(tabs) = &mut self.tabs {
                                tabs.split_active(SplitDir::Horizontal);
                                self.update_split_ui(cx);
                            }
                            return;
                        }
                        KeyCode::KeyW => {
                            if let Some(tabs) = &mut self.tabs {
                                let tab = tabs.active_tab_mut();
                                if tab.is_split() {
                                    tab.close_split();
                                    self.update_split_ui(cx);
                                } else if tabs.tab_count() > 1 {
                                    tabs.close_active_tab();
                                    self.update_tab_ui(cx);
                                    self.update_split_ui(cx);
                                }
                            }
                            return;
                        }
                        KeyCode::KeyT => {
                            if let Some(tabs) = &mut self.tabs {
                                tabs.new_tab();
                                self.update_tab_ui(cx);
                                self.update_split_ui(cx);
                            }
                            return;
                        }
                        KeyCode::KeyA => {
                            if let Some(tabs) = &self.tabs {
                                tabs.active_tab().active_pane().select_all();
                            }
                            return;
                        }
                        _ => {}
                    }
                }

                // Ctrl+Tab / Ctrl+Shift+Tab = switch tabs
                if ke.modifiers.control && ke.key_code == KeyCode::Tab {
                    if let Some(tabs) = &mut self.tabs {
                        if ke.modifiers.shift { tabs.prev_tab(); } else { tabs.next_tab(); }
                        self.update_tab_ui(cx);
                        self.update_split_ui(cx);
                    }
                    return;
                }

                // Alt+1-5 = go to tab N
                if ke.modifiers.alt {
                    let tab_num = match ke.key_code {
                        KeyCode::Key1 => Some(0),
                        KeyCode::Key2 => Some(1),
                        KeyCode::Key3 => Some(2),
                        KeyCode::Key4 => Some(3),
                        KeyCode::Key5 => Some(4),
                        KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                            if let Some(tabs) = &mut self.tabs {
                                tabs.active_tab_mut().toggle_pane();
                            }
                            return;
                        }
                        _ => None,
                    };
                    if let Some(n) = tab_num {
                        if let Some(tabs) = &mut self.tabs {
                            tabs.switch_tab(n);
                            self.update_tab_ui(cx);
                            self.update_split_ui(cx);
                        }
                        return;
                    }
                }

                // Shift+PageUp/Down = scroll
                if ke.modifiers.shift {
                    match ke.key_code {
                        KeyCode::PageUp | KeyCode::PageDown => {
                            // TODO: scrollback
                            return;
                        }
                        _ => {}
                    }
                }

                // Forward to PTY
                let bytes = key_event_to_bytes(ke);
                if !bytes.is_empty() {
                    if let Some(tabs) = &mut self.tabs {
                        tabs.write_active(&bytes);
                    }
                }
            }
            Event::MouseDown(me) => {
                // Close menu on click outside
                if self.menu_open {
                    self.toggle_menu(cx);
                    return;
                }
                // Start selection
                let col = ((me.abs.x - 10.0) / 8.0).max(0.0) as usize;
                let row = ((me.abs.y - 40.0) / 18.0).max(0.0) as usize;
                if let Some(tabs) = &self.tabs {
                    tabs.active_tab().active_pane().start_selection(row, col);
                }
            }
            Event::MouseMove(me) => {
                let col = ((me.abs.x - 10.0) / 8.0).max(0.0) as usize;
                let row = ((me.abs.y - 40.0) / 18.0).max(0.0) as usize;
                if let Some(tabs) = &self.tabs {
                    tabs.active_tab().active_pane().update_selection(row, col);
                }
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
                    let cl = c.to_ascii_lowercase();
                    if ('a'..='z').contains(&cl) {
                        return vec![(cl as u8) - b'a' + 1];
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
    let c = match kc {
        KeyCode::KeyA => 'a', KeyCode::KeyB => 'b', KeyCode::KeyC => 'c',
        KeyCode::KeyD => 'd', KeyCode::KeyE => 'e', KeyCode::KeyF => 'f',
        KeyCode::KeyG => 'g', KeyCode::KeyH => 'h', KeyCode::KeyI => 'i',
        KeyCode::KeyJ => 'j', KeyCode::KeyK => 'k', KeyCode::KeyL => 'l',
        KeyCode::KeyM => 'm', KeyCode::KeyN => 'n', KeyCode::KeyO => 'o',
        KeyCode::KeyP => 'p', KeyCode::KeyQ => 'q', KeyCode::KeyR => 'r',
        KeyCode::KeyS => 's', KeyCode::KeyT => 't', KeyCode::KeyU => 'u',
        KeyCode::KeyV => 'v', KeyCode::KeyW => 'w', KeyCode::KeyX => 'x',
        KeyCode::KeyY => 'y', KeyCode::KeyZ => 'z',
        KeyCode::Key0 => '0', KeyCode::Key1 => '1', KeyCode::Key2 => '2',
        KeyCode::Key3 => '3', KeyCode::Key4 => '4', KeyCode::Key5 => '5',
        KeyCode::Key6 => '6', KeyCode::Key7 => '7', KeyCode::Key8 => '8',
        KeyCode::Key9 => '9',
        KeyCode::Space => return Some(' '),
        KeyCode::Minus => return Some(if shift { '_' } else { '-' }),
        KeyCode::Equals => return Some(if shift { '+' } else { '=' }),
        KeyCode::LBracket => return Some(if shift { '{' } else { '[' }),
        KeyCode::RBracket => return Some(if shift { '}' } else { ']' }),
        KeyCode::Backslash => return Some(if shift { '|' } else { '\\' }),
        KeyCode::Semicolon => return Some(if shift { ':' } else { ';' }),
        KeyCode::Quote => return Some(if shift { '"' } else { '\'' }),
        KeyCode::Comma => return Some(if shift { '<' } else { ',' }),
        KeyCode::Period => return Some(if shift { '>' } else { '.' }),
        KeyCode::Slash => return Some(if shift { '?' } else { '/' }),
        KeyCode::Backtick => return Some(if shift { '~' } else { '`' }),
        _ => return None,
    };
    if shift && c.is_alphabetic() {
        Some(c.to_ascii_uppercase())
    } else if shift {
        Some(match c {
            '0' => ')', '1' => '!', '2' => '@', '3' => '#', '4' => '$',
            '5' => '%', '6' => '^', '7' => '&', '8' => '*', '9' => '(',
            _ => c,
        })
    } else {
        Some(c)
    }
}

app_main!(LeuwiApp);
