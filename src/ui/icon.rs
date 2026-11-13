use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Controls::{LIM_LARGE, LIM_SMALL, LoadIconMetric};
use windows::Win32::UI::WindowsAndMessaging::{HICON, IDI_APPLICATION, LoadIconW};
use windows::core::PCWSTR;

const APP_ICON_ID: usize = 1;

pub fn small() -> HICON {
    load(false)
}

pub fn large() -> HICON {
    load(true)
}

fn load(large: bool) -> HICON {
    let metric = if large { LIM_LARGE } else { LIM_SMALL };
    unsafe {
        GetModuleHandleW(None)
            .and_then(|module| LoadIconMetric(Some(module.into()), PCWSTR(APP_ICON_ID as *const u16), metric))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))
            .unwrap_or_default()
    }
}
