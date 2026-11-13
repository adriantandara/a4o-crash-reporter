use std::path::PathBuf;
use std::sync::OnceLock;

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{FOLDERID_Downloads, FOLDERID_LocalAppData, KF_FLAG_DEFAULT, SHGetKnownFolderPath};
use windows::core::GUID;

use crate::config;

pub fn data_dir() -> &'static PathBuf {
    static DIR: OnceLock<PathBuf> = OnceLock::new();
    DIR.get_or_init(|| {
        let base = known_folder(&FOLDERID_LocalAppData).unwrap_or_else(std::env::temp_dir);
        base.join("ALL4ONE").join(config::DATA_FOLDER)
    })
}

pub fn ensure_data_dir() {
    let _ = std::fs::create_dir_all(data_dir());
}

pub fn local_app_data() -> Option<PathBuf> {
    known_folder(&FOLDERID_LocalAppData)
}

pub fn downloads_dir() -> Option<PathBuf> {
    known_folder(&FOLDERID_Downloads)
}

fn known_folder(id: &GUID) -> Option<PathBuf> {
    unsafe {
        let raw = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None).ok()?;
        let path = raw.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(raw.0 as _));
        path
    }
}
