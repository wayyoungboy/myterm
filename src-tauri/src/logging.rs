use chrono::Local;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};
use parking_lot::Mutex;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

struct FileLogger {
    file: Mutex<File>,
    level: LevelFilter,
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &Metadata<'_>) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record<'_>) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let line = format!(
            "{timestamp} [{:<5}] [{}] {}\n",
            record.level(),
            record.target(),
            record.args()
        );

        {
            let mut file = self.file.lock();
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }

        eprint!("{line}");
    }

    fn flush(&self) {
        let _ = self.file.lock().flush();
    }
}

fn level_from_env() -> LevelFilter {
    match std::env::var("MYTERM_LOG")
        .unwrap_or_else(|_| "info".to_string())
        .to_lowercase()
        .as_str()
    {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" | "warning" => LevelFilter::Warn,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

pub fn init(app_dir: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(app_dir).map_err(|e| format!("Failed to create log dir: {e}"))?;
    let path = app_dir.join("myterm.log");
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("Failed to open log file {}: {e}", path.display()))?;

    let level = level_from_env();
    set_logger(FileLogger {
        file: Mutex::new(file),
        level,
    })
    .map_err(|e| format!("Failed to initialize logger: {e}"))?;
    log::set_max_level(level);
    log::info!(
        target: "myterm::logging",
        "logger initialized path={} level={level:?}",
        path.display()
    );

    Ok(path)
}

fn set_logger(logger: FileLogger) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(logger))
}
