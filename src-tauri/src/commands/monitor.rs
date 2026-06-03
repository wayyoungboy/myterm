use crate::commands::ssh_params::connect_for_terminal_session;
use crate::db::DbConn;
use crate::db::models::MonitorData;
use crate::terminal::TerminalManager;
use tauri::State;

#[tauri::command]
pub fn get_monitor_data(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
) -> Result<MonitorData, String> {
    let op_id = uuid::Uuid::new_v4().to_string();
    let started = std::time::Instant::now();
    log::info!(
        target: "myterm::monitor",
        "fetch start op_id={} session_id={}",
        op_id,
        session_id
    );
    let result = connect_for_terminal_session(&db, &tm, &session_id)
        .and_then(|ssh| crate::monitor::fetch_monitor_data(&ssh.session));

    match result {
        Ok(data) => {
            log::info!(
                target: "myterm::monitor",
                "fetch success op_id={} session_id={} hostname={} elapsed_ms={}",
                op_id,
                session_id,
                data.hostname,
                started.elapsed().as_millis()
            );
            Ok(data)
        }
        Err(err) => {
            log::error!(
                target: "myterm::monitor",
                "fetch failed op_id={} session_id={} elapsed_ms={} error={}",
                op_id,
                session_id,
                started.elapsed().as_millis(),
                err
            );
            Err(err)
        }
    }
}
