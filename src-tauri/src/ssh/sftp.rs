use crate::db::models::SftpEntry;
use ssh2::{FileStat, Session, Sftp};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

const TRANSFER_BUFFER_SIZE: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct TransferProgress {
    pub path: String,
    pub file_name: String,
    pub bytes_transferred: u64,
    pub total_bytes: u64,
}

pub fn list_dir(session: &Session, path: &str) -> Result<Vec<SftpEntry>, String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    let entries_raw = sftp
        .readdir(std::path::Path::new(path))
        .map_err(|e| format!("Read dir failed: {}", e))?;

    let mut entries = Vec::new();
    for (path_buf, stat) in entries_raw {
        let name = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name == "." || name == ".." {
            continue;
        }

        let is_dir = stat.is_dir();
        let size = stat.size.unwrap_or(0);
        let permissions = format!("{:o}", stat.perm.unwrap_or(0) & 0o777);
        let modified = stat
            .mtime
            .map(|t| {
                chrono::DateTime::from_timestamp(t as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        entries.push(SftpEntry {
            name,
            path: path_buf.to_string_lossy().to_string(),
            is_dir,
            size,
            permissions,
            modified,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_dir == b.is_dir {
            a.name.cmp(&b.name)
        } else if a.is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    Ok(entries)
}

pub fn read_file(session: &Session, path: &str) -> Result<Vec<u8>, String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    let mut file = sftp
        .open(std::path::Path::new(path))
        .map_err(|e| format!("Open file failed: {}", e))?;

    let mut contents = Vec::new();
    use std::io::Read;
    file.read_to_end(&mut contents)
        .map_err(|e| format!("Read failed: {}", e))?;
    Ok(contents)
}

pub fn write_file(session: &Session, path: &str, data: &[u8]) -> Result<(), String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    let mut file = sftp
        .create(std::path::Path::new(path))
        .map_err(|e| format!("Open file failed: {}", e))?;

    use std::io::Write;
    file.write_all(data)
        .map_err(|e| format!("Write failed: {}", e))?;
    Ok(())
}

pub fn download_path_with_progress(
    session: &Session,
    remote_path: &str,
    local_parent: &str,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(TransferProgress) -> Result<(), String>,
) -> Result<usize, String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    let remote = Path::new(remote_path);
    let root_name = source_entry_name(remote)?;
    let local_target = Path::new(local_parent).join(root_name);

    download_path_inner(&sftp, remote, &local_target, cancel, &mut on_progress)
}

fn download_path_inner(
    sftp: &Sftp,
    remote_path: &Path,
    local_path: &Path,
    cancel: &AtomicBool,
    on_progress: &mut impl FnMut(TransferProgress) -> Result<(), String>,
) -> Result<usize, String> {
    check_transfer_cancelled(cancel)?;
    let stat = sftp
        .stat(remote_path)
        .map_err(|e| format!("Remote stat failed for {}: {}", remote_path.display(), e))?;

    if stat.is_dir() {
        fs::create_dir_all(local_path).map_err(|e| {
            format!(
                "Create local dir failed for {}: {}",
                local_path.display(),
                e
            )
        })?;

        let entries = sftp.readdir(remote_path).map_err(|e| {
            format!(
                "Remote read dir failed for {}: {}",
                remote_path.display(),
                e
            )
        })?;
        let mut copied = 0;
        for (child_path, _) in entries {
            let name = source_entry_name(&child_path)?;
            if name == "." || name == ".." {
                continue;
            }
            let child_remote = remote_child_path(&remote_path.to_string_lossy(), &name);
            let child_local = local_path.join(name);
            copied += download_path_inner(
                sftp,
                Path::new(&child_remote),
                &child_local,
                cancel,
                on_progress,
            )?;
        }
        Ok(copied)
    } else {
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!("Create local parent failed for {}: {}", parent.display(), e)
            })?;
        }

        let mut remote_file = sftp.open(remote_path).map_err(|e| {
            format!(
                "Open remote file failed for {}: {}",
                remote_path.display(),
                e
            )
        })?;
        let mut local_file = fs::File::create(local_path).map_err(|e| {
            format!(
                "Create local file failed for {}: {}",
                local_path.display(),
                e
            )
        })?;
        let total_bytes = stat.size.unwrap_or(0);
        let path = remote_path.to_string_lossy().to_string();
        let file_name = source_entry_name(remote_path)?;
        copy_stream_with_progress(
            &mut remote_file,
            &mut local_file,
            total_bytes,
            cancel,
            |bytes, total| {
                on_progress(TransferProgress {
                    path: path.clone(),
                    file_name: file_name.clone(),
                    bytes_transferred: bytes,
                    total_bytes: total,
                })
            },
        )
        .map_err(|e| {
            format!(
                "Copy remote file failed from {} to {}: {}",
                remote_path.display(),
                local_path.display(),
                e
            )
        })?;
        Ok(1)
    }
}

