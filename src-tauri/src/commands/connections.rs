use crate::crypto::{encrypt_password, get_master_password};
use crate::db::models::{Connection, ConnectionInput, Group};
use crate::db::DbConn;
use crate::ssh::connection::{connect, SshConnectParams};
use rusqlite::OptionalExtension;
use tauri::State;
use uuid::Uuid;

use super::ssh_params::load_ssh_params;

// Response type that excludes sensitive fields
#[derive(Debug, serde::Serialize, Clone)]
pub struct ConnectionResponse {
    pub id: String,
    pub group_id: Option<String>,
    pub name: String,
    pub host: String,
    pub port: i32,
    pub auth_type: String,
    pub username: Option<String>,
    pub has_password: bool,
    pub key_path: Option<String>,
    pub proxy_type: Option<String>,
    pub proxy_host: Option<String>,
    pub proxy_port: Option<i32>,
    pub proxy_jump_id: Option<String>,
    pub init_command: Option<String>,
    pub init_path: Option<String>,
    pub timeout_ms: Option<i32>,
    pub heartbeat_ms: Option<i32>,
    pub remark: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn to_response(conn: &Connection) -> ConnectionResponse {
    ConnectionResponse {
        id: conn.id.clone(),
        group_id: conn.group_id.clone(),
        name: conn.name.clone(),
        host: conn.host.clone(),
        port: conn.port,
        auth_type: conn.auth_type.clone(),
        username: conn.username.clone(),
        has_password: conn.password_enc.is_some()
            && !conn.password_enc.as_deref().unwrap_or("").is_empty(),
        key_path: conn.key_path.clone(),
        proxy_type: conn.proxy_type.clone(),
        proxy_host: conn.proxy_host.clone(),
        proxy_port: conn.proxy_port,
        proxy_jump_id: conn.proxy_jump_id.clone(),
        init_command: conn.init_command.clone(),
        init_path: conn.init_path.clone(),
        timeout_ms: conn.timeout_ms,
        heartbeat_ms: conn.heartbeat_ms,
        remark: conn.remark.clone(),
        created_at: conn.created_at.clone(),
        updated_at: conn.updated_at.clone(),
    }
}

fn auth_type_keeps_password(auth_type: &str) -> bool {
    matches!(auth_type, "password" | "interactive" | "ask")
}

fn current_password_enc(
    conn_guard: &rusqlite::Connection,
    id: &str,
) -> Result<Option<String>, String> {
    conn_guard
        .query_row(
            "SELECT password_enc FROM connections WHERE id = ?1",
            rusqlite::params![id],
            |row| row.get(0),
        )
        .optional()
        .map(|value| value.flatten())
        .map_err(|e| e.to_string())
}

fn encrypted_password_for_update(
    conn_guard: &rusqlite::Connection,
    id: &str,
    auth_type: &str,
    password: Option<&str>,
) -> Result<Option<String>, String> {
    if let Some(password) = password.filter(|value| !value.is_empty()) {
        let master = get_master_password();
        return Ok(Some(encrypt_password(password, &master)));
    }

    if auth_type_keeps_password(auth_type) {
        current_password_enc(conn_guard, id)
    } else {
        Ok(None)
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        if value.trim().is_empty() {
            None
        } else {
            Some(value)
        }
    })
}

fn port_i32_to_u16(port: i32, field: &str) -> Result<u16, String> {
    u16::try_from(port)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{} must be between 1 and 65535", field))
}

