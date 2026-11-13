use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

use super::paths;
use crate::system::time;

const MAX_HISTORY_SIZE: u64 = 128 * 1024;

pub fn file() -> PathBuf {
    paths::data_dir().join("rapoarte.txt")
}

pub fn record(status: &str, crash: &str, details: &str) {
    paths::ensure_data_dir();
    let path = file();
    if fs::metadata(&path).map(|m| m.len() > MAX_HISTORY_SIZE).unwrap_or(false) {
        let _ = fs::remove_file(&path);
    }
    let is_new = !path.exists();
    if let Ok(mut out) = OpenOptions::new().create(true).append(true).open(&path) {
        if is_new {
            let _ = writeln!(out, "Istoricul rapoartelor ALL4ONE Crash Reporter\n");
        }
        let _ = writeln!(out, "{}  {}  {}  {}", time::local_timestamp(), status, crash, details);
    }
}
