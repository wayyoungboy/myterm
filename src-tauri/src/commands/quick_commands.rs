use crate::db::models::QuickCommand;
use crate::db::DbConn;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_quick_commands(db: State<'_, DbConn>, group_id: Option<String>) -> Result<Vec<QuickCommand>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(gid) = group_id {
        ("SELECT id, group_id, name, command, shortcut, sort_order FROM quick_commands WHERE group_id = ?1 ORDER BY sort_order",
         vec![Box::new(gid)])
    } else {
        ("SELECT id, group_id, name, command, shortcut, sort_order FROM quick_commands ORDER BY sort_order",
         vec![])
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(QuickCommand {
            id: row.get(0)?,
            group_id: row.get(1)?,
            name: row.get(2)?,
            command: row.get(3)?,
            shortcut: row.get(4)?,
            sort_order: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_quick_command(
    db: State<'_, DbConn>,
    name: String,
    command: String,
    group_id: Option<String>,
    shortcut: Option<String>,
) -> Result<QuickCommand, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let max_order: i32 = conn
        .query_row("SELECT COALESCE(MAX(sort_order), 0) FROM quick_commands", [], |r| r.get(0))
        .unwrap_or(0);

    conn.execute(
        "INSERT INTO quick_commands (id, group_id, name, command, shortcut, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, group_id, name, command, shortcut, max_order + 1],
    ).map_err(|e| e.to_string())?;

    Ok(QuickCommand {
        id,
        group_id,
        name,
        command,
        shortcut,
        sort_order: max_order + 1,
    })
}

#[tauri::command]
pub fn update_quick_command(
    db: State<'_, DbConn>,
    id: String,
    name: String,
    command: String,
    shortcut: Option<String>,
) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE quick_commands SET name = ?1, command = ?2, shortcut = ?3 WHERE id = ?4",
        rusqlite::params![name, command, shortcut, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_quick_command(db: State<'_, DbConn>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM quick_commands WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Replace variables in a command string
/// Supported: ${host}, ${port}, ${username}, ${date}, ${time}
pub fn expand_command(command: &str, host: &str, port: u16, username: &str) -> String {
    let now = chrono::Local::now();
    command
        .replace("${host}", host)
        .replace("${port}", &port.to_string())
        .replace("${username}", username)
        .replace("${date}", &now.format("%Y-%m-%d").to_string())
        .replace("${time}", &now.format("%H:%M:%S").to_string())
}
