use crate::crypto::{decrypt_password, get_master_password};
use crate::db::DbConn;
use crate::ssh::connection::{connect, SshConnectParams};
use crate::ssh::SshSession;
use crate::terminal::TerminalManager;

pub fn load_ssh_params(db: &DbConn, connection_id: &str) -> Result<SshConnectParams, String> {
    let (
        host,
        port,
        auth_type,
        username,
        password_enc,
        key_path,
        timeout_ms,
        init_command,
        init_path,
        heartbeat_ms,
    ) = {
        let conn_guard = db.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn_guard
            .prepare(
                "SELECT host, port, auth_type, username, password_enc, key_path, timeout_ms, init_command, init_path, heartbeat_ms FROM connections WHERE id = ?1",
            )
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

    let password = password_enc
        .as_deref()
        .filter(|enc| !enc.is_empty())
        .and_then(|enc| decrypt_password(enc, &get_master_password()).ok());

    let params = SshConnectParams {
        host,
        port: port as u16,
        username,
        auth_type,
        password,
        key_path,
        timeout_ms: timeout_ms.map(|t| t as u32),
        proxy_jump_id: None,
        init_command,
        init_path,
        heartbeat_ms,
    };

    log::debug!(
        target: "myterm::ssh",
        "loaded ssh params connection_id={} host={} port={} username={} auth_type={}",
        connection_id,
        params.host,
        params.port,
        params.username,
        params.auth_type
    );

    Ok(params)
}

pub fn connect_for_terminal_session(
    db: &DbConn,
    tm: &TerminalManager,
    terminal_session_id: &str,
) -> Result<SshSession, String> {
    let connection_id = tm
        .get_connection_id(terminal_session_id)
        .ok_or("Session not found")?;
    let params = load_ssh_params(db, &connection_id)?;
    log::info!(
        target: "myterm::ssh",
        "opening auxiliary ssh session terminal_session_id={} connection_id={} host={} port={} purpose=ssh_subsystem",
        terminal_session_id,
        connection_id,
        params.host,
        params.port
    );
    let ssh = connect(&params).map_err(|err| {
        log::error!(
            target: "myterm::ssh",
            "auxiliary ssh session failed terminal_session_id={} connection_id={} error={}",
            terminal_session_id,
            connection_id,
            err
        );
        err
    })?;
    ssh.session.set_blocking(true);
    log::info!(
        target: "myterm::ssh",
        "auxiliary ssh session opened terminal_session_id={} connection_id={}",
        terminal_session_id,
        connection_id
    );
    Ok(ssh)
}