fn ssh_params_from_input(db: &DbConn, input: ConnectionInput) -> Result<SshConnectParams, String> {
    let saved = input
        .id
        .as_deref()
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(|id| load_ssh_params(db, id))
        .transpose()?;

    let host = if input.host.trim().is_empty() {
        saved
            .as_ref()
            .map(|params| params.host.clone())
            .ok_or_else(|| "Host is required".to_string())?
    } else {
        input.host
    };
    let port = match input.port {
        Some(port) => port_i32_to_u16(port, "Port")?,
        None => saved.as_ref().map(|params| params.port).unwrap_or(22),
    };
    let auth_type = non_empty(input.auth_type)
        .or_else(|| saved.as_ref().map(|params| params.auth_type.clone()))
        .unwrap_or_else(|| "password".to_string());
    let username = non_empty(input.username)
        .or_else(|| saved.as_ref().map(|params| params.username.clone()))
        .unwrap_or_else(|| "root".to_string());
    let password = non_empty(input.password).or_else(|| {
        if auth_type_keeps_password(&auth_type) {
            saved.as_ref().and_then(|params| params.password.clone())
        } else {
            None
        }
    });
    let proxy_jump_id = non_empty(input.proxy_jump_id);
    let proxy_jump = proxy_jump_id
        .as_deref()
        .map(|jump_id| load_ssh_params(db, jump_id).map(Box::new))
        .transpose()?;
    let use_proxy_jump = proxy_jump.is_some();

    Ok(SshConnectParams {
        host,
        port,
        username,
        auth_type,
        password,
        key_path: non_empty(input.key_path),
        timeout_ms: input.timeout_ms.map(|timeout| timeout.max(1) as u32),
        proxy_type: if use_proxy_jump {
            None
        } else {
            non_empty(input.proxy_type)
        },
        proxy_host: if use_proxy_jump {
            None
        } else {
            non_empty(input.proxy_host)
        },
        proxy_port: if use_proxy_jump {
            None
        } else {
            input
                .proxy_port
                .map(|port| port_i32_to_u16(port, "Proxy port"))
                .transpose()?
        },
        proxy_jump_id,
        proxy_jump,
        init_command: non_empty(input.init_command),
        init_path: non_empty(input.init_path),
        heartbeat_ms: input.heartbeat_ms,
    })
}

fn query_connection(conn_guard: &rusqlite::Connection, id: &str) -> Result<Connection, String> {
    let mut stmt = conn_guard
        .prepare("SELECT id, group_id, name, host, port, auth_type, username, password_enc, key_path, credential_id, proxy_type, proxy_host, proxy_port, proxy_jump_id, init_command, init_path, timeout_ms, heartbeat_ms, remark, created_at, updated_at FROM connections WHERE id = ?1")
        .map_err(|e| e.to_string())?;
    stmt.query_row(rusqlite::params![id], |row| {
        Ok(Connection {
            id: row.get(0)?,
            group_id: row.get(1)?,
            name: row.get(2)?,
            host: row.get(3)?,
            port: row.get(4)?,
            auth_type: row.get(5)?,
            username: row.get(6)?,
            password_enc: row.get(7)?,
            key_path: row.get(8)?,
            credential_id: row.get(9)?,
            proxy_type: row.get(10)?,
            proxy_host: row.get(11)?,
            proxy_port: row.get(12)?,
            proxy_jump_id: row.get(13)?,
            init_command: row.get(14)?,
            init_path: row.get(15)?,
            timeout_ms: row.get(16)?,
            heartbeat_ms: row.get(17)?,
            remark: row.get(18)?,
            created_at: row.get(19)?,
            updated_at: row.get(20)?,
        })
    })
    .map_err(|e| format!("Connection not found: {}", e))
}

