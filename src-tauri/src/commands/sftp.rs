use crate::commands::ssh_params::connect_for_terminal_session;
use crate::db::models::SftpEntry;
use crate::db::DbConn;
use crate::ssh::sftp::TransferProgress;
use crate::terminal::TerminalManager;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

pub struct SftpTransferManager {
    cancellations: parking_lot::Mutex<HashMap<String, Arc<AtomicBool>>>,
}

impl SftpTransferManager {
    pub fn new() -> Self {
        Self {
            cancellations: parking_lot::Mutex::new(HashMap::new()),
        }
    }

    fn register(&self, transfer_id: &str) -> Arc<AtomicBool> {
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancellations
            .lock()
            .insert(transfer_id.to_string(), cancel.clone());
        cancel
    }

    fn finish(&self, transfer_id: &str) {
        self.cancellations.lock().remove(transfer_id);
    }

    fn cancel(&self, transfer_id: &str) -> bool {
        match self.cancellations.lock().get(transfer_id) {
            Some(cancel) => {
                cancel.store(true, Ordering::SeqCst);
                true
            }
            None => false,
        }
    }
}

#[derive(Clone, Serialize)]
struct TransferProgressEvent {
    transfer_id: String,
    path: String,
    file_name: String,
    bytes_transferred: u64,
    total_bytes: u64,
}

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
pub fn sftp_download_path(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    transfers: State<'_, SftpTransferManager>,
    app_handle: AppHandle,
    transfer_id: String,
    session_id: String,
    remote_path: String,
    local_parent: String,
) -> Result<usize, String> {
    let path = format!("{remote_path} -> {local_parent}");
    let cancel = transfers.register(&transfer_id);
    let result = run_sftp_op(&db, &tm, &session_id, "download_path", &path, |session| {
        crate::ssh::sftp::download_path_with_progress(
            session,
            &remote_path,
            &local_parent,
            &cancel,
            |progress| emit_transfer_progress(&app_handle, &transfer_id, progress),
        )
    });
    transfers.finish(&transfer_id);
    result
}

#[tauri::command]
pub fn sftp_upload_path(
    db: State<'_, DbConn>,
    tm: State<'_, TerminalManager>,
    transfers: State<'_, SftpTransferManager>,
    app_handle: AppHandle,
    transfer_id: String,
    session_id: String,
    local_path: String,
    remote_parent: String,
) -> Result<usize, String> {
    let path = format!("{local_path} -> {remote_parent}");
    let cancel = transfers.register(&transfer_id);
    let result = run_sftp_op(&db, &tm, &session_id, "upload_path", &path, |session| {
        crate::ssh::sftp::upload_path_with_progress(
            session,
            &local_path,
            &remote_parent,
            &cancel,
            |progress| emit_transfer_progress(&app_handle, &transfer_id, progress),
        )
    });
    transfers.finish(&transfer_id);
    result
}

#[tauri::command]
pub fn sftp_cancel_transfer(
    transfers: State<'_, SftpTransferManager>,
    transfer_id: String,
) -> Result<(), String> {
    if transfers.cancel(&transfer_id) {
        log::info!(
            target: "myterm::sftp",
            "transfer cancel requested transfer_id={}",
            transfer_id
        );
        Ok(())
    } else {
        log::warn!(
            target: "myterm::sftp",
            "transfer cancel requested for missing transfer_id={}",
            transfer_id
        );
        Err("Transfer not found".to_string())
    }
}

fn emit_transfer_progress(
    app_handle: &AppHandle,
    transfer_id: &str,
    progress: TransferProgress,
) -> Result<(), String> {
    app_handle
        .emit(
            &format!("sftp-transfer-progress-{transfer_id}"),
            TransferProgressEvent {
                transfer_id: transfer_id.to_string(),
                path: progress.path,
                file_name: progress.file_name,
                bytes_transferred: progress.bytes_transferred,
                total_bytes: progress.total_bytes,
            },
        )
        .map_err(|e| format!("Emit transfer progress failed: {}", e))
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