pub fn upload_path_with_progress(
    session: &Session,
    local_path: &str,
    remote_parent: &str,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(TransferProgress) -> Result<(), String>,
) -> Result<usize, String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    let local = Path::new(local_path);
    let root_name = source_entry_name(local)?;
    let remote_target = remote_child_path(remote_parent, &root_name);

    upload_path_inner(&sftp, local, &remote_target, cancel, &mut on_progress)
}

fn upload_path_inner(
    sftp: &Sftp,
    local_path: &Path,
    remote_path: &str,
    cancel: &AtomicBool,
    on_progress: &mut impl FnMut(TransferProgress) -> Result<(), String>,
) -> Result<usize, String> {
    check_transfer_cancelled(cancel)?;
    let metadata = fs::metadata(local_path)
        .map_err(|e| format!("Local stat failed for {}: {}", local_path.display(), e))?;

    if metadata.is_dir() {
        ensure_remote_dir(sftp, remote_path)?;
        let mut copied = 0;
        let entries = fs::read_dir(local_path)
            .map_err(|e| format!("Local read dir failed for {}: {}", local_path.display(), e))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Local read dir entry failed: {}", e))?;
            let child_path = entry.path();
            let name = source_entry_name(&child_path)?;
            let child_remote = remote_child_path(remote_path, &name);
            copied += upload_path_inner(sftp, &child_path, &child_remote, cancel, on_progress)?;
        }
        Ok(copied)
    } else {
        let mut local_file = fs::File::open(local_path)
            .map_err(|e| format!("Open local file failed for {}: {}", local_path.display(), e))?;
        let mut remote_file = sftp
            .create(Path::new(remote_path))
            .map_err(|e| format!("Create remote file failed for {}: {}", remote_path, e))?;
        let total_bytes = metadata.len();
        let path = local_path.to_string_lossy().to_string();
        let file_name = source_entry_name(local_path)?;
        copy_stream_with_progress(
            &mut local_file,
            &mut remote_file,
            total_bytes,
            cancel,
            |bytes, total| {
                on_progress(TransferProgress {
                    path: path.clone(),
                    file_name: file_name.clone(),
                    bytes_transferred: bytes,
                    total_bytes: total,
                })
            },
        )
        .map_err(|e| {
            format!(
                "Copy local file failed from {} to {}: {}",
                local_path.display(),
                remote_path,
                e
            )
        })?;
        remote_file
            .flush()
            .map_err(|e| format!("Flush remote file failed for {}: {}", remote_path, e))?;
        Ok(1)
    }
}

fn ensure_remote_dir(sftp: &Sftp, remote_path: &str) -> Result<(), String> {
    match sftp.mkdir(Path::new(remote_path), 0o755) {
        Ok(()) => Ok(()),
        Err(err) => match sftp.stat(Path::new(remote_path)) {
            Ok(stat) if stat.is_dir() => Ok(()),
            _ => Err(format!(
                "Create remote dir failed for {}: {}",
                remote_path, err
            )),
        },
    }
}

pub fn remove_file(session: &Session, path: &str) -> Result<(), String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    remove_path(&sftp, Path::new(path))
}

fn remove_path(sftp: &Sftp, path: &Path) -> Result<(), String> {
    let stat = sftp
        .stat(path)
        .map_err(|e| format!("Stat before remove failed for {}: {}", path.display(), e))?;

    if stat.is_dir() {
        let entries = sftp.readdir(path).map_err(|e| {
            format!(
                "Read dir before remove failed for {}: {}",
                path.display(),
                e
            )
        })?;

        for (child_path, _) in entries {
            let Some(name) = child_path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if name == "." || name == ".." {
                continue;
            }
            remove_path(sftp, &child_path)?;
        }

        sftp.rmdir(path)
            .map_err(|e| format!("Remove dir failed for {}: {}", path.display(), e))
    } else {
        sftp.unlink(path)
            .map_err(|e| format!("Remove file failed for {}: {}", path.display(), e))
    }
}

pub fn rename(session: &Session, src: &str, dst: &str) -> Result<(), String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    sftp.rename(std::path::Path::new(src), std::path::Path::new(dst), None)
        .map_err(|e| format!("Rename failed: {}", e))?;
    Ok(())
}

pub fn mkdir(session: &Session, path: &str) -> Result<(), String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    sftp.mkdir(std::path::Path::new(path), 0o755)
        .map_err(|e| format!("Mkdir failed: {}", e))?;
    Ok(())
}

pub fn chmod(session: &Session, path: &str, mode: &str) -> Result<(), String> {
    let mode = parse_chmod_mode(mode)?;
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    sftp.setstat(
        std::path::Path::new(path),
        FileStat {
            size: None,
            uid: None,
            gid: None,
            perm: Some(mode),
            atime: None,
            mtime: None,
        },
    )
    .map_err(|e| format!("Chmod failed: {}", e))?;
    Ok(())
}

