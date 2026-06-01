use crate::terminal::TerminalManager;
use crate::db::models::SftpEntry;
use tauri::State;

fn with_blocking_session<F, R>(tm: &TerminalManager, session_id: &str, f: F) -> Result<R, String>
where
    F: FnOnce(&ssh2::Session) -> Result<R, String>,
{
    let session = tm.get_session(session_id).ok_or("Session not found")?;
    // Temporarily set blocking mode for SFTP operations
    session.set_blocking(true);
    let result = f(&session);
    // Set back to non-blocking for terminal
    session.set_blocking(false);
    result
}

#[tauri::command]
pub fn sftp_list_dir(
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<Vec<SftpEntry>, String> {
    with_blocking_session(&tm, &session_id, |session| {
        crate::ssh::sftp::list_dir(session, &path)
    })
}

#[tauri::command]
pub fn sftp_read_file(
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<Vec<u8>, String> {
    with_blocking_session(&tm, &session_id, |session| {
        crate::ssh::sftp::read_file(session, &path)
    })
}

#[tauri::command]
pub fn sftp_write_file(
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
    data: Vec<u8>,
) -> Result<(), String> {
    with_blocking_session(&tm, &session_id, |session| {
        crate::ssh::sftp::write_file(session, &path, &data)
    })
}

#[tauri::command]
pub fn sftp_remove_file(
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    with_blocking_session(&tm, &session_id, |session| {
        crate::ssh::sftp::remove_file(session, &path)
    })
}

#[tauri::command]
pub fn sftp_rename(
    tm: State<'_, TerminalManager>,
    session_id: String,
    src: String,
    dst: String,
) -> Result<(), String> {
    with_blocking_session(&tm, &session_id, |session| {
        crate::ssh::sftp::rename(session, &src, &dst)
    })
}

#[tauri::command]
pub fn sftp_mkdir(
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    with_blocking_session(&tm, &session_id, |session| {
        crate::ssh::sftp::mkdir(session, &path)
    })
}
