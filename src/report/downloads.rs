use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const POLL_INTERVAL: Duration = Duration::from_millis(500);

pub fn find_recent_archive(dir: &Path, since: SystemTime) -> Option<PathBuf> {
    fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|entry| is_crash_archive(&entry.file_name().to_string_lossy()))
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            (modified >= since).then_some((modified, entry.path()))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

pub fn wait_for_archive(dir: &Path, since: SystemTime, timeout: Duration) -> Option<PathBuf> {
    let deadline = Instant::now() + timeout;
    let mut last_seen: Option<(PathBuf, u64)> = None;
    while Instant::now() < deadline {
        if let Some(path) = find_recent_archive(dir, since) {
            let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if size > 0 && last_seen.as_ref() == Some(&(path.clone(), size)) {
                return Some(path);
            }
            last_seen = Some((path, size));
        }
        thread::sleep(POLL_INTERVAL);
    }
    None
}

fn is_crash_archive(name: &str) -> bool {
    name.starts_with("CfxCrashDump_") && name.to_ascii_lowercase().ends_with(".zip")
}