fn parse_chmod_mode(mode: &str) -> Result<u32, String> {
    let trimmed = mode.trim();
    if !(trimmed.len() == 3 || trimmed.len() == 4) {
        return Err("Mode must be a 3 or 4 digit octal value".to_string());
    }
    if !trimmed.chars().all(|ch| ('0'..='7').contains(&ch)) {
        return Err("Mode must contain only octal digits 0-7".to_string());
    }

    let parsed =
        u32::from_str_radix(trimmed, 8).map_err(|e| format!("Mode parse failed: {}", e))?;
    Ok(parsed & 0o7777)
}

fn remote_child_path(parent: &str, child_name: &str) -> String {
    if parent == "/" {
        format!("/{child_name}")
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), child_name)
    }
}

fn source_entry_name(path: &Path) -> Result<String, String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| format!("Source path has no file name: {}", path.display()))
}

fn copy_stream_with_progress(
    reader: &mut impl Read,
    writer: &mut impl Write,
    total_bytes: u64,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(u64, u64) -> Result<(), String>,
) -> Result<u64, String> {
    let mut buf = [0u8; TRANSFER_BUFFER_SIZE];
    let mut transferred = 0u64;

    loop {
        check_transfer_cancelled(cancel)?;
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Read transfer stream failed: {}", e))?;
        if n == 0 {
            break;
        }
        writer
            .write_all(&buf[..n])
            .map_err(|e| format!("Write transfer stream failed: {}", e))?;
        transferred += n as u64;
        on_progress(transferred, total_bytes)?;
    }

    Ok(transferred)
}

fn check_transfer_cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        Err("Transfer cancelled".to_string())
    } else {
        Ok(())
    }
}

#[allow(dead_code)]
pub fn stat(session: &Session, path: &str) -> Result<ssh2::FileStat, String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    sftp.stat(std::path::Path::new(path))
        .map_err(|e| format!("Stat failed: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn parses_three_digit_octal_mode() {
        assert_eq!(parse_chmod_mode("755").expect("mode"), 0o755);
        assert_eq!(parse_chmod_mode("600").expect("mode"), 0o600);
    }

    #[test]
    fn parses_four_digit_octal_mode() {
        assert_eq!(parse_chmod_mode("0644").expect("mode"), 0o644);
        assert_eq!(parse_chmod_mode("1755").expect("mode"), 0o1755);
    }

    #[test]
    fn rejects_invalid_chmod_modes() {
        assert!(parse_chmod_mode("").is_err());
        assert!(parse_chmod_mode("88").is_err());
        assert!(parse_chmod_mode("10000").is_err());
        assert!(parse_chmod_mode("8888").is_err());
        assert!(parse_chmod_mode("rwx").is_err());
    }

    #[test]
    fn joins_remote_child_paths_without_duplicate_slashes() {
        assert_eq!(remote_child_path("/", "app.log"), "/app.log");
        assert_eq!(remote_child_path("/var/log", "app.log"), "/var/log/app.log");
        assert_eq!(
            remote_child_path("/var/log/", "app.log"),
            "/var/log/app.log"
        );
    }

    #[test]
    fn derives_copy_root_name_from_source_path() {
        assert_eq!(
            source_entry_name(Path::new("/home/deploy/releases")).expect("name"),
            "releases"
        );
        assert_eq!(
            source_entry_name(Path::new("/home/deploy/app.tar.gz")).expect("name"),
            "app.tar.gz"
        );
        assert!(source_entry_name(Path::new("/")).is_err());
    }

    #[test]
    fn copy_stream_reports_byte_progress() {
        let mut reader = Cursor::new(vec![1u8; TRANSFER_BUFFER_SIZE * 2 + 7]);
        let mut writer = Vec::new();
        let cancel = AtomicBool::new(false);
        let mut progress = Vec::new();

        let copied = copy_stream_with_progress(
            &mut reader,
            &mut writer,
            TRANSFER_BUFFER_SIZE as u64 * 2 + 7,
            &cancel,
            |bytes, total| {
                progress.push((bytes, total));
                Ok(())
            },
        )
        .expect("copy");

        assert_eq!(copied, TRANSFER_BUFFER_SIZE as u64 * 2 + 7);
        assert_eq!(writer.len(), TRANSFER_BUFFER_SIZE * 2 + 7);
        assert_eq!(progress.last(), Some(&(copied, copied)));
        assert!(progress.len() >= 2);
    }

    #[test]
    fn copy_stream_stops_when_cancelled() {
        let mut reader = Cursor::new(vec![1u8; TRANSFER_BUFFER_SIZE * 3]);
        let mut writer = Vec::new();
        let cancel = AtomicBool::new(false);

        let err = copy_stream_with_progress(
            &mut reader,
            &mut writer,
            TRANSFER_BUFFER_SIZE as u64 * 3,
            &cancel,
            |_, _| {
                cancel.store(true, Ordering::SeqCst);
                Ok(())
            },
        )
        .expect_err("cancelled");

        assert!(err.contains("cancelled"));
        assert!(writer.len() < TRANSFER_BUFFER_SIZE * 3);
    }
}