fn query_connections(
    conn_guard: &rusqlite::Connection,
    sql: &str,
    params: &[&dyn rusqlite::types::ToSql],
) -> Result<Vec<ConnectionResponse>, String> {
    let mut stmt = conn_guard.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params, |row| {
            Ok(Connection {
                id: row.get(0)?,
                group_id: row.get(1)?,
                name: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                auth_type: row.get(5)?,
                username: row.get(6)?,
                password_enc: row.get(7)?,
                key_path: row.get(8)?,
                credential_id: row.get(9)?,
                proxy_type: row.get(10)?,
                proxy_host: row.get(11)?,
                proxy_port: row.get(12)?,
                proxy_jump_id: row.get(13)?,
                init_command: row.get(14)?,
                init_path: row.get(15)?,
                timeout_ms: row.get(16)?,
                heartbeat_ms: row.get(17)?,
                remark: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        if let Ok(conn) = row {
            result.push(to_response(&conn));
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn get_groups(db: State<'_, DbConn>) -> Result<Vec<Group>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name, parent_id, icon, sort_order, created_at FROM groups ORDER BY sort_order")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                icon: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_group(
    db: State<'_, DbConn>,
    name: String,
    parent_id: Option<String>,
    icon: Option<String>,
) -> Result<Group, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM groups", [], |r| {
            r.get(0)
        })
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO groups (id, name, parent_id, icon, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, name, parent_id, icon, max_order + 1],
    )
    .map_err(|e| e.to_string())?;

    Ok(Group {
        id,
        name,
        parent_id,
        icon,
        sort_order: max_order + 1,
        created_at: None,
    })
}

#[tauri::command]
pub fn update_group(
    db: State<'_, DbConn>,
    id: String,
    name: String,
    icon: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE groups SET name = ?1, icon = ?2 WHERE id = ?3",
        rusqlite::params![name, icon, id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_group(db: State<'_, DbConn>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM groups WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_connections(db: State<'_, DbConn>) -> Result<Vec<ConnectionResponse>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    query_connections(&conn, "SELECT id, group_id, name, host, port, auth_type, username, password_enc, key_path, credential_id, proxy_type, proxy_host, proxy_port, proxy_jump_id, init_command, init_path, timeout_ms, heartbeat_ms, remark, created_at, updated_at FROM connections ORDER BY name", &[])
}

#[tauri::command]
pub fn create_connection(
    db: State<'_, DbConn>,
    input: ConnectionInput,
) -> Result<ConnectionResponse, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = input.id.unwrap_or_else(|| Uuid::new_v4().to_string());
    let port = input.port.unwrap_or(22);
    let auth_type = input.auth_type.unwrap_or_else(|| "password".to_string());
    log::info!(
        target: "myterm::connections",
        "create connection start id={} name={} host={} port={} auth_type={}",
        id,
        input.name,
        input.host,
        port,
        auth_type
    );

    // Encrypt password before storing
    let encrypted_password = if let Some(ref pwd) = input.password {
        if !pwd.is_empty() {
            let master = get_master_password();
            Some(encrypt_password(pwd, &master))
        } else {
            None
        }
    } else {
        None
    };

    let result = conn.execute(
        "INSERT INTO connections (id, group_id, name, host, port, auth_type, username, password_enc, key_path, credential_id, proxy_type, proxy_host, proxy_port, proxy_jump_id, init_command, init_path, timeout_ms, heartbeat_ms, remark) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19)",
        rusqlite::params![
            id, input.group_id, input.name, input.host, port, auth_type,
            input.username, encrypted_password, input.key_path, input.credential_id,
            input.proxy_type, input.proxy_host, input.proxy_port, input.proxy_jump_id,
            input.init_command, input.init_path, input.timeout_ms,
            input.heartbeat_ms.unwrap_or(5000), input.remark
        ],
    );
    if let Err(e) = result {
        log::error!(
            target: "myterm::connections",
            "create connection failed id={} error={}",
            id,
            e
        );
        return Err(e.to_string());
    }

    let connection = query_connection(&conn, &id)?;
    log::info!(
        target: "myterm::connections",
        "create connection success id={}",
        id
    );
    Ok(to_response(&connection))
}

#[tauri::command]
pub fn update_connection(
    db: State<'_, DbConn>,
    id: String,
    input: ConnectionInput,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let auth_type = input.auth_type.unwrap_or_else(|| "password".to_string());
    log::info!(
        target: "myterm::connections",
        "update connection start id={} name={} host={} port={}",
        id,
        input.name,
        input.host,
        input.port.unwrap_or(22)
    );

    let encrypted_password =
        encrypted_password_for_update(&conn, &id, &auth_type, input.password.as_deref())?;

    let result = conn.execute(
        "UPDATE connections SET group_id=?1, name=?2, host=?3, port=?4, auth_type=?5, username=?6, password_enc=?7, key_path=?8, credential_id=?9, proxy_type=?10, proxy_host=?11, proxy_port=?12, proxy_jump_id=?13, init_command=?14, init_path=?15, timeout_ms=?16, heartbeat_ms=?17, remark=?18, updated_at=CURRENT_TIMESTAMP WHERE id=?19",
        rusqlite::params![
            input.group_id, input.name, input.host,
            input.port.unwrap_or(22),
            auth_type,
            input.username, encrypted_password, input.key_path, input.credential_id,
            input.proxy_type, input.proxy_host, input.proxy_port, input.proxy_jump_id,
            input.init_command, input.init_path, input.timeout_ms,
            input.heartbeat_ms.unwrap_or(5000), input.remark, id
        ],
    );
    if let Err(e) = result {
        log::error!(
            target: "myterm::connections",
            "update connection failed id={} error={}",
            id,
            e
        );
        return Err(e.to_string());
    }
    log::info!(
        target: "myterm::connections",
        "update connection success id={}",
        id
    );
    Ok(())
}

#[tauri::command]
pub fn delete_connection(db: State<'_, DbConn>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    log::info!(target: "myterm::connections", "delete connection start id={}", id);
    conn.execute(
        "DELETE FROM connections WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| {
        log::error!(
            target: "myterm::connections",
            "delete connection failed id={} error={}",
            id,
            e
        );
        e.to_string()
    })?;
    log::info!(target: "myterm::connections", "delete connection success id={}", id);
    Ok(())
}

