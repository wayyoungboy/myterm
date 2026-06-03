use crate::db::models::{AiConversation, AiMessage};
use crate::db::DbConn;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub fn get_ai_conversations(db: State<'_, DbConn>) -> Result<Vec<AiConversation>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, title, created_at FROM ai_conversations ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(AiConversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_ai_conversation(
    db: State<'_, DbConn>,
    title: Option<String>,
) -> Result<AiConversation, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let title = title.unwrap_or_else(|| {
        format!(
            "Conversation {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M")
        )
    });
    conn.execute(
        "INSERT INTO ai_conversations (id, title) VALUES (?1, ?2)",
        rusqlite::params![id, title],
    )
    .map_err(|e| e.to_string())?;
    Ok(AiConversation {
        id,
        title: Some(title),
        created_at: None,
    })
}

#[tauri::command]
pub fn delete_ai_conversation(db: State<'_, DbConn>, id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "DELETE FROM ai_conversations WHERE id = ?1",
        rusqlite::params![id],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_ai_messages(
    db: State<'_, DbConn>,
    conversation_id: String,
) -> Result<Vec<AiMessage>, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, conversation_id, role, content, created_at FROM ai_messages WHERE conversation_id = ?1 ORDER BY created_at")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params![conversation_id], |row| {
            Ok(AiMessage {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_ai_message(
    db: State<'_, DbConn>,
    conversation_id: String,
    role: String,
    content: String,
) -> Result<AiMessage, String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO ai_messages (id, conversation_id, role, content) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, conversation_id, role, content],
    )
    .map_err(|e| e.to_string())?;
    Ok(AiMessage {
        id,
        conversation_id,
        role,
        content,
        created_at: None,
    })
}
