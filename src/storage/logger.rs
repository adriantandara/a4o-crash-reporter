use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

use super::paths;
use crate::system::time;

const MAX_LOG_SIZE: u64 = 256 * 1024;

static LOCK: Mutex<()> = Mutex::new(());

pub fn write(level: &str, message: &str) {
    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    paths::ensure_data_dir();
    let path = paths::data_dir().join("reporter.log");
    if fs::metadata(&path).map(|m| m.len() > MAX_LOG_SIZE).unwrap_or(false) {
        let _ = fs::rename(&path, path.with_extension("old.log"));
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{} [{}] {}", time::local_timestamp(), level, message);
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => { $crate::storage::logger::write("info", &format!($($arg)*)) };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => { $crate::storage::logger::write("error", &format!($($arg)*)) };
}