#[tauri::command]
pub fn test_connection(db: State<'_, DbConn>, input: ConnectionInput) -> Result<String, String> {
    let op_id = Uuid::new_v4().to_string();
    let params = ssh_params_from_input(&db, input)?;
    log::info!(
        target: "myterm::connections",
        "test connection start op_id={} host={} port={} username={} auth_type={}",
        op_id,
        params.host,
        params.port,
        params.username,
        params.auth_type
    );
    connect(&params).map_err(|err| {
        log::error!(
            target: "myterm::connections",
            "test connection failed op_id={} host={} port={} error={}",
            op_id,
            params.host,
            params.port,
            err
        );
        err
    })?;
    log::info!(
        target: "myterm::connections",
        "test connection success op_id={} host={} port={}",
        op_id,
        params.host,
        params.port
    );
    Ok("Connection successful".to_string())
}

/// Collect server hardware info (OS, CPU cores, memory, disk) via SSH
#[tauri::command]
pub fn collect_server_info(
    db: State<'_, DbConn>,
    input: ConnectionInput,
) -> Result<ServerInfo, String> {
    use std::io::Read;
    let op_id = Uuid::new_v4().to_string();

    let params = ssh_params_from_input(&db, input)?;

    log::info!(
        target: "myterm::connections",
        "collect server info start op_id={} host={} port={}",
        op_id,
        params.host,
        params.port
    );
    let session = connect(&params).map_err(|err| {
        log::error!(
            target: "myterm::connections",
            "collect server info ssh failed op_id={} host={} port={} error={}",
            op_id,
            params.host,
            params.port,
            err
        );
        err
    })?;

    // Run a script to collect info
    let script = r#"
echo "===OS==="
cat /etc/os-release 2>/dev/null | grep PRETTY_NAME | cut -d'"' -f2 || uname -srm
echo "===CPU==="
nproc 2>/dev/null || echo "1"
echo "===MEM==="
grep MemTotal /proc/meminfo 2>/dev/null | awk '{print $2}' || echo "0"
echo "===DISK==="
df -B1 / 2>/dev/null | tail -1 | awk '{print $2}' || echo "0"
echo "===END==="
"#;

    let mut channel = session.session.channel_session().map_err(|e| {
        log::error!(
            target: "myterm::connections",
            "collect server info channel failed op_id={} error={}",
            op_id,
            e
        );
        format!("Channel failed: {}", e)
    })?;
    channel
        .exec(&format!("sh -c '{}'", script.replace("'", "'\\''")))
        .map_err(|e| {
            log::error!(
                target: "myterm::connections",
                "collect server info exec failed op_id={} error={}",
                op_id,
                e
            );
            format!("Exec failed: {}", e)
        })?;

    let mut output = String::new();
    channel.read_to_string(&mut output).ok();
    channel.wait_close().ok();

    // Parse output
    let os = extract_info(&output, "OS").unwrap_or_default();
    let cpu_cores: u32 = extract_info(&output, "CPU")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);
    let mem_kb: u64 = extract_info(&output, "MEM")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let disk_bytes: u64 = extract_info(&output, "DISK")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let info = ServerInfo {
        os,
        cpu_cores,
        memory_total: mem_kb * 1024, // KB to bytes
        disk_total: disk_bytes,
    };
    log::info!(
        target: "myterm::connections",
        "collect server info success op_id={} os={} cpu_cores={} memory_total={} disk_total={}",
        op_id,
        info.os,
        info.cpu_cores,
        info.memory_total,
        info.disk_total
    );

    Ok(info)
}

