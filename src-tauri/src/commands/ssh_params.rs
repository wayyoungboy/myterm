use crate::crypto::{decrypt_password, get_master_password};
use crate::db::DbConn;
use crate::ssh::connection::{connect, SshConnectParams};
use crate::ssh::SshSession;
use crate::terminal::TerminalManager;
use std::collections::HashSet;

pub fn load_ssh_params(db: &DbConn, connection_id: &str) -> Result<SshConnectParams, String> {
    let mut seen = HashSet::new();
    load_ssh_params_inner(db, connection_id, &mut seen)
}

fn load_ssh_params_inner(
    db: &DbConn,
    connection_id: &str,
    seen: &mut HashSet<String>,
) -> Result<SshConnectParams, String> {
    if !seen.insert(connection_id.to_string()) {
        return Err(format!(
            "ProxyJump cycle detected at connection {}",
            connection_id
        ));
    }

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

    let proxy_jump = match proxy_jump_id.as_deref().filter(|id| !id.trim().is_empty()) {
        Some(jump_id) => Some(Box::new(load_ssh_params_inner(db, jump_id, seen)?)),
        None => None,
    };
    seen.remove(connection_id);

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
        proxy_jump,
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

    #[test]
    fn loads_nested_proxy_jump_params() {
        let conn = Connection::open_in_memory().expect("db");
        init_db(&conn).expect("schema");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["jump-1", "Jump", "jump.example", 2222, "key", "jumper"],
        )
        .expect("insert jump");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username, proxy_jump_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                "target-1",
                "Target",
                "target.example",
                22,
                "key",
                "tester",
                "jump-1"
            ],
        )
        .expect("insert target");
        let db = DbConn(Mutex::new(conn));

        let params = load_ssh_params(&db, "target-1").expect("params");
        let jump = params.proxy_jump.as_ref().expect("jump params");

        assert_eq!(params.proxy_jump_id.as_deref(), Some("jump-1"));
        assert_eq!(jump.host, "jump.example");
        assert_eq!(jump.port, 2222);
        assert_eq!(jump.username, "jumper");
    }

    #[test]
    fn rejects_proxy_jump_cycles() {
        let conn = Connection::open_in_memory().expect("db");
        init_db(&conn).expect("schema");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["a", "A", "a.example", 22, "key", "auser"],
        )
        .expect("insert a");
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params!["b", "B", "b.example", 22, "key", "buser"],
        )
        .expect("insert b");
        conn.execute(
            "UPDATE connections SET proxy_jump_id = ?1 WHERE id = ?2",
            rusqlite::params!["b", "a"],
        )
        .expect("link a");
        conn.execute(
            "UPDATE connections SET proxy_jump_id = ?1 WHERE id = ?2",
            rusqlite::params!["a", "b"],
        )
        .expect("link b");
        let db = DbConn(Mutex::new(conn));

        let err = load_ssh_params(&db, "a").expect_err("cycle must fail");

        assert!(err.contains("ProxyJump cycle"));
    }
}
