use crate::commands::ssh_params::load_ssh_params;
use crate::db::DbConn;
use crate::ssh::connection::connect;
use crate::terminal::pty::open_shell;
use crate::terminal::{TerminalManager, TerminalSession};
use parking_lot::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn connect_terminal(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    app_handle: AppHandle,
    connection_id: String,
) -> Result<String, String> {
    let op_id = uuid::Uuid::new_v4().to_string();
    let started = std::time::Instant::now();
    log::info!(
        target: "myterm::terminal",
        "connect start op_id={} connection_id={}",
        op_id,
        connection_id
    );

    let result = (|| {
        let params = load_ssh_params(&db, &connection_id)?;
        log::info!(
            target: "myterm::terminal",
            "ssh connect start op_id={} connection_id={} host={} port={} username={} auth_type={}",
            op_id,
            connection_id,
            params.host,
            params.port,
            params.username,
            params.auth_type
        );
        let ssh_session = connect(&params)?;
        let session_ref = ssh_session.session.clone();
        let mut channel = open_shell(&session_ref)?;
        let session_id = uuid::Uuid::new_v4().to_string();

        // Send init command if configured. Do not log command content.
        if let Some(ref cmd) = params.init_command {
            if !cmd.is_empty() {
                use std::io::Write;
                channel.write_all(cmd.as_bytes()).ok();
                channel.write_all(b"\n").ok();
                channel.flush().ok();
                log::info!(
                    target: "myterm::terminal",
                    "init command sent op_id={} session_id={}",
                    op_id,
                    session_id
                );
            }
        }

        // Send init path if configured.
        if let Some(ref path) = params.init_path {
            if !path.is_empty() {
                use std::io::Write;
                channel.write_all(format!("cd {}\n", path).as_bytes()).ok();
                channel.flush().ok();
                log::info!(
                    target: "myterm::terminal",
                    "init path sent op_id={} session_id={} path={}",
                    op_id,
                    session_id,
                    path
                );
            }
        }

        tm.insert(TerminalSession {
            id: session_id.clone(),
            connection_id: connection_id.clone(),
            _ssh: ssh_session, // Keep TCP stream alive
            channel: Arc::new(Mutex::new(channel)),
            running: Arc::new(AtomicBool::new(true)),
        });

        // Start the background reader thread
        tm.start_reader(&session_id, app_handle)?;

        Ok(session_id)
    })();

    match result {
        Ok(session_id) => {
            log::info!(
                target: "myterm::terminal",
                "connect success op_id={} connection_id={} session_id={} elapsed_ms={}",
                op_id,
                connection_id,
                session_id,
                started.elapsed().as_millis()
            );
            Ok(session_id)
        }
        Err(err) => {
            log::error!(
                target: "myterm::terminal",
                "connect failed op_id={} connection_id={} elapsed_ms={} error={}",
                op_id,
                connection_id,
                started.elapsed().as_millis(),
                err
            );
            Err(err)
        }
    }
}

#[tauri::command]
pub fn disconnect_terminal(
    tm: State<'_, TerminalManager>,
    session_id: String,
) -> Result<(), String> {
    log::info!(
        target: "myterm::terminal",
        "disconnect session_id={}",
        session_id
    );
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
