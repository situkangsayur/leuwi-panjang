use crate::terminal_pane::TerminalPane;

/// Split direction
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitDir {
    Vertical,
    Horizontal,
}

/// A tab contains one or more panes (via splits)
pub struct Tab {
    pub id: u32,
    pub panes: Vec<TerminalPane>,
    pub active_pane: usize,
    pub split: Option<SplitDir>,
}

static TAB_ID_COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

impl Tab {
    pub fn new(shell: &str, cols: u16, rows: u16, scrollback: usize) -> Self {
        let id = TAB_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let pane = TerminalPane::new(shell, cols, rows, scrollback);
        Self {
            id,
            panes: vec![pane],
            active_pane: 0,
            split: None,
        }
    }

    pub fn title(&self) -> &str {
        &self.panes[self.active_pane].title
    }

    pub fn active_pane_mut(&mut self) -> &mut TerminalPane {
        &mut self.panes[self.active_pane]
    }

    pub fn active_pane(&self) -> &TerminalPane {
        &self.panes[self.active_pane]
    }

    /// Add a split pane
    pub fn split(&mut self, dir: SplitDir, shell: &str, cols: u16, rows: u16, scrollback: usize) {
        if self.panes.len() >= 2 {
            return; // Max 2 panes per tab for now
        }
        let pane_cols = if dir == SplitDir::Vertical { cols / 2 } else { cols };
        let pane_rows = if dir == SplitDir::Horizontal { rows / 2 } else { rows };
        let pane = TerminalPane::new(shell, pane_cols, pane_rows, scrollback);
        self.panes.push(pane);
        self.active_pane = 1;
        self.split = Some(dir);
    }

    /// Close the split (remove second pane)
    pub fn close_split(&mut self) {
        if self.panes.len() > 1 {
            self.panes.pop();
            self.active_pane = 0;
            self.split = None;
        }
    }

    /// Toggle active pane
    pub fn toggle_pane(&mut self) {
        if self.panes.len() > 1 {
            self.active_pane = if self.active_pane == 0 { 1 } else { 0 };
        }
    }

    pub fn is_split(&self) -> bool {
        self.split.is_some()
    }

    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }
}

/// Manages all tabs
pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub shell: String,
    pub cols: u16,
    pub rows: u16,
    pub scrollback: usize,
}

impl TabManager {
    pub fn new(shell: &str, cols: u16, rows: u16, scrollback: usize) -> Self {
        let tab = Tab::new(shell, cols, rows, scrollback);
        Self {
            tabs: vec![tab],
            active_tab: 0,
            shell: shell.to_string(),
            cols,
            rows,
            scrollback,
        }
    }

    pub fn new_tab(&mut self) {
        let tab = Tab::new(&self.shell, self.cols, self.rows, self.scrollback);
        self.tabs.push(tab);
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, idx: usize) {
        if self.tabs.len() <= 1 {
            return; // Don't close last tab
        }
        self.tabs.remove(idx);
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
    }

    pub fn close_active_tab(&mut self) {
        self.close_tab(self.active_tab);
    }

    pub fn switch_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active_tab = idx;
        }
    }

    pub fn next_tab(&mut self) {
        self.active_tab = (self.active_tab + 1) % self.tabs.len();
    }

    pub fn prev_tab(&mut self) {
        if self.active_tab == 0 {
            self.active_tab = self.tabs.len() - 1;
        } else {
            self.active_tab -= 1;
        }
    }

    pub fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    pub fn split_active(&mut self, dir: SplitDir) {
        let shell = self.shell.clone();
        let cols = self.cols;
        let rows = self.rows;
        let scrollback = self.scrollback;
        let tab = self.active_tab_mut();
        tab.split(dir, &shell, cols, rows, scrollback);
    }

    /// Write to the active pane of the active tab
    pub fn write_active(&mut self, data: &[u8]) {
        self.active_tab_mut().active_pane_mut().write(data);
    }

    /// Resize all panes
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        for tab in &mut self.tabs {
            for pane in &mut tab.panes {
                pane.resize(cols, rows);
            }
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn tab_titles(&self) -> Vec<String> {
        self.tabs.iter().enumerate().map(|(i, _)| {
            format!("Terminal {}", i + 1)
        }).collect()
    }
}
