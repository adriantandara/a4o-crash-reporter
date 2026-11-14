use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const POLL_INTERVAL: Duration = Duration::from_millis(500);
const DIALOG_GONE_GRACE: Duration = Duration::from_secs(10);

pub fn find_recent_archive(dirs: &[PathBuf], since: SystemTime) -> Option<PathBuf> {
    dirs.iter()
        .filter_map(|dir| fs::read_dir(dir).ok())
        .flat_map(|entries| entries.flatten())
        .filter(|entry| is_crash_archive(&entry.file_name().to_string_lossy()))
        .filter_map(|entry| {
            let modified = entry.metadata().ok()?.modified().ok()?;
            (modified >= since).then_some((modified, entry.path()))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

pub fn wait_for_archive(
    dirs: &[PathBuf],
    since: SystemTime,
    timeout: Duration,
    dialog_open: impl Fn() -> bool,
) -> Option<PathBuf> {
    let deadline = Instant::now() + timeout;
    let mut closed_at: Option<Instant> = None;
    let mut last_seen: Option<(PathBuf, u64)> = None;
    while Instant::now() < deadline {
        if dialog_open() {
            closed_at = None;
        } else if closed_at.get_or_insert_with(Instant::now).elapsed() > DIALOG_GONE_GRACE {
            return None;
        }
        if let Some(path) = find_recent_archive(dirs, since) {
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
