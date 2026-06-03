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
        proxy_type,
        proxy_host,
        proxy_port,
        proxy_jump_id,
        timeout_ms,
        init_command,
        init_path,
        heartbeat_ms,
    ) = {
        let conn_guard = db.0.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn_guard
            .prepare(
                "SELECT host, port, auth_type, username, password_enc, key_path, proxy_type, proxy_host, proxy_port, proxy_jump_id, timeout_ms, init_command, init_path, heartbeat_ms FROM connections WHERE id = ?1",
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
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
                row.get::<_, Option<i32>>(8)?,
                row.get::<_, Option<String>>(9)?,
                row.get::<_, Option<i32>>(10)?,
                row.get::<_, Option<String>>(11)?,
                row.get::<_, Option<String>>(12)?,
                row.get::<_, Option<i32>>(13)?,
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
        proxy_type,
        proxy_host,
        proxy_port: proxy_port.and_then(|port| u16::try_from(port).ok()),
        proxy_jump_id,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::init_db;
    use rusqlite::Connection;
    use std::sync::Mutex;

    #[test]
    fn loads_proxy_fields_for_stored_connection() {
        let conn = Connection::open_in_memory().expect("db");
        init_db(&conn).expect("schema");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["jump-1", "Jump", "jump.example", 22, "key", "jumper"],
        )
        .expect("insert jump");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username, proxy_type, proxy_host, proxy_port, proxy_jump_id, heartbeat_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                "conn-1",
                "Proxy Test",
                "target.example",
                22,
                "key",
                "tester",
                "socks5",
                "127.0.0.1",
                1080,
                "jump-1",
                7000
            ],
        )
        .expect("insert");
        let db = DbConn(Mutex::new(conn));

        let params = load_ssh_params(&db, "conn-1").expect("params");

        assert_eq!(params.proxy_type.as_deref(), Some("socks5"));
        assert_eq!(params.proxy_host.as_deref(), Some("127.0.0.1"));
        assert_eq!(params.proxy_port, Some(1080));
        assert_eq!(params.proxy_jump_id.as_deref(), Some("jump-1"));
        assert_eq!(params.heartbeat_ms, Some(7000));
    }
}
