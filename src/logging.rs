/// File logging setup for KeyBlast.
///
/// Uses tracing + tracing-appender for rolling log files with daily rotation.

use std::path::PathBuf;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

/// Returns the path to the log directory.
///
/// Location:
/// - macOS: ~/Library/Application Support/keyblast/logs
/// - Windows: %APPDATA%/keyblast/logs
/// - Linux: ~/.local/share/keyblast/logs
pub fn log_directory() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("keyblast")
        .join("logs")
}

/// Initialize file logging with daily rotation and 7-day retention.
///
/// Returns the WorkerGuard that must be kept alive for the duration of the program.
/// If logging setup fails, returns None and the application continues without file logging.
pub fn init_file_logging() -> Option<WorkerGuard> {
    let log_dir = log_directory();

    // Ensure log directory exists
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Warning: Could not create log directory: {} - {}", log_dir.display(), e);
        return None;
    }

    // Create rolling file appender with daily rotation
    let file_appender = match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("keyblast")
        .filename_suffix("log")
        .max_log_files(7) // Keep 7 days of logs
        .build(&log_dir)
    {
        Ok(appender) => appender,
        Err(e) => {
            eprintln!("Warning: Could not create log appender: {}", e);
            return None;
        }
    };

    // Wrap in non-blocking writer for performance
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Initialize the subscriber (use try_init to avoid panic on double-init)
    if fmt::Subscriber::builder()
        .with_writer(non_blocking)
        .with_ansi(false) // No ANSI colors in log files
        .try_init()
        .is_err()
    {
        eprintln!("Warning: Logging already initialized");
    }

    Some(guard)
}

/// Open the log directory in the system file browser.
///
/// Opens Finder on macOS, Explorer on Windows, or the default file manager on Linux.
pub fn open_logs_directory() {
    let log_dir = log_directory();

    if !log_dir.exists() {
        eprintln!("Log directory does not exist: {}", log_dir.display());
        return;
    }

    if let Err(e) = open::that(&log_dir) {
        eprintln!("Failed to open logs directory: {}", e);
    }
}
