use crate::cell::Cell;

/// The terminal screen grid — a 2D array of cells.
pub struct Grid {
    /// Number of columns
    cols: usize,
    /// Number of rows (visible area)
    rows: usize,
    /// Cell storage (row-major: cells[row * cols + col])
    cells: Vec<Cell>,
    /// Cursor position
    pub cursor_row: usize,
    pub cursor_col: usize,
    /// Scrollback buffer (ring buffer of rows)
    scrollback: Vec<Vec<Cell>>,
    /// Maximum scrollback lines
    max_scrollback: usize,
    /// Selection start (row, col) — None if no selection
    pub selection_start: Option<(usize, usize)>,
    /// Selection end (row, col)
    pub selection_end: Option<(usize, usize)>,
}

impl Grid {
    pub fn new(cols: usize, rows: usize, max_scrollback: usize) -> Self {
        let cells = vec![Cell::blank(); cols * rows];
        Self {
            cols,
            rows,
            cells,
            cursor_row: 0,
            cursor_col: 0,
            scrollback: Vec::new(),
            max_scrollback,
            selection_start: None,
            selection_end: None,
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Get a cell reference at (row, col)
    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[row * self.cols + col]
    }

    /// Get a mutable cell reference at (row, col)
    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.cells[row * self.cols + col]
    }

    /// Write a character at the cursor position and advance cursor
    pub fn write_char(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.newline();
        }
        let cell = self.cell_mut(self.cursor_row, self.cursor_col);
        cell.c = c;
        cell.dirty = true;
        self.cursor_col += 1;
    }

    /// Move cursor to next line, scrolling if needed
    pub fn newline(&mut self) {
        self.cursor_col = 0;
        if self.cursor_row + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_row += 1;
        }
    }

    /// Scroll the grid up by one line
    fn scroll_up(&mut self) {
        // Save top row to scrollback
        let top_row: Vec<Cell> = (0..self.cols)
            .map(|col| self.cells[col].clone())
            .collect();

        self.scrollback.push(top_row);
        if self.scrollback.len() > self.max_scrollback {
            self.scrollback.remove(0);
        }

        // Shift all rows up
        for row in 0..self.rows - 1 {
            for col in 0..self.cols {
                let src = (row + 1) * self.cols + col;
                let dst = row * self.cols + col;
                self.cells[dst] = self.cells[src].clone();
            }
        }

        // Clear bottom row
        let last_row = self.rows - 1;
        for col in 0..self.cols {
            self.cells[last_row * self.cols + col] = Cell::blank();
        }
    }

    /// Clear the entire visible grid
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            *cell = Cell::blank();
        }
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    /// Resize the grid
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) {
        let mut new_cells = vec![Cell::blank(); new_cols * new_rows];

        let copy_rows = self.rows.min(new_rows);
        let copy_cols = self.cols.min(new_cols);
        for row in 0..copy_rows {
            for col in 0..copy_cols {
                new_cells[row * new_cols + col] = self.cells[row * self.cols + col].clone();
            }
        }

        self.cells = new_cells;
        self.cols = new_cols;
        self.rows = new_rows;
        self.cursor_row = self.cursor_row.min(new_rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(new_cols.saturating_sub(1));
    }

    /// Mark all cells as clean (after rendering)
    pub fn mark_clean(&mut self) {
        for cell in &mut self.cells {
            cell.dirty = false;
        }
    }

    /// Get scrollback line count
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// Start a text selection at (row, col)
    pub fn start_selection(&mut self, row: usize, col: usize) {
        self.selection_start = Some((row.min(self.rows - 1), col.min(self.cols - 1)));
        self.selection_end = None;
    }

    /// Update selection end point
    pub fn update_selection(&mut self, row: usize, col: usize) {
        self.selection_end = Some((row.min(self.rows - 1), col.min(self.cols - 1)));
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
        self.selection_end = None;
    }

    /// Check if a cell is within the current selection
    pub fn is_selected(&self, row: usize, col: usize) -> bool {
        let (start, end) = match (self.selection_start, self.selection_end) {
            (Some(s), Some(e)) => {
                // Normalize: start should be before end
                if s.0 < e.0 || (s.0 == e.0 && s.1 <= e.1) {
                    (s, e)
                } else {
                    (e, s)
                }
            }
            _ => return false,
        };

        if row < start.0 || row > end.0 {
            return false;
        }
        if row == start.0 && row == end.0 {
            return col >= start.1 && col <= end.1;
        }
        if row == start.0 {
            return col >= start.1;
        }
        if row == end.0 {
            return col <= end.1;
        }
        true // middle rows are fully selected
    }

    /// Get selected text as a string
    pub fn get_selected_text(&self) -> Option<String> {
        let (start, end) = match (self.selection_start, self.selection_end) {
            (Some(s), Some(e)) => {
                if s.0 < e.0 || (s.0 == e.0 && s.1 <= e.1) {
                    (s, e)
                } else {
                    (e, s)
                }
            }
            _ => return None,
        };

        let mut text = String::new();
        for row in start.0..=end.0 {
            let col_start = if row == start.0 { start.1 } else { 0 };
            let col_end = if row == end.0 { end.1 } else { self.cols - 1 };

            for col in col_start..=col_end.min(self.cols - 1) {
                let c = self.cell(row, col).c;
                text.push(if c == '\0' { ' ' } else { c });
            }

            // Trim trailing spaces on each line
            let trimmed_len = text.trim_end_matches(' ').len();
            text.truncate(trimmed_len);

            if row < end.0 {
                text.push('\n');
            }
        }

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    /// Select all visible text
    pub fn select_all(&mut self) {
        self.selection_start = Some((0, 0));
        self.selection_end = Some((self.rows - 1, self.cols - 1));
    }

    /// Get all visible text (for Ctrl+Shift+C without selection = copy all)
    pub fn get_all_text(&self) -> String {
        let mut text = String::with_capacity(self.cols * self.rows);
        for row in 0..self.rows {
            for col in 0..self.cols {
                let c = self.cell(row, col).c;
                text.push(if c == '\0' { ' ' } else { c });
            }
            let trimmed_len = text.trim_end_matches(' ').len();
            text.truncate(trimmed_len);
            if row < self.rows - 1 {
                text.push('\n');
            }
        }
        // Remove trailing empty lines
        text.trim_end().to_string()
    }
}
