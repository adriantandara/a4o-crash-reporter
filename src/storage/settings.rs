use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use serde_json::{Map, Value};

use super::paths;

static SETTINGS: Mutex<Option<Map<String, Value>>> = Mutex::new(None);

pub const CONSENT_VERSION: &str = "consent_version";
pub const CONSENT_DATE: &str = "consent_date";
pub const PAUSED: &str = "paused";
pub const AUTO_CLOSE: &str = "auto_close_dialog";
pub const AUTOSTART: &str = "autostart";

fn file() -> PathBuf {
    paths::data_dir().join("settings.json")
}

fn with<T>(f: impl FnOnce(&mut Map<String, Value>) -> T) -> T {
    let mut guard = SETTINGS.lock().unwrap_or_else(|e| e.into_inner());
    let map = guard.get_or_insert_with(|| {
        fs::read(file())
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .and_then(|value| value.as_object().cloned())
            .unwrap_or_default()
    });
    f(map)
}

pub fn get_bool(key: &str, default: bool) -> bool {
    with(|map| map.get(key).and_then(Value::as_bool).unwrap_or(default))
}

pub fn get_u64(key: &str) -> Option<u64> {
    with(|map| map.get(key).and_then(Value::as_u64))
}

pub fn set(key: &str, value: impl Into<Value>) {
    with(|map| {
        map.insert(key.to_owned(), value.into());
        paths::ensure_data_dir();
        if let Ok(text) = serde_json::to_vec_pretty(map) {
            let _ = fs::write(file(), text);
        }
    });
}

pub fn forget() {
    with(|map| map.clear());
}
