use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum PtyError {
    #[error("cwd does not exist: {0}")]
    CwdNotFound(String),
    #[error("program not found: {0}")]
    ProgramNotFound(String),
    #[error("failed to spawn PTY: {0}")]
    SpawnFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub enum PtyExitStatus {
    Exited(Option<i32>),
    Killed,
}

pub struct PtyHost {
    master: Arc<Mutex<Box<dyn MasterPty + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    child_pid: Option<u32>,
    _output_handle: tokio::task::JoinHandle<()>,
    _exit_handle: tokio::task::JoinHandle<()>,
}

impl PtyHost {
    pub fn spawn(
        command: &str,
        args: &[String],
        cwd: &str,
        cols: u16,
        rows: u16,
        output_tx: mpsc::UnboundedSender<Vec<u8>>,
        exit_tx: mpsc::UnboundedSender<PtyExitStatus>,
    ) -> Result<Self, PtyError> {
        // Validate cwd
        let cwd_path = Path::new(cwd);
        if !cwd_path.exists() {
            return Err(PtyError::CwdNotFound(cwd.to_string()));
        }

        // Validate program exists
        if which::which(command).is_err() && !Path::new(command).exists() {
            return Err(PtyError::ProgramNotFound(command.to_string()));
        }

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let mut cmd = CommandBuilder::new(command);
        cmd.args(args);
        cmd.cwd(cwd_path);

        // Set essential environment variables for proper terminal behavior
        // This is critical in .app bundles where the environment may be minimal
        cmd.env("TERM", "xterm-256color");

        // Set locale to UTF-8 to ensure proper character encoding
        // Without this, characters may render as broken control sequences
        if std::env::var("LANG").is_err() {
            cmd.env("LANG", "en_US.UTF-8");
        }
        if std::env::var("LC_ALL").is_err() {
            cmd.env("LC_ALL", "en_US.UTF-8");
        }

        let mut child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let child_pid = child.process_id();

        // Drop slave — we only need the master side
        drop(pair.slave);

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;

        // Async read loop
        let output_handle = tokio::task::spawn_blocking(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if output_tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Exit status monitor
        let exit_handle = tokio::task::spawn_blocking(move || {
            let status = match child.wait() {
                Ok(exit) => {
                    if exit.success() {
                        PtyExitStatus::Exited(Some(0))
                    } else {
                        // portable-pty doesn't expose raw exit code directly on all platforms
                        PtyExitStatus::Exited(None)
                    }
                }
                Err(_) => PtyExitStatus::Killed,
            };
            let _ = exit_tx.send(status);
        });

        Ok(Self {
            master: Arc::new(Mutex::new(pair.master)),
            writer: Arc::new(Mutex::new(writer)),
            child_pid,
            _output_handle: output_handle,
            _exit_handle: exit_handle,
        })
    }

    pub fn write(&self, data: &[u8]) -> Result<(), PtyError> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(data)?;
        Ok(())
    }

    pub fn resize(&self, cols: u16, rows: u16) -> Result<(), PtyError> {
        let master = self.master.lock().unwrap();
        master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| PtyError::SpawnFailed(e.to_string()))?;
        Ok(())
    }

    pub fn kill(&self) -> Result<(), PtyError> {
        self.send_signal(libc::SIGKILL)
    }

    pub async fn graceful_close(&self, timeout_ms: u64) -> Result<(), PtyError> {
        // Send SIGHUP first
        if self.send_signal(libc::SIGHUP).is_ok() {
            tokio::time::sleep(std::time::Duration::from_millis(timeout_ms)).await;
        }
        // Force kill if still alive
        let _ = self.kill();
        Ok(())
    }

    fn send_signal(&self, signal: i32) -> Result<(), PtyError> {
        if let Some(pid) = self.child_pid {
            unsafe {
                libc::kill(pid as i32, signal);
            }
        }
        Ok(())
    }
}
