use std::cell::RefCell;
use std::ffi::c_void;
use std::path::PathBuf;

use windows::Win32::Foundation::{CloseHandle, HANDLE, HWND};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    INFINITE, OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    QueryFullProcessImageNameW, RegisterWaitForSingleObject, UnregisterWaitEx, WT_EXECUTEONLYONCE,
};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::WindowsAndMessaging::{
    CHILDID_SELF, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, GA_ROOT, GetAncestor, GetClassNameW, GetWindowTextW,
    GetWindowThreadProcessId, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
};
use windows::core::PWSTR;

use super::install;
use crate::report::pipeline;
use crate::ui::tray;

struct TrackedProcess {
    pid: u32,
    handle: HANDLE,
    wait: HANDLE,
}

#[derive(Default)]
struct Watcher {
    processes: Vec<TrackedProcess>,
    foreground_hook: Option<HWINEVENTHOOK>,
    show_hook: Option<HWINEVENTHOOK>,
}

thread_local! {
    static WATCHER: RefCell<Watcher> = RefCell::new(Watcher::default());
}

pub fn start() {
    let hook = install_hook(EVENT_SYSTEM_FOREGROUND);
    WATCHER.with_borrow_mut(|watcher| watcher.foreground_hook = hook);
    scan_processes();
}

pub fn stop() {
    WATCHER.with_borrow_mut(|watcher| {
        for hook in [watcher.foreground_hook.take(), watcher.show_hook.take()]
            .into_iter()
            .flatten()
        {
            unsafe {
                let _ = UnhookWinEvent(hook);
            }
        }
        for process in watcher.processes.drain(..) {
            release(process);
        }
    });
}

pub fn on_process_exit(pid: u32) {
    let now_idle = WATCHER.with_borrow_mut(|watcher| {
        if let Some(index) = watcher.processes.iter().position(|p| p.pid == pid) {
            release(watcher.processes.remove(index));
        }
        if !watcher.processes.is_empty() {
            return false;
        }
        if let Some(hook) = watcher.show_hook.take() {
            unsafe {
                let _ = UnhookWinEvent(hook);
            }
        }
        true
    });
    if now_idle {
        tray::set_fivem_running(false);
    }
}

fn install_hook(event: u32) -> Option<HWINEVENTHOOK> {
    let hook = unsafe {
        SetWinEventHook(
            event,
            event,
            None,
            Some(on_window_event),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };
    (!hook.is_invalid()).then_some(hook)
}

fn release(process: TrackedProcess) {
    unsafe {
        let _ = UnregisterWaitEx(process.wait, None);
        let _ = CloseHandle(process.handle);
    }
}

fn is_tracked(pid: u32) -> bool {
    WATCHER.with_borrow(|watcher| watcher.processes.iter().any(|p| p.pid == pid))
}

fn scan_processes() {
    let Ok(snapshot) = (unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }) else {
        return;
    };
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut found = Vec::new();
    let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
    while ok {
        let name = String::from_utf16_lossy(&entry.szExeFile[..wide_len(&entry.szExeFile)]);
        if install::is_fivem_executable(name.as_ref()) {
            found.push(entry.th32ProcessID);
        }
        ok = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
    }
    unsafe {
        let _ = CloseHandle(snapshot);
    }

    for pid in found {
        if !is_tracked(pid) {
            track(pid);
        }
    }
}

fn track(pid: u32) {
    let Ok(handle) = (unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }) else {
        return;
    };
    let mut wait = HANDLE::default();
    let registered = unsafe {
        RegisterWaitForSingleObject(
            &mut wait,
            handle,
            Some(on_wait_signaled),
            Some(pid as usize as *const c_void),
            INFINITE,
            WT_EXECUTEONLYONCE,
        )
    };
    if registered.is_err() {
        unsafe {
            let _ = CloseHandle(handle);
        }
        return;
    }

    let first = WATCHER.with_borrow_mut(|watcher| {
        watcher.processes.push(TrackedProcess { pid, handle, wait });
        if watcher.show_hook.is_none() {
            watcher.show_hook = install_hook(EVENT_OBJECT_SHOW);
            return true;
        }
        false
    });
    if first {
        tray::set_fivem_running(true);
    }
}

unsafe extern "system" fn on_wait_signaled(context: *mut c_void, _timed_out: bool) {
    tray::post_process_exit(context as usize as u32);
}

unsafe extern "system" fn on_window_event(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    object_id: i32,
    child_id: i32,
    _thread: u32,
    _time: u32,
) {
    if hwnd.is_invalid() || object_id != OBJID_WINDOW.0 || child_id != CHILDID_SELF as i32 {
        return;
    }
    if event == EVENT_OBJECT_SHOW && (unsafe { GetAncestor(hwnd, GA_ROOT) } != hwnd || !is_dialog(hwnd)) {
        return;
    }

    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    let Some(executable) = process_path(pid).filter(|path| install::is_fivem_executable(path)) else {
        return;
    };

    if !is_tracked(pid) {
        scan_processes();
    }
    if is_dialog(hwnd) && window_title(hwnd).starts_with("Error") {
        pipeline::start(hwnd, executable);
    }
}

fn is_dialog(hwnd: HWND) -> bool {
    let mut buffer = [0u16; 16];
    let len = unsafe { GetClassNameW(hwnd, &mut buffer) } as usize;
    String::from_utf16_lossy(&buffer[..len]) == "#32770"
}

fn window_title(hwnd: HWND) -> String {
    let mut buffer = [0u16; 128];
    let len = unsafe { GetWindowTextW(hwnd, &mut buffer) } as usize;
    String::from_utf16_lossy(&buffer[..len])
}

fn process_path(pid: u32) -> Option<PathBuf> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }.ok()?;
    let mut buffer = [0u16; 1024];
    let mut len = buffer.len() as u32;
    let result =
        unsafe { QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, PWSTR(buffer.as_mut_ptr()), &mut len) };
    unsafe {
        let _ = CloseHandle(handle);
    }
    result.ok()?;
    Some(PathBuf::from(String::from_utf16_lossy(&buffer[..len as usize])))
}

fn wide_len(buffer: &[u16]) -> usize {
    buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len())
}
