use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub struct Pty {
    master: Box<dyn MasterPty + Send>,
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Box<dyn Write + Send>,
    size: PtySize,
}

impl Pty {
    /// Spawn a new PTY with the given shell and size.
    pub fn spawn(shell: &str, cols: u16, rows: u16) -> anyhow::Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(size)?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        cmd.env("LEUWI_PANJANG", "1");
        cmd.env("LEUWI_PANJANG_VERSION", "0.1.0");

        // Propagate important env vars from parent
        for var in &["HOME", "USER", "PATH", "LANG", "LC_ALL", "XDG_RUNTIME_DIR",
                     "DISPLAY", "WAYLAND_DISPLAY", "DBUS_SESSION_BUS_ADDRESS",
                     "SSH_AUTH_SOCK", "GPG_AGENT_INFO", "EDITOR", "VISUAL"] {
            if let Ok(val) = std::env::var(var) {
                cmd.env(var, &val);
            }
        }

        pair.slave.spawn_command(cmd)?;
        drop(pair.slave);

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        Ok(Self {
            master: pair.master,
            reader: Arc::new(Mutex::new(reader)),
            writer,
            size,
        })
    }

    /// Read available data from the PTY.
    /// Returns the number of bytes read, or 0 if no data available.
    pub fn read(&self, buf: &mut [u8]) -> anyhow::Result<usize> {
        let mut reader = self.reader.lock().map_err(|e| anyhow::anyhow!("Lock error: {e}"))?;
        let n = reader.read(buf)?;
        Ok(n)
    }

    /// Write data to the PTY (keyboard input from user).
    pub fn write(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&mut self, cols: u16, rows: u16) -> anyhow::Result<()> {
        self.size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.master.resize(self.size)?;
        Ok(())
    }

    /// Get a clone of the reader for use in a separate thread.
    pub fn clone_reader(&self) -> Arc<Mutex<Box<dyn Read + Send>>> {
        Arc::clone(&self.reader)
    }

    pub fn cols(&self) -> u16 {
        self.size.cols
    }

    pub fn rows(&self) -> u16 {
        self.size.rows
    }
}
