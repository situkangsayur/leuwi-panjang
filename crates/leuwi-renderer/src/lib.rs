// leuwi-renderer: Terminal grid renderer using Makepad's draw system
//
// Renders the terminal cell grid as a Makepad widget.
// Each cell is drawn as a colored rectangle (background) + text glyph.
// The cursor, selection, and decorations are overlaid.

// Renderer stub — rendering now handled by VTE widget in GTK4

live_design! {
    use link::theme::*;
    use link::widgets::*;

    TerminalView = {{TerminalView}} {
        width: Fill,
        height: Fill,
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct TerminalView {
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[live]
    draw_text: DrawText,
    #[walk]
    walk: Walk,
    #[layout]
    layout: Layout,
    #[rust]
    cell_width: f64,
    #[rust]
    cell_height: f64,
    #[rust]
    grid_data: Option<GridSnapshot>,
}

/// A snapshot of the grid state for rendering (avoids holding lock during draw)
pub struct GridSnapshot {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<CellSnapshot>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub cursor_visible: bool,
}

#[derive(Clone)]
pub struct CellSnapshot {
    pub c: char,
    pub fg: [f32; 4],
    pub bg: [f32; 4],
    pub bold: bool,
}

impl GridSnapshot {
    pub fn from_grid(grid: &Grid) -> Self {
        let cols = grid.cols();
        let rows = grid.rows();
        let mut cells = Vec::with_capacity(cols * rows);

        for row in 0..rows {
            for col in 0..cols {
                let cell = grid.cell(row, col);
                cells.push(CellSnapshot {
                    c: cell.c,
                    fg: term_color_to_vec4(&cell.attrs.fg),
                    bg: term_color_to_vec4(&cell.attrs.bg),
                    bold: cell.attrs.bold,
                });
            }
        }

        Self {
            cols,
            rows,
            cells,
            cursor_row: grid.cursor_row,
            cursor_col: grid.cursor_col,
            cursor_visible: true,
        }
    }
}

fn term_color_to_vec4(c: &TermColor) -> [f32; 4] {
    [
        c.r as f32 / 255.0,
        c.g as f32 / 255.0,
        c.b as f32 / 255.0,
        c.a as f32 / 255.0,
    ]
}

impl Widget for TerminalView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, _scope: &mut Scope) {
        match event {
            Event::KeyDown(ke) => {
                // Will forward to PTY
                let _key = &ke.key_code;
            }
            _ => {}
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        let rect = cx.walk_turtle(walk);

        // Draw background
        self.draw_bg.draw_abs(cx, rect);

        // Draw grid if we have a snapshot
        if let Some(grid) = &self.grid_data {
            let font_size = 13.0_f64;
            let cw = font_size * 0.6; // approximate cell width for monospace
            let ch = font_size * 1.4; // cell height with line spacing

            self.cell_width = cw;
            self.cell_height = ch;

            let origin_x = rect.pos.x + 8.0; // padding
            let origin_y = rect.pos.y + 4.0;

            // Draw each cell
            for row in 0..grid.rows {
                for col in 0..grid.cols {
                    let idx = row * grid.cols + col;
                    if idx >= grid.cells.len() {
                        break;
                    }
                    let cell = &grid.cells[idx];

                    let x = origin_x + (col as f64) * cw;
                    let y = origin_y + (row as f64) * ch;

                    // Draw cell background if not default
                    let bg = cell.bg;
                    if bg[0] > 0.11 || bg[1] > 0.11 || bg[2] > 0.19 {
                        self.draw_bg.color = vec4(bg[0] as f32, bg[1] as f32, bg[2] as f32, bg[3] as f32);
                        self.draw_bg.draw_abs(cx, Rect {
                            pos: dvec2(x, y),
                            size: dvec2(cw, ch),
                        });
                    }

                    // Draw character
                    if cell.c != ' ' && cell.c != '\0' {
                        let s = cell.c.to_string();
                        self.draw_text.color = vec4(
                            cell.fg[0] as f32,
                            cell.fg[1] as f32,
                            cell.fg[2] as f32,
                            cell.fg[3] as f32,
                        );
                        self.draw_text.draw_abs(cx, dvec2(x, y), &s);
                    }
                }
            }

            // Draw cursor
            if grid.cursor_visible
                && grid.cursor_row < grid.rows
                && grid.cursor_col < grid.cols
            {
                let cx_pos = origin_x + (grid.cursor_col as f64) * cw;
                let cy_pos = origin_y + (grid.cursor_row as f64) * ch;

                // Beam cursor (2px wide line)
                self.draw_bg.color = vec4(0.914, 0.271, 0.376, 1.0); // LEUWI_ACCENT
                self.draw_bg.draw_abs(cx, Rect {
                    pos: dvec2(cx_pos, cy_pos),
                    size: dvec2(2.0, ch),
                });
            }
        }

        DrawStep::done()
    }
}

impl TerminalView {
    /// Update the grid snapshot for rendering
    pub fn update_grid(&mut self, grid: &Grid) {
        self.grid_data = Some(GridSnapshot::from_grid(grid));
    }
}

impl TerminalViewRef {
    pub fn update_grid(&self, grid: &Grid) {
        if let Some(mut inner) = self.borrow_mut() {
            inner.update_grid(grid);
        }
    }
}
