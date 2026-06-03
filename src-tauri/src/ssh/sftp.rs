use crate::db::models::SftpEntry;
use ssh2::{FileStat, Session, Sftp};
use std::path::Path;

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
}
