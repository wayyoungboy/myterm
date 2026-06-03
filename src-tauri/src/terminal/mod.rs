pub mod pty;

use crate::ssh::SshSession;
use parking_lot::Mutex;
use ssh2::Channel;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use tauri::{AppHandle, Emitter};

pub struct TerminalSession {
    pub id: String,
    pub connection_id: String,
    pub _ssh: SshSession, // Keep the SSH session + TCP stream alive
    pub channel: Arc<Mutex<Channel>>,
    pub running: Arc<AtomicBool>,
}

pub struct TerminalManager {
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

impl TerminalManager {
    pub fn new() -> Self {
        TerminalManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_connection_id(&self, id: &str) -> Option<String> {
        let sessions = self.sessions.lock();
        sessions.get(id).map(|s| s.connection_id.clone())
    }

    pub fn insert(&self, session: TerminalSession) {
        let mut sessions = self.sessions.lock();
        log::info!(
            target: "myterm::terminal",
            "session inserted session_id={} connection_id={}",
            session.id,
            session.connection_id
        );
        sessions.insert(session.id.clone(), session);
    }

    pub fn remove(&self, id: &str) {
        let mut sessions = self.sessions.lock();
        if let Some(session) = sessions.remove(id) {
            log::info!(
                target: "myterm::terminal",
                "session remove start session_id={} connection_id={}",
                session.id,
                session.connection_id
            );
            // Signal the reader thread to stop
            session.running.store(false, Ordering::SeqCst);
            // Close the channel
            let mut channel = session.channel.lock();
            let _ = channel.close();
            let _ = channel.wait_close();
            log::info!(
                target: "myterm::terminal",
                "session removed session_id={} connection_id={}",
                session.id,
                session.connection_id
            );
        } else {
            log::warn!(
                target: "myterm::terminal",
                "session remove requested for missing session_id={}",
                id
            );
        }
    }

    pub fn write_to_channel(&self, id: &str, data: &[u8]) -> Result<(), String> {
        let op_id = uuid::Uuid::new_v4().to_string();
        let started = std::time::Instant::now();
        let (channel, connection_id) = {
            let sessions = self.sessions.lock();
            match sessions.get(id) {
                Some(session) => (session.channel.clone(), session.connection_id.clone()),
                None => {
                    log::warn!(
                        target: "myterm::terminal",
                        "write requested for missing session op_id={} session_id={} bytes={}",
                        op_id,
                        id,
                        data.len()
                    );
                    return Err("Session not found".to_string());
                }
            }
        };

        log::debug!(
            target: "myterm::terminal",
            "write start op_id={} session_id={} connection_id={} bytes={}",
            op_id,
            id,
            connection_id,
            data.len()
        );

        let result = (|| {
            let mut channel = channel.lock();
            channel
                .write_all(data)
                .map_err(|e| format!("Write failed: {}", e))?;
            channel
                .flush()
                .map_err(|e| format!("Flush failed: {}", e))?;
            Ok(())
        })();

        match result {
            Ok(()) => {
                log::debug!(
                    target: "myterm::terminal",
                    "write success op_id={} session_id={} connection_id={} bytes={} elapsed_ms={}",
                    op_id,
                    id,
                    connection_id,
                    data.len(),
                    started.elapsed().as_millis()
                );
                Ok(())
            }
            Err(err) => {
                log::error!(
                    target: "myterm::terminal",
                    "write failed op_id={} session_id={} connection_id={} bytes={} elapsed_ms={} error={}",
                    op_id,
                    id,
                    connection_id,
                    data.len(),
                    started.elapsed().as_millis(),
                    err
                );
                Err(err)
            }
        }
    }

    pub fn resize_channel(&self, id: &str, cols: u32, rows: u32) -> Result<(), String> {
        let op_id = uuid::Uuid::new_v4().to_string();
        let started = std::time::Instant::now();
        let (channel, connection_id) = {
            let sessions = self.sessions.lock();
            match sessions.get(id) {
                Some(session) => (session.channel.clone(), session.connection_id.clone()),
                None => {
                    log::warn!(
                        target: "myterm::terminal",
                        "resize requested for missing session op_id={} session_id={} cols={} rows={}",
                        op_id,
                        id,
                        cols,
                        rows
                    );
                    return Err("Session not found".to_string());
                }
            }
        };

        log::debug!(
            target: "myterm::terminal",
            "resize start op_id={} session_id={} connection_id={} cols={} rows={}",
            op_id,
            id,
            connection_id,
            cols,
            rows
        );

        let result = channel
            .lock()
            .request_pty_size(cols, rows, None, None)
            .map_err(|e| format!("Resize failed: {}", e));

        match result {
            Ok(()) => {
                log::debug!(
                    target: "myterm::terminal",
                    "resize success op_id={} session_id={} connection_id={} cols={} rows={} elapsed_ms={}",
                    op_id,
                    id,
                    connection_id,
                    cols,
                    rows,
                    started.elapsed().as_millis()
                );
                Ok(())
            }
            Err(err) => {
                log::error!(
                    target: "myterm::terminal",
                    "resize failed op_id={} session_id={} connection_id={} cols={} rows={} elapsed_ms={} error={}",
                    op_id,
                    id,
                    connection_id,
                    cols,
                    rows,
                    started.elapsed().as_millis(),
                    err
                );
                Err(err)
            }
        }
    }

    pub fn start_reader(&self, id: &str, app_handle: AppHandle) -> Result<(), String> {
        let sessions = self.sessions.lock();
        if let Some(session) = sessions.get(id) {
            let channel = session.channel.clone();
            let running = session.running.clone();
            let session_id = id.to_string();
            let connection_id = session.connection_id.clone();
            let sessions_for_reader = self.sessions.clone();

            thread::spawn(move || {
                let mut buf = [0u8; 8192];
                while running.load(Ordering::SeqCst) {
                    let mut channel = channel.lock();
                    match channel.read(&mut buf) {
                        Ok(0) => {
                            // Non-blocking 0 can mean no data; check if channel is truly closed
                            if channel.eof() {
                                drop(channel);
                                log::info!(
                                    target: "myterm::terminal",
                                    "reader eof session_id={} connection_id={}",
                                    session_id,
                                    connection_id
                                );
                                running.store(false, Ordering::SeqCst);
                                sessions_for_reader.lock().remove(&session_id);
                                let _ =
                                    app_handle.emit(&format!("terminal-exit-{}", session_id), ());
                                break;
                            }
                            drop(channel);
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        Ok(n) => {
                            let data = buf[..n].to_vec();
                            drop(channel);
                            let _ =
                                app_handle.emit(&format!("terminal-output-{}", session_id), data);
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            drop(channel);
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            continue;
                        }
                        Err(err) => {
                            drop(channel);
                            log::warn!(
                                target: "myterm::terminal",
                                "reader read error session_id={} connection_id={} error={}",
                                session_id,
                                connection_id,
                                err
                            );
                            running.store(false, Ordering::SeqCst);
                            sessions_for_reader.lock().remove(&session_id);
                            let _ = app_handle.emit(&format!("terminal-exit-{}", session_id), ());
                            break;
                        }
                    }
                }
                log::debug!(
                    target: "myterm::terminal",
                    "reader stopped session_id={} connection_id={}",
                    session_id,
                    connection_id
                );
            });

            Ok(())
        } else {
            Err("Session not found".to_string())
        }
    }
}
