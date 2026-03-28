use leuwi_terminal::TerminalSession;
use leuwi_pty::Pty;
use std::sync::{Arc, Mutex};

/// A single terminal pane — owns a PTY + session.
pub struct TerminalPane {
    pub session: TerminalSession,
    pub pty: Option<Pty>,
    pub id: u32,
    pub title: String,
    pub scroll_offset: usize, // lines scrolled up from bottom
}

static PANE_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

impl TerminalPane {
    pub fn new(shell: &str, cols: u16, rows: u16, scrollback: usize) -> Self {
        let id = PANE_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let mut session = TerminalSession::new(cols as usize, rows as usize, scrollback);

        let pty = match Pty::spawn(shell, cols, rows) {
            Ok(pty) => {
                session.start_reader(pty.clone_reader());
                Some(pty)
            }
            Err(e) => {
                session.process_bytes(format!("Error: {e}\r\n").as_bytes());
                None
            }
        };

        Self {
            session,
            pty,
            id,
            title: format!("Terminal {id}"),
            scroll_offset: 0,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        if let Some(pty) = &mut self.pty {
            let _ = pty.write(data);
        }
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        if let Some(pty) = &mut self.pty {
            let _ = pty.resize(cols, rows);
        }
        let mut grid = self.session.grid.lock().unwrap();
        grid.resize(cols as usize, rows as usize);
    }

    /// Render grid to text, preserving column alignment for monospace.
    pub fn render_text(&self) -> String {
        let grid = self.session.grid.lock().unwrap();
        let cols = grid.cols();
        let rows = grid.rows();
        let mut output = String::with_capacity((cols + 1) * rows);

        let mut last_content_row = 0;
        for row in 0..rows {
            for col in 0..cols {
                let c = grid.cell(row, col).c;
                if c != '\0' && c != ' ' {
                    last_content_row = row;
                    break;
                }
            }
        }

        for row in 0..=last_content_row {
            let mut last_non_space = 0;
            for col in 0..cols {
                let c = grid.cell(row, col).c;
                if c != '\0' && c != ' ' {
                    last_non_space = col + 1;
                }
            }

            for col in 0..last_non_space {
                let c = grid.cell(row, col).c;
                output.push(if c == '\0' { ' ' } else { c });
            }

            if row < last_content_row {
                output.push('\n');
            }
        }

        output
    }

    /// Render grid with ANSI color codes for colored output.
    /// This produces text with embedded escape sequences that
    /// could be used by a color-aware renderer.
    pub fn render_colored_text(&self) -> Vec<ColoredLine> {
        let grid = self.session.grid.lock().unwrap();
        let cols = grid.cols();
        let rows = grid.rows();
        let mut lines = Vec::with_capacity(rows);

        for row in 0..rows {
            let mut spans = Vec::new();
            let mut current_text = String::new();
            let mut current_fg = grid.cell(row, 0).attrs.fg;
            let mut current_bg = grid.cell(row, 0).attrs.bg;
            let mut current_bold = grid.cell(row, 0).attrs.bold;

            for col in 0..cols {
                let cell = grid.cell(row, col);
                let fg = cell.attrs.fg;
                let bg = cell.attrs.bg;
                let bold = cell.attrs.bold;

                // If attributes changed, start new span
                if fg != current_fg || bg != current_bg || bold != current_bold {
                    if !current_text.is_empty() {
                        spans.push(ColorSpan {
                            text: std::mem::take(&mut current_text),
                            fg: [current_fg.r, current_fg.g, current_fg.b],
                            bg: [current_bg.r, current_bg.g, current_bg.b],
                            bold: current_bold,
                        });
                    }
                    current_fg = fg;
                    current_bg = bg;
                    current_bold = bold;
                }

                let c = if cell.c == '\0' { ' ' } else { cell.c };
                current_text.push(c);
            }

            // Push last span
            if !current_text.is_empty() {
                // Trim trailing spaces
                let trimmed = current_text.trim_end().to_string();
                if !trimmed.is_empty() {
                    spans.push(ColorSpan {
                        text: trimmed,
                        fg: [current_fg.r, current_fg.g, current_fg.b],
                        bg: [current_bg.r, current_bg.g, current_bg.b],
                        bold: current_bold,
                    });
                }
            }

            lines.push(ColoredLine { spans });
        }

        // Trim trailing empty lines
        while lines.last().map_or(false, |l| l.spans.is_empty()) {
            lines.pop();
        }

        lines
    }

    pub fn copy_selected_or_all(&self) -> String {
        let grid = self.session.grid.lock().unwrap();
        grid.get_selected_text().unwrap_or_else(|| grid.get_all_text())
    }

    pub fn select_all(&self) {
        let mut grid = self.session.grid.lock().unwrap();
        grid.select_all();
    }

    pub fn start_selection(&self, row: usize, col: usize) {
        let mut grid = self.session.grid.lock().unwrap();
        grid.start_selection(row, col);
    }

    pub fn update_selection(&self, row: usize, col: usize) {
        let mut grid = self.session.grid.lock().unwrap();
        grid.update_selection(row, col);
    }

    pub fn scroll_up(&mut self, lines: usize) {
        let max = self.session.grid.lock().unwrap().scrollback_len();
        self.scroll_offset = (self.scroll_offset + lines).min(max);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
}

impl Drop for TerminalPane {
    fn drop(&mut self) {
        self.session.stop();
    }
}

#[derive(Clone, Debug)]
pub struct ColoredLine {
    pub spans: Vec<ColorSpan>,
}

#[derive(Clone, Debug)]
pub struct ColorSpan {
    pub text: String,
    pub fg: [u8; 3],
    pub bg: [u8; 3],
    pub bold: bool,
}
