use crate::db::models::Note;
use crate::db::DbConn;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_notes(db: State<'_, DbConn>, connection_id: Option<String>) -> Result<Vec<Note>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(cid) = connection_id {
        ("SELECT id, connection_id, group_id, title, content, created_at, updated_at FROM notes WHERE connection_id = ?1 ORDER BY updated_at DESC",
         vec![Box::new(cid)])
    } else {
        ("SELECT id, connection_id, group_id, title, content, created_at, updated_at FROM notes ORDER BY updated_at DESC",
         vec![])
    };

    let mut stmt = conn.prepare(sql).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(rusqlite::params_from_iter(params.iter()), |row| {
        Ok(Note {
            id: row.get(0)?,
            connection_id: row.get(1)?,
            group_id: row.get(2)?,
            title: row.get(3)?,
            content: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_note(db: State<'_, DbConn>, connection_id: Option<String>, group_id: Option<String>, title: String, content: String) -> Result<Note, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO notes (id, connection_id, group_id, title, content) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, connection_id, group_id, title, content],
    ).map_err(|e| e.to_string())?;

    Ok(Note {
        id,
        connection_id,
        group_id,
        title: Some(title),
        content: Some(content),
        created_at: None,
        updated_at: None,
    })
}

#[tauri::command]
pub fn update_note(db: State<'_, DbConn>, id: String, title: String, content: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE notes SET title = ?1, content = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        rusqlite::params![title, content, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_note(db: State<'_, DbConn>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM notes WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
