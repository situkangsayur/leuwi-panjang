use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::grid::Grid;
use crate::parser::Parser;

/// A terminal session: manages a grid, parser, and PTY reader thread.
pub struct TerminalSession {
    pub grid: Arc<Mutex<Grid>>,
    parser: Arc<Mutex<Parser>>,
    _reader_handle: Option<thread::JoinHandle<()>>,
    /// Flag to signal the reader thread to stop
    running: Arc<Mutex<bool>>,
}

impl TerminalSession {
    /// Create a new terminal session with the given grid size.
    pub fn new(cols: usize, rows: usize, scrollback: usize) -> Self {
        Self {
            grid: Arc::new(Mutex::new(Grid::new(cols, rows, scrollback))),
            parser: Arc::new(Mutex::new(Parser::new())),
            _reader_handle: None,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start reading from a PTY reader in a background thread.
    /// The reader should be the PTY's reader (clone of master reader).
    pub fn start_reader(&mut self, reader: Arc<Mutex<Box<dyn Read + Send>>>) {
        let grid = Arc::clone(&self.grid);
        let parser = Arc::clone(&self.parser);
        let running = Arc::clone(&self.running);

        *running.lock().unwrap() = true;

        let handle = thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                {
                    let is_running = running.lock().unwrap();
                    if !*is_running {
                        break;
                    }
                }

                let n = {
                    let mut reader = match reader.lock() {
                        Ok(r) => r,
                        Err(_) => break,
                    };
                    match reader.read(&mut buf) {
                        Ok(0) => break, // EOF — shell exited
                        Ok(n) => n,
                        Err(_) => break,
                    }
                };

                // Parse the data and update the grid
                let mut parser = parser.lock().unwrap();
                let mut grid = grid.lock().unwrap();
                parser.process(&buf[..n], &mut grid);
            }
        });

        self._reader_handle = Some(handle);
    }

    /// Process raw bytes directly (for testing or non-PTY input).
    pub fn process_bytes(&self, data: &[u8]) {
        let mut parser = self.parser.lock().unwrap();
        let mut grid = self.grid.lock().unwrap();
        parser.process(data, &mut grid);
    }

    /// Stop the reader thread.
    pub fn stop(&self) {
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
    }

    /// Get a clone of the grid Arc for rendering.
    pub fn grid_ref(&self) -> Arc<Mutex<Grid>> {
        Arc::clone(&self.grid)
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        self.stop();
    }
}
