pub mod history;
pub mod logger;
pub mod paths;
pub mod settings;

pub fn purge() {
    settings::forget();
    let dir = paths::data_dir();
    let _ = std::fs::remove_dir_all(dir);
    if let Some(parent) = dir.parent() {
        let _ = std::fs::remove_dir(parent);
    }
}
