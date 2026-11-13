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
use windows::Win32::UI::WindowsAndMessaging::{IsWindow, PostMessageW, WM_CLOSE};

pub const SAVE_BUTTON: &str = "Save information";
pub const CLOSE_BUTTON: &str = "Close";

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
        let deadline = Instant::now() + timeout;
        loop {
            match self.try_press(label) {
                Ok(true) => return Ok(()),
                Ok(false) | Err(_) if Instant::now() < deadline && self.is_open() => {
                    thread::sleep(Duration::from_millis(250))
                }
                Ok(false) => return Err(format!("butonul \"{label}\" nu a fost gasit")),
                Err(error) => return Err(error),
            }
        }
    }

    pub fn close(&self) {
        if self.press(CLOSE_BUTTON, Duration::from_secs(2)).is_err() && self.is_open() {
            unsafe {
                let _ = PostMessageW(Some(self.hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }

    fn try_press(&self, label: &str) -> Result<bool, String> {
        unsafe {
            let root = self.automation.ElementFromHandle(self.hwnd).map_err(|e| e.message())?;
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
