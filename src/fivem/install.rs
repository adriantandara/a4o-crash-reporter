use std::path::{Path, PathBuf};

use crate::storage::paths;

pub fn is_fivem_executable(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_ascii_lowercase())
        .is_some_and(|name| name.starts_with("fivem") && name.ends_with(".exe"))
}

pub fn app_dir(executable: &Path) -> PathBuf {
    for dir in executable.ancestors().skip(1) {
        if dir
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("FiveM.app"))
        {
            return dir.to_path_buf();
        }
        let candidate = dir.join("FiveM.app");
        if candidate.is_dir() {
            return candidate;
        }
    }
    default_app_dir()
}

fn default_app_dir() -> PathBuf {
    paths::local_app_data()
        .unwrap_or_default()
        .join("FiveM")
        .join("FiveM.app")
}
