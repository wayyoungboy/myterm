use crate::commands::ssh_params::connect_for_terminal_session;
use crate::db::models::SftpEntry;
use crate::db::DbConn;
use crate::terminal::TerminalManager;
use tauri::State;

fn run_sftp_op<T>(
    db: &DbConn,
    tm: &TerminalManager,
    session_id: &str,
    op: &str,
    path: &str,
    f: impl FnOnce(&ssh2::Session) -> Result<T, String>,
) -> Result<T, String> {
    let op_id = uuid::Uuid::new_v4().to_string();
    let started = std::time::Instant::now();
    log::info!(
        target: "myterm::sftp",
        "operation start op_id={} op={} session_id={} path={}",
        op_id,
        op,
        session_id,
        path
    );

    let result = connect_for_terminal_session(db, tm, session_id).and_then(|ssh| f(&ssh.session));

    match result {
        Ok(value) => {
            log::info!(
                target: "myterm::sftp",
                "operation success op_id={} op={} session_id={} elapsed_ms={}",
                op_id,
                op,
                session_id,
                started.elapsed().as_millis()
            );
            Ok(value)
        }
        Err(err) => {
            log::error!(
                target: "myterm::sftp",
                "operation failed op_id={} op={} session_id={} elapsed_ms={} error={}",
                op_id,
                op,
                session_id,
                started.elapsed().as_millis(),
                err
            );
            Err(err)
        }
    }
}

#[tauri::command]
pub fn sftp_list_dir(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<Vec<SftpEntry>, String> {
    run_sftp_op(&db, &tm, &session_id, "list_dir", &path, |session| {
        crate::ssh::sftp::list_dir(session, &path)
    })
}

#[tauri::command]
pub fn sftp_read_file(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<Vec<u8>, String> {
    run_sftp_op(&db, &tm, &session_id, "read_file", &path, |session| {
        crate::ssh::sftp::read_file(session, &path)
    })
}

#[tauri::command]
pub fn sftp_write_file(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
    data: Vec<u8>,
) -> Result<(), String> {
    run_sftp_op(&db, &tm, &session_id, "write_file", &path, |session| {
        crate::ssh::sftp::write_file(session, &path, &data)
    })
}

#[tauri::command]
pub fn sftp_remove_file(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    run_sftp_op(&db, &tm, &session_id, "remove_file", &path, |session| {
        crate::ssh::sftp::remove_file(session, &path)
    })
}

#[tauri::command]
pub fn sftp_rename(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    src: String,
    dst: String,
) -> Result<(), String> {
    let path = format!("{src} -> {dst}");
    run_sftp_op(&db, &tm, &session_id, "rename", &path, |session| {
        crate::ssh::sftp::rename(session, &src, &dst)
    })
}

#[tauri::command]
pub fn sftp_mkdir(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
) -> Result<(), String> {
    run_sftp_op(&db, &tm, &session_id, "mkdir", &path, |session| {
        crate::ssh::sftp::mkdir(session, &path)
    })
}

#[tauri::command]
pub fn sftp_chmod(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    session_id: String,
    path: String,
    mode: String,
) -> Result<(), String> {
    run_sftp_op(&db, &tm, &session_id, "chmod", &path, |session| {
        crate::ssh::sftp::chmod(session, &path, &mode)
    })
}
