use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationInvokePattern, TreeScope_Descendants, UIA_ButtonControlTypeId,
    UIA_InvokePatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindow, IsWindowVisible,
    PostMessageW, SW_RESTORE, SetForegroundWindow, ShowWindow, WM_CLOSE,
};
use windows::core::BOOL;

pub const SAVE_BUTTON: &str = "Save information";
pub const CLOSE_BUTTON: &str = "Close";
pub const OK_BUTTON: &str = "OK";
const SAVED_NOTICE_TITLE: &str = "CitizenFX";

pub struct CrashDialog {
    hwnd: HWND,
    automation: IUIAutomation,
    _com: ComApartment,
}

struct ComApartment;

impl ComApartment {
    fn enter() -> Result<Self, String> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok()
                .map_err(|e| e.message())?
        };
        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

impl CrashDialog {
    pub fn attach(raw_hwnd: isize) -> Result<Self, String> {
        let com = ComApartment::enter()?;
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }.map_err(|e| e.message())?;
        Ok(Self {
            hwnd: HWND(raw_hwnd as *mut _),
            automation,
            _com: com,
        })
    }

    pub fn is_open(&self) -> bool {
        unsafe { IsWindow(Some(self.hwnd)).as_bool() }
    }

    pub fn press(&self, label: &str, timeout: Duration) -> Result<(), String> {
        self.press_in(self.hwnd, label, timeout)
    }

    fn press_in(&self, hwnd: HWND, label: &str, timeout: Duration) -> Result<(), String> {
        let deadline = Instant::now() + timeout;
        loop {
            focus(hwnd);
            match self.try_press(hwnd, label) {
                Ok(true) => return Ok(()),
                Ok(false) | Err(_) if Instant::now() < deadline && unsafe { IsWindow(Some(hwnd)).as_bool() } => {
                    thread::sleep(Duration::from_millis(250))
                }
                Ok(false) => return Err(format!("butonul \"{label}\" nu a fost gasit")),
                Err(error) => return Err(error),
            }
        }
    }

    pub fn confirm_saved_notice(&self, timeout: Duration) {
        let mut pid = 0;
        unsafe { GetWindowThreadProcessId(self.hwnd, Some(&mut pid)) };
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if let Some(notice) = find_notice(pid, self.hwnd) {
                if self.press_in(notice, OK_BUTTON, Duration::from_secs(2)).is_err()
                    && unsafe { IsWindow(Some(notice)).as_bool() }
                {
                    unsafe {
                        let _ = PostMessageW(Some(notice), WM_CLOSE, WPARAM(0), LPARAM(0));
                    }
                }
                return;
            }
            thread::sleep(Duration::from_millis(250));
        }
    }

    pub fn close(&self) {
        if self.press(CLOSE_BUTTON, Duration::from_secs(2)).is_err() && self.is_open() {
            unsafe {
                let _ = PostMessageW(Some(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }

    fn try_press(&self, hwnd: HWND, label: &str) -> Result<bool, String> {
        unsafe {
            let root = self.automation.ElementFromHandle(hwnd).map_err(|e| e.message())?;
            let condition = self.automation.CreateTrueCondition().map_err(|e| e.message())?;
            let elements = root
                .FindAll(TreeScope_Descendants, &condition)
                .map_err(|e| e.message())?;
            for index in 0..elements.Length().unwrap_or(0) {
                let Ok(button) = elements.GetElement(index) else {
                    continue;
                };
                if button.CurrentControlType().ok() != Some(UIA_ButtonControlTypeId) {
                    continue;
                }
                let name = button.CurrentName().map(|n| n.to_string()).unwrap_or_default();
                if !name.trim_start().starts_with(label) {
                    continue;
                }
                let pattern: IUIAutomationInvokePattern = button
                    .GetCurrentPatternAs(UIA_InvokePatternId)
                    .map_err(|e| e.message())?;
                pattern.Invoke().map_err(|e| e.message())?;
                return Ok(true);
            }
            Ok(false)
        }
    }
}

struct NoticeSearch {
    pid: u32,
    skip: HWND,
    found: Option<HWND>,
}

fn find_notice(pid: u32, skip: HWND) -> Option<HWND> {
    let mut search = NoticeSearch { pid, skip, found: None };
    unsafe {
        let _ = EnumWindows(Some(enum_notice), LPARAM(&mut search as *mut NoticeSearch as isize));
    }
    search.found
}

unsafe extern "system" fn enum_notice(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let search = unsafe { &mut *(lparam.0 as *mut NoticeSearch) };
    if hwnd == search.skip || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
        return BOOL(1);
    }
    let mut owner = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut owner)) };
    if owner != search.pid {
        return BOOL(1);
    }
    let mut class = [0u16; 16];
    let class_len = unsafe { GetClassNameW(hwnd, &mut class) } as usize;
    let mut title = [0u16; 64];
    let title_len = unsafe { GetWindowTextW(hwnd, &mut title) } as usize;
    if String::from_utf16_lossy(&class[..class_len]) == "#32770"
        && String::from_utf16_lossy(&title[..title_len]) == SAVED_NOTICE_TITLE
    {
        search.found = Some(hwnd);
        return BOOL(0);
    }
    BOOL(1)
}

fn focus(hwnd: HWND) {
    unsafe {
        if IsIconic(hwnd).as_bool() {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
        let _ = SetForegroundWindow(hwnd);
    }
}
