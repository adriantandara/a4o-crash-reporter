use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows::core::{PCWSTR, w};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const VALUE_NAME: PCWSTR = w!("ALL4ONE Crash Reporter");

pub fn set_enabled(enabled: bool) {
    if cfg!(feature = "local") {
        return;
    }
    let Some(key) = open_run_key() else { return };
    unsafe {
        if enabled {
            if let Ok(exe) = std::env::current_exe() {
                let command = format!("\"{}\" --autostart", exe.display());
                let data: Vec<u8> = command.encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect();
                let _ = RegSetValueExW(key, VALUE_NAME, None, REG_SZ, Some(&data));
            }
        } else {
            let _ = RegDeleteValueW(key, VALUE_NAME);
        }
        let _ = RegCloseKey(key);
    }
}

fn open_run_key() -> Option<HKEY> {
    let mut key = HKEY::default();
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, RUN_KEY, None, KEY_SET_VALUE, &mut key) };
    status.is_ok().then_some(key)
}
