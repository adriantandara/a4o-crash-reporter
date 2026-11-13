use std::path::Path;

use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, w};

use crate::config;

pub fn open_url(url: &str) {
    if url.starts_with(config::REPOSITORY_URL) {
        open(&HSTRING::from(url));
    }
}

pub fn open_path(path: &Path) {
    open(&HSTRING::from(path.as_os_str()));
}

fn open(target: &HSTRING) {
    unsafe { ShellExecuteW(None, w!("open"), target, None, None, SW_SHOWNORMAL) };
}