fn extract_info(output: &str, section: &str) -> Option<String> {
    let marker = format!("==={}===", section);
    let start = output.find(&marker)? + marker.len();
    let rest = output[start..].trim_start_matches('\n');
    let end = rest.find("\n===").unwrap_or(rest.len());
    let val = rest[..end].trim().to_string();
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

#[derive(serde::Serialize)]
pub struct ServerInfo {
    pub os: String,
    pub cpu_cores: u32,
    pub memory_total: u64,
    pub disk_total: u64,
}

#[tauri::command]
pub fn search_connections(
    db: State<'_, DbConn>,
    query: String,
) -> Result<Vec<ConnectionResponse>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let pattern = format!("%{}%", query);
    query_connections(&conn, "SELECT id, group_id, name, host, port, auth_type, username, password_enc, key_path, credential_id, proxy_type, proxy_host, proxy_port, proxy_jump_id, init_command, init_path, timeout_ms, heartbeat_ms, remark, created_at, updated_at FROM connections WHERE name LIKE ?1 OR host LIKE ?1 OR remark LIKE ?1 ORDER BY name", &[&pattern])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::decrypt_password;
    use crate::db::schema::init_db;

    fn test_connection_db(password: Option<&str>) -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().expect("db");
        init_db(&conn).expect("schema");
        let encrypted = password.map(|pwd| encrypt_password(pwd, &get_master_password()));
        conn.execute(
            "INSERT INTO connections (id, name, host, port, auth_type, username, password_enc, heartbeat_ms) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["conn-1", "Server", "127.0.0.1", 22, "password", "root", encrypted, 5000],
        )
        .expect("insert connection");
        conn
    }

    #[test]
    fn update_password_preserves_existing_password_when_input_omits_password() {
        let conn = test_connection_db(Some("old-secret"));

        let encrypted =
            encrypted_password_for_update(&conn, "conn-1", "password", None).expect("password");

        let plaintext = decrypt_password(
            encrypted.as_deref().expect("encrypted"),
            &get_master_password(),
        )
        .expect("decrypt");
        assert_eq!(plaintext, "old-secret");
    }

    #[test]
    fn update_password_clears_existing_password_when_switching_to_key_auth() {
        let conn = test_connection_db(Some("old-secret"));

        let encrypted =
            encrypted_password_for_update(&conn, "conn-1", "key", None).expect("password");

        assert!(encrypted.is_none());
    }

    #[test]
    fn ssh_params_from_input_reuses_saved_password_for_existing_connection_test() {
        let conn = test_connection_db(Some("old-secret"));
        let db = DbConn(std::sync::Mutex::new(conn));

        let params = ssh_params_from_input(
            &db,
            ConnectionInput {
                id: Some("conn-1".to_string()),
                name: "Server".to_string(),
                host: "127.0.0.1".to_string(),
                port: Some(22),
                auth_type: Some("password".to_string()),
                username: Some("root".to_string()),
                password: None,
                key_path: None,
                group_id: None,
                credential_id: None,
                proxy_type: None,
                proxy_host: None,
                proxy_port: None,
                proxy_jump_id: None,
                init_command: None,
                init_path: None,
                timeout_ms: None,
                heartbeat_ms: None,
                remark: None,
            },
        )
        .expect("params");

        assert_eq!(params.password.as_deref(), Some("old-secret"));
    }
}
