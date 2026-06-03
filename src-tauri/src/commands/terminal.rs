use crate::terminal::{TerminalManager, TerminalSession};
use crate::terminal::pty::open_shell;
use crate::ssh::connection::{connect, SshConnectParams};
use crate::db::DbConn;
use crate::crypto::decrypt_password;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use parking_lot::Mutex;
use tauri::{State, AppHandle};

#[tauri::command]
pub fn connect_terminal(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    app_handle: AppHandle,
    connection_id: String,
) -> Result<String, String> {
    let (host, port, auth_type, username, password_enc, key_path, timeout_ms, init_command, init_path, heartbeat_ms) = {
        let conn_guard = db.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn_guard
            .prepare("SELECT host, port, auth_type, username, password_enc, key_path, timeout_ms, init_command, init_path, heartbeat_ms FROM connections WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        stmt.query_row(rusqlite::params![connection_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i32>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<String>>(8)?,
                row.get::<_, Option<i32>>(9)?,
            ))
        })
        .map_err(|e| format!("Connection not found: {}", e))?
    };

    // Decrypt password
    let password = if let Some(ref enc) = password_enc {
        if !enc.is_empty() {
            let master = crate::crypto::get_master_password();
            decrypt_password(enc, &master).ok()
        } else {
            None
        }
    } else {
        None
    };

    let params = SshConnectParams {
        host,
        port: port as u16,
        username,
        auth_type,
        password,
        key_path,
        timeout_ms: timeout_ms.map(|t| t as u32),
        proxy_jump_id: None,
        init_command: init_command.clone(),
        init_path: init_path.clone(),
        heartbeat_ms,
    };

    let ssh_session = connect(&params)?;
    let session_ref = ssh_session.session.clone();
    let mut channel = open_shell(&session_ref)?;
    let session_id = uuid::Uuid::new_v4().to_string();

    // Send init command if configured
    if let Some(ref cmd) = init_command {
        if !cmd.is_empty() {
            use std::io::Write;
            channel.write_all(cmd.as_bytes()).ok();
            channel.write_all(b"\n").ok();
            channel.flush().ok();
        }
    }

    // Send init path if configured
    if let Some(ref path) = init_path {
        if !path.is_empty() {
            use std::io::Write;
            channel.write_all(format!("cd {}\n", path).as_bytes()).ok();
            channel.flush().ok();
        }
    }

    tm.insert(TerminalSession {
        id: session_id.clone(),
        connection_id,
        _ssh: ssh_session,       // Keep TCP stream alive
        session: session_ref,
        channel: Arc::new(Mutex::new(channel)),
        running: Arc::new(AtomicBool::new(true)),
    });

    // Start the background reader thread
    tm.start_reader(&session_id, app_handle)?;

    Ok(session_id)
}

#[tauri::command]
pub fn connect_terminal_for_sftp(
    db: State<'_, DbConn>,
    connection_id: String,
) -> Result<String, String> {
    // Create a separate SSH session for SFTP operations (blocking mode)
    let (host, port, auth_type, username, password_enc, key_path, timeout_ms) = {
        let conn_guard = db.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn_guard
            .prepare("SELECT host, port, auth_type, username, password_enc, key_path, timeout_ms FROM connections WHERE id = ?1")
            .map_err(|e| e.to_string())?;

        stmt.query_row(rusqlite::params![connection_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, Option<i32>>(6)?,
            ))
        })
        .map_err(|e| format!("Connection not found: {}", e))?
    };

    let password = if let Some(ref enc) = password_enc {
        if !enc.is_empty() {
            let master = crate::crypto::get_master_password();
            decrypt_password(enc, &master).ok()
        } else {
            None
        }
    } else {
        None
    };

    let params = SshConnectParams {
        host,
        port: port as u16,
        username,
        auth_type,
        password,
        key_path,
        timeout_ms: timeout_ms.map(|t| t as u32),
        proxy_jump_id: None,
        init_command: None,
        init_path: None,
        heartbeat_ms: None,
    };

    let ssh_session = connect(&params)?;
    let session_id = uuid::Uuid::new_v4().to_string();

    // Store this session for SFTP use (blocking mode)
    // We'll use a separate manager or store it differently
    Ok(session_id)
}

#[tauri::command]
pub fn disconnect_terminal(
    tm: State<'_, TerminalManager>,
    session_id: String,
) -> Result<(), String> {
    tm.remove(&session_id);
    Ok(())
}

#[tauri::command]
pub fn terminal_write(
    tm: State<'_, TerminalManager>,
    session_id: String,
    data: String,
) -> Result<(), String> {
    tm.write_to_channel(&session_id, data.as_bytes())
}

#[tauri::command]
pub fn terminal_resize(
    tm: State<'_, TerminalManager>,
    session_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    tm.resize_channel(&session_id, cols, rows)
}
