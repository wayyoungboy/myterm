use crate::db::models::ConnectionInput;
use crate::db::DbConn;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub version: String,
    pub connections: Vec<ConnectionInput>,
}

#[tauri::command]
pub fn export_connections(db: State<'_, DbConn>) -> Result<String, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, group_id, name, host, port, auth_type, username, key_path, proxy_type, proxy_host, proxy_port, init_command, init_path, timeout_ms, heartbeat_ms, remark FROM connections ORDER BY name")
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ConnectionInput {
                id: row.get(0)?,
                group_id: row.get(1)?,
                name: row.get(2)?,
                host: row.get(3)?,
                port: row.get(4)?,
                auth_type: row.get(5)?,
                username: row.get(6)?,
                password: None, // Don't export passwords
                key_path: row.get(7)?,
                credential_id: None,
                proxy_type: row.get(8)?,
                proxy_host: row.get(9)?,
                proxy_port: row.get(10)?,
                proxy_jump_id: None,
                init_command: row.get(11)?,
                init_path: row.get(12)?,
                timeout_ms: row.get(13)?,
                heartbeat_ms: row.get(14)?,
                remark: row.get(15)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let connections: Vec<ConnectionInput> = rows.filter_map(|r| r.ok()).collect();

    let export = ExportData {
        version: "1.0".to_string(),
        connections,
    };

    serde_json::to_string_pretty(&export).map_err(|e| format!("Serialization failed: {}", e))
}

#[tauri::command]
pub fn import_connections(db: State<'_, DbConn>, json: String) -> Result<usize, String> {
    let import: ExportData =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON format: {}", e))?;

    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut count = 0;

    for input in import.connections {
        let id = Uuid::new_v4().to_string();
        let port = input.port.unwrap_or(22);
        let auth_type = input.auth_type.unwrap_or_else(|| "password".to_string());

        let result = conn.execute(
            "INSERT INTO connections (id, group_id, name, host, port, auth_type, username, key_path, proxy_type, proxy_host, proxy_port, init_command, init_path, timeout_ms, heartbeat_ms, remark) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
            rusqlite::params![
                id, input.group_id, input.name, input.host, port, auth_type,
                input.username, input.key_path, input.proxy_type, input.proxy_host,
                input.proxy_port, input.init_command, input.init_path, input.timeout_ms,
                input.heartbeat_ms.unwrap_or(5000), input.remark
            ],
        );

        if result.is_ok() {
            count += 1;
        }
    }

    Ok(count)
}
