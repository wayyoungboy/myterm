use crate::terminal::TerminalManager;
use crate::db::models::MonitorData;
use tauri::State;

#[tauri::command]
pub fn get_monitor_data(
    tm: State<'_, TerminalManager>,
    session_id: String,
) -> Result<MonitorData, String> {
    let session = tm.get_session(&session_id).ok_or("Session not found")?;
    crate::monitor::fetch_monitor_data(&session)
}
