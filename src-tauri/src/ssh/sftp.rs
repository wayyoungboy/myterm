use crate::db::models::SftpEntry;
use ssh2::Session;

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

pub fn remove_file(session: &Session, path: &str) -> Result<(), String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    sftp.unlink(std::path::Path::new(path))
        .map_err(|e| format!("Remove failed: {}", e))?;
    Ok(())
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

#[allow(dead_code)]
pub fn stat(session: &Session, path: &str) -> Result<ssh2::FileStat, String> {
    let sftp = session
        .sftp()
        .map_err(|e| format!("SFTP init failed: {}", e))?;
    sftp.stat(std::path::Path::new(path))
        .map_err(|e| format!("Stat failed: {}", e))
}
