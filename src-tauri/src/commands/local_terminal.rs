use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use std::io::{Read, Write};
use std::process::{Command, Stdio, Child};
use std::thread;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub struct LocalTerminalSession {
    pub id: String,
    pub child: Child,
}

pub struct LocalTerminalManager {
    sessions: Arc<Mutex<HashMap<String, LocalTerminalSession>>>,
}

impl LocalTerminalManager {
    pub fn new() -> Self {
        LocalTerminalManager {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tauri::command]
pub fn open_local_terminal(
    ltm: State<'_, LocalTerminalManager>,
    app_handle: AppHandle,
    shell: Option<String>,
) -> Result<String, String> {
    let shell = shell.unwrap_or_else(|| {
        if cfg!(target_os = "windows") {
            "cmd".to_string()
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        }
    });

    let mut child = Command::new(&shell)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn shell: {}", e))?;

    let session_id = Uuid::new_v4().to_string();

    // Take stdout and stderr for reading
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    let mut stdin = child.stdin.take().ok_or("Failed to capture stdin")?;

    // Spawn stdout reader thread
    let sid = session_id.clone();
    let app = app_handle.clone();
    thread::spawn(move || {
        let mut reader = stdout;
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = app.emit(&format!("local-exit-{}", sid), ());
                    break;
                }
                Ok(n) => {
                    let _ = app.emit(&format!("local-output-{}", sid), buf[..n].to_vec());
                }
                Err(_) => {
                    let _ = app.emit(&format!("local-exit-{}", sid), ());
                    break;
                }
            }
        }
    });

    // Spawn stderr reader thread
    let sid = session_id.clone();
    let app = app_handle.clone();
    thread::spawn(move || {
        let mut reader = stderr;
        let mut buf = [0u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let _ = app.emit(&format!("local-output-{}", sid), buf[..n].to_vec());
                }
            }
        }
    });

    ltm.sessions.lock().insert(session_id.clone(), LocalTerminalSession {
        id: session_id.clone(),
        child,
    });

    Ok(session_id)
}

#[tauri::command]
pub fn local_terminal_write(
    ltm: State<'_, LocalTerminalManager>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    let mut sessions = ltm.sessions.lock();
    if let Some(session) = sessions.get_mut(&session_id) {
        if let Some(ref mut stdin) = session.child.stdin {
            stdin.write_all(data.as_bytes())
                .map_err(|e| format!("Write failed: {}", e))?;
            stdin.flush()
                .map_err(|e| format!("Flush failed: {}", e))?;
            Ok(())
        } else {
            Err("stdin not available".to_string())
        }
    } else {
        Err("Session not found".to_string())
    }
}

#[tauri::command]
pub fn close_local_terminal(
    ltm: State<'_, LocalTerminalManager>,
    session_id: String,
) -> Result<(), String> {
    let mut sessions = ltm.sessions.lock();
    if let Some(mut session) = sessions.remove(&session_id) {
        let _ = session.child.kill();
        let _ = session.child.wait();
    }
    Ok(())
}
