use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::core::w;

pub struct SingleInstance(#[allow(dead_code)] HANDLE);

impl SingleInstance {
    pub fn acquire() -> Option<Self> {
        let handle = unsafe { CreateMutexW(None, true, w!("Local\\ALL4ONE.CrashReporter")) }.ok()?;
        if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
            return None;
        }
        Some(Self(handle))
    }
}
