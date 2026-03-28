use crate::cell::{CellAttributes, Color};
use crate::grid::Grid;

/// Terminal parser that processes VT escape sequences and updates the grid.
pub struct Parser {
    state: ParserState,
    params: Vec<u16>,
    intermediates: Vec<u8>,
    current_attrs: CellAttributes,
    /// True if CSI sequence started with '?' (private mode)
    private_mode: bool,
    /// UTF-8 accumulation buffer
    utf8_buf: Vec<u8>,
    utf8_remaining: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ParserState {
    Ground,
    Escape,
    EscapeIntermediate,
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    OscString,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            state: ParserState::Ground,
            params: Vec::with_capacity(16),
            intermediates: Vec::with_capacity(4),
            current_attrs: CellAttributes::default(),
            private_mode: false,
            utf8_buf: Vec::with_capacity(4),
            utf8_remaining: 0,
        }
    }

    /// Process a byte stream and apply changes to the grid.
    pub fn process(&mut self, data: &[u8], grid: &mut Grid) {
        for &byte in data {
            self.process_byte(byte, grid);
        }
    }

    fn process_byte(&mut self, byte: u8, grid: &mut Grid) {
        match self.state {
            ParserState::Ground => self.ground(byte, grid),
            ParserState::Escape => self.escape(byte, grid),
            ParserState::EscapeIntermediate => self.escape_intermediate(byte, grid),
            ParserState::CsiEntry => self.csi_entry(byte, grid),
            ParserState::CsiParam => self.csi_param(byte, grid),
            ParserState::CsiIntermediate => self.csi_intermediate(byte, grid),
            ParserState::OscString => self.osc_string(byte, grid),
        }
    }

    fn ground(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            // ESC
            0x1b => {
                self.state = ParserState::Escape;
            }
            // Newline / Line Feed
            0x0a | 0x0b | 0x0c => {
                grid.newline();
            }
            // Carriage Return
            0x0d => {
                grid.cursor_col = 0;
            }
            // Backspace
            0x08 => {
                grid.cursor_col = grid.cursor_col.saturating_sub(1);
            }
            // Tab
            0x09 => {
                let next_tab = (grid.cursor_col / 8 + 1) * 8;
                grid.cursor_col = next_tab.min(grid.cols() - 1);
            }
            // Bell
            0x07 => {
                // TODO: visual/audible bell
            }
            // Printable ASCII characters
            0x20..=0x7e => {
                if grid.cursor_col >= grid.cols() {
                    grid.newline();
                }
                let cell = grid.cell_mut(grid.cursor_row, grid.cursor_col);
                cell.c = byte as char;
                cell.attrs = self.current_attrs.clone();
                cell.dirty = true;
                grid.cursor_col += 1;
            }
            // UTF-8 multi-byte start
            0xc0..=0xdf => {
                self.utf8_buf.clear();
                self.utf8_buf.push(byte);
                self.utf8_remaining = 1;
            }
            0xe0..=0xef => {
                self.utf8_buf.clear();
                self.utf8_buf.push(byte);
                self.utf8_remaining = 2;
            }
            0xf0..=0xf7 => {
                self.utf8_buf.clear();
                self.utf8_buf.push(byte);
                self.utf8_remaining = 3;
            }
            // UTF-8 continuation byte
            0x80..=0xbf if self.utf8_remaining > 0 => {
                self.utf8_buf.push(byte);
                self.utf8_remaining -= 1;
                if self.utf8_remaining == 0 {
                    if let Ok(s) = std::str::from_utf8(&self.utf8_buf) {
                        for c in s.chars() {
                            if grid.cursor_col >= grid.cols() {
                                grid.newline();
                            }
                            let cell = grid.cell_mut(grid.cursor_row, grid.cursor_col);
                            cell.c = c;
                            cell.attrs = self.current_attrs.clone();
                            cell.dirty = true;
                            grid.cursor_col += 1;
                        }
                    }
                    self.utf8_buf.clear();
                }
            }
            _ => {
                // Unknown byte — ignore
            }
        }
    }

    fn escape(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            // CSI - Control Sequence Introducer
            b'[' => {
                self.params.clear();
                self.intermediates.clear();
                self.private_mode = false;
                self.state = ParserState::CsiEntry;
            }
            // OSC - Operating System Command
            b']' => {
                self.state = ParserState::OscString;
            }
            // DCS - Device Control String (skip until ST)
            b'P' => {
                self.state = ParserState::OscString; // reuse OSC handler to skip
            }
            // APC - Application Program Command (skip)
            b'_' => {
                self.state = ParserState::OscString;
            }
            // PM - Privacy Message (skip)
            b'^' => {
                self.state = ParserState::OscString;
            }
            // RIS - Reset to Initial State
            b'c' => {
                grid.clear();
                self.current_attrs = CellAttributes::default();
                self.state = ParserState::Ground;
            }
            // IND - Index (move down one line)
            b'D' => {
                grid.newline();
                self.state = ParserState::Ground;
            }
            // NEL - Next Line
            b'E' => {
                grid.cursor_col = 0;
                grid.newline();
                self.state = ParserState::Ground;
            }
            // Save cursor
            b'7' => {
                // TODO: save cursor position and attributes
                self.state = ParserState::Ground;
            }
            // Restore cursor
            b'8' => {
                // TODO: restore cursor position and attributes
                self.state = ParserState::Ground;
            }
            0x20..=0x2f => {
                self.intermediates.push(byte);
                self.state = ParserState::EscapeIntermediate;
            }
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn escape_intermediate(&mut self, byte: u8, _grid: &mut Grid) {
        match byte {
            0x20..=0x2f => {
                self.intermediates.push(byte);
            }
            0x30..=0x7e => {
                // Final byte — dispatch escape sequence
                // TODO: handle specific escape sequences
                self.state = ParserState::Ground;
            }
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn csi_entry(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            // Private mode prefix: '?' for DECSET/DECRST, '>' for DA2, etc.
            b'?' | b'>' | b'!' | b'=' => {
                self.private_mode = byte == b'?';
                self.intermediates.push(byte);
                self.state = ParserState::CsiParam;
            }
            b'0'..=b'9' => {
                self.params.push((byte - b'0') as u16);
                self.state = ParserState::CsiParam;
            }
            b';' => {
                self.params.push(0);
                self.state = ParserState::CsiParam;
            }
            0x20..=0x2f => {
                self.intermediates.push(byte);
                self.state = ParserState::CsiIntermediate;
            }
            0x40..=0x7e => {
                self.dispatch_csi(byte, grid);
                self.state = ParserState::Ground;
            }
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn csi_param(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            b'0'..=b'9' => {
                if let Some(last) = self.params.last_mut() {
                    *last = last.saturating_mul(10).saturating_add((byte - b'0') as u16);
                }
            }
            b';' => {
                self.params.push(0);
            }
            0x20..=0x2f => {
                self.intermediates.push(byte);
                self.state = ParserState::CsiIntermediate;
            }
            0x40..=0x7e => {
                self.dispatch_csi(byte, grid);
                self.state = ParserState::Ground;
            }
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn csi_intermediate(&mut self, byte: u8, grid: &mut Grid) {
        match byte {
            0x20..=0x2f => {
                self.intermediates.push(byte);
            }
            0x40..=0x7e => {
                self.dispatch_csi(byte, grid);
                self.state = ParserState::Ground;
            }
            _ => {
                self.state = ParserState::Ground;
            }
        }
    }

    fn osc_string(&mut self, byte: u8, _grid: &mut Grid) {
        match byte {
            // ST (String Terminator) via BEL
            0x07 => {
                // TODO: process OSC command
                self.state = ParserState::Ground;
            }
            // ESC (might be ESC \ = ST)
            0x1b => {
                // TODO: handle ESC \ properly
                self.state = ParserState::Ground;
            }
            _ => {
                // Accumulate OSC data
                // TODO: buffer OSC string content
            }
        }
    }

    fn dispatch_csi(&mut self, final_byte: u8, grid: &mut Grid) {
        let _is_private = self.private_mode;
        self.private_mode = false;

        let param = |idx: usize, default: u16| -> u16 {
            self.params.get(idx).copied().filter(|&v| v != 0).unwrap_or(default)
        };

        match final_byte {
            // CUU - Cursor Up
            b'A' => {
                let n = param(0, 1) as usize;
                grid.cursor_row = grid.cursor_row.saturating_sub(n);
            }
            // CUD - Cursor Down
            b'B' => {
                let n = param(0, 1) as usize;
                grid.cursor_row = (grid.cursor_row + n).min(grid.rows() - 1);
            }
            // CUF - Cursor Forward
            b'C' => {
                let n = param(0, 1) as usize;
                grid.cursor_col = (grid.cursor_col + n).min(grid.cols() - 1);
            }
            // CUB - Cursor Backward
            b'D' => {
                let n = param(0, 1) as usize;
                grid.cursor_col = grid.cursor_col.saturating_sub(n);
            }
            // CUP - Cursor Position
            b'H' | b'f' => {
                let row = (param(0, 1) as usize).saturating_sub(1);
                let col = (param(1, 1) as usize).saturating_sub(1);
                grid.cursor_row = row.min(grid.rows() - 1);
                grid.cursor_col = col.min(grid.cols() - 1);
            }
            // ED - Erase in Display
            b'J' => {
                let mode = param(0, 0);
                match mode {
                    0 => {
                        // Clear from cursor to end
                        for col in grid.cursor_col..grid.cols() {
                            *grid.cell_mut(grid.cursor_row, col) = crate::cell::Cell::blank();
                        }
                        for row in (grid.cursor_row + 1)..grid.rows() {
                            for col in 0..grid.cols() {
                                *grid.cell_mut(row, col) = crate::cell::Cell::blank();
                            }
                        }
                    }
                    1 => {
                        // Clear from start to cursor
                        for row in 0..grid.cursor_row {
                            for col in 0..grid.cols() {
                                *grid.cell_mut(row, col) = crate::cell::Cell::blank();
                            }
                        }
                        for col in 0..=grid.cursor_col.min(grid.cols() - 1) {
                            *grid.cell_mut(grid.cursor_row, col) = crate::cell::Cell::blank();
                        }
                    }
                    2 | 3 => {
                        grid.clear();
                    }
                    _ => {}
                }
            }
            // EL - Erase in Line
            b'K' => {
                let mode = param(0, 0);
                let row = grid.cursor_row;
                match mode {
                    0 => {
                        for col in grid.cursor_col..grid.cols() {
                            *grid.cell_mut(row, col) = crate::cell::Cell::blank();
                        }
                    }
                    1 => {
                        for col in 0..=grid.cursor_col.min(grid.cols() - 1) {
                            *grid.cell_mut(row, col) = crate::cell::Cell::blank();
                        }
                    }
                    2 => {
                        for col in 0..grid.cols() {
                            *grid.cell_mut(row, col) = crate::cell::Cell::blank();
                        }
                    }
                    _ => {}
                }
            }
            // SGR - Select Graphic Rendition
            b'm' => {
                self.process_sgr();
            }
            // DECSET / DECRST (private modes) — silently accept
            // Handles: ?1h (DECCKM), ?25h/l (cursor visible), ?2004h/l (bracketed paste),
            // ?1049h/l (alt screen), ?7h/l (autowrap), ?1h/l (app cursor keys), etc.
            b'h' | b'l' => {
                // Private modes and standard modes — accepted but not all implemented yet
                // This prevents escape code text from leaking to screen
            }
            // DECSTBM - Set Scrolling Region
            b'r' => {
                // TODO: implement scroll regions
            }
            // ICH - Insert Characters
            b'@' => {
                // TODO: implement
            }
            // DCH - Delete Characters
            b'P' => {
                // TODO: implement
            }
            // IL - Insert Lines
            b'L' => {
                // TODO: implement
            }
            // DL - Delete Lines
            b'M' => {
                // TODO: implement
            }
            // ECH - Erase Characters
            b'X' => {
                let n = param(0, 1) as usize;
                let row = grid.cursor_row;
                for i in 0..n {
                    let col = grid.cursor_col + i;
                    if col < grid.cols() {
                        *grid.cell_mut(row, col) = crate::cell::Cell::blank();
                    }
                }
            }
            // SU - Scroll Up
            b'S' => {
                // TODO: implement scroll up N lines
            }
            // SD - Scroll Down
            b'T' => {
                // TODO: implement scroll down N lines
            }
            // CUP with column only - Cursor Horizontal Absolute
            b'G' => {
                let col = (param(0, 1) as usize).saturating_sub(1);
                grid.cursor_col = col.min(grid.cols() - 1);
            }
            // VPA - Cursor Vertical Absolute
            b'd' => {
                let row = (param(0, 1) as usize).saturating_sub(1);
                grid.cursor_row = row.min(grid.rows() - 1);
            }
            // DSR - Device Status Report
            b'n' => {
                // TODO: respond with cursor position
            }
            // CNL - Cursor Next Line
            b'E' => {
                let n = param(0, 1) as usize;
                grid.cursor_col = 0;
                grid.cursor_row = (grid.cursor_row + n).min(grid.rows() - 1);
            }
            // CPL - Cursor Previous Line
            b'F' => {
                let n = param(0, 1) as usize;
                grid.cursor_col = 0;
                grid.cursor_row = grid.cursor_row.saturating_sub(n);
            }
            // REP - Repeat preceding character
            b'b' => {
                // TODO: implement
            }
            _ => {
                // Silently ignore unhandled CSI sequences (don't leak to screen)
            }
        }
    }

    fn process_sgr(&mut self) {
        if self.params.is_empty() {
            self.current_attrs = CellAttributes::default();
            return;
        }

        let mut i = 0;
        while i < self.params.len() {
            match self.params[i] {
                0 => self.current_attrs = CellAttributes::default(),
                1 => self.current_attrs.bold = true,
                2 => self.current_attrs.dim = true,
                3 => self.current_attrs.italic = true,
                4 => self.current_attrs.underline = crate::cell::UnderlineStyle::Single,
                7 => self.current_attrs.inverse = true,
                8 => self.current_attrs.hidden = true,
                9 => self.current_attrs.strikethrough = true,
                22 => { self.current_attrs.bold = false; self.current_attrs.dim = false; }
                23 => self.current_attrs.italic = false,
                24 => self.current_attrs.underline = crate::cell::UnderlineStyle::None,
                27 => self.current_attrs.inverse = false,
                28 => self.current_attrs.hidden = false,
                29 => self.current_attrs.strikethrough = false,
                // Standard foreground colors
                30..=37 => {
                    self.current_attrs.fg = ansi_color(self.params[i] - 30);
                }
                39 => self.current_attrs.fg = Color::default(),
                // Standard background colors
                40..=47 => {
                    self.current_attrs.bg = ansi_color(self.params[i] - 40);
                }
                49 => self.current_attrs.bg = Color::rgb(10, 20, 16), // default bg
                // 256-color and truecolor
                38 => {
                    if i + 1 < self.params.len() {
                        match self.params[i + 1] {
                            5 if i + 2 < self.params.len() => {
                                self.current_attrs.fg = color_256(self.params[i + 2]);
                                i += 2;
                            }
                            2 if i + 4 < self.params.len() => {
                                self.current_attrs.fg = Color::rgb(
                                    self.params[i + 2] as u8,
                                    self.params[i + 3] as u8,
                                    self.params[i + 4] as u8,
                                );
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                48 => {
                    if i + 1 < self.params.len() {
                        match self.params[i + 1] {
                            5 if i + 2 < self.params.len() => {
                                self.current_attrs.bg = color_256(self.params[i + 2]);
                                i += 2;
                            }
                            2 if i + 4 < self.params.len() => {
                                self.current_attrs.bg = Color::rgb(
                                    self.params[i + 2] as u8,
                                    self.params[i + 3] as u8,
                                    self.params[i + 4] as u8,
                                );
                                i += 4;
                            }
                            _ => {}
                        }
                    }
                }
                // Bright foreground
                90..=97 => {
                    self.current_attrs.fg = ansi_bright_color(self.params[i] - 90);
                }
                // Bright background
                100..=107 => {
                    self.current_attrs.bg = ansi_bright_color(self.params[i] - 100);
                }
                _ => {}
            }
            i += 1;
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// ANSI colors — high contrast for dark green background (#0A1410)
fn ansi_color(idx: u16) -> Color {
    match idx {
        0 => Color::rgb(40, 50, 45),     // black (visible on dark bg)
        1 => Color::rgb(255, 85, 85),    // red — bright, alert
        2 => Color::rgb(0, 255, 136),    // green — neon, success
        3 => Color::rgb(255, 214, 102),  // yellow — warm, warning
        4 => Color::rgb(100, 160, 255),  // blue — visible on dark
        5 => Color::rgb(210, 140, 255),  // magenta — soft purple
        6 => Color::rgb(80, 220, 240),   // cyan — bright info
        7 => Color::rgb(200, 215, 205),  // white — slightly green tint
        _ => Color::default(),
    }
}

fn ansi_bright_color(idx: u16) -> Color {
    match idx {
        0 => Color::rgb(90, 110, 100),   // bright black (comment color)
        1 => Color::rgb(255, 120, 120),  // bright red
        2 => Color::rgb(100, 255, 170),  // bright green
        3 => Color::rgb(255, 230, 150),  // bright yellow
        4 => Color::rgb(140, 190, 255),  // bright blue
        5 => Color::rgb(230, 180, 255),  // bright magenta
        6 => Color::rgb(130, 240, 255),  // bright cyan
        7 => Color::rgb(240, 245, 240),  // bright white
        _ => Color::default(),
    }
}

fn color_256(idx: u16) -> Color {
    match idx {
        0..=7 => ansi_color(idx),
        8..=15 => ansi_bright_color(idx - 8),
        16..=231 => {
            let idx = idx - 16;
            let r = ((idx / 36) as u8) * 51;
            let g = (((idx / 6) % 6) as u8) * 51;
            let b = ((idx % 6) as u8) * 51;
            Color::rgb(r, g, b)
        }
        232..=255 => {
            let v = ((idx - 232) * 10 + 8) as u8;
            Color::rgb(v, v, v)
        }
        _ => Color::default(),
    }
}
