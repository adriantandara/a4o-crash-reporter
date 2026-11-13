use std::fs;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_INFO, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIIF_INFO, NIIF_NOSOUND, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NIM_SETVERSION, NOTIFYICON_VERSION_4, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow, DispatchMessageW,
    FindWindowW, GetCursorPos, GetMessageW, HMENU, IDYES, MB_ICONQUESTION, MB_YESNO, MF_CHECKED, MF_GRAYED,
    MF_SEPARATOR, MF_STRING, MSG, MessageBoxW, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW,
    SetForegroundWindow, TPM_BOTTOMALIGN, TPM_RIGHTBUTTON, TrackPopupMenu, TranslateMessage, WINDOW_EX_STYLE, WM_APP,
    WM_CLOSE, WM_COMMAND, WM_CONTEXTMENU, WM_DESTROY, WM_LBUTTONUP, WM_NULL, WM_RBUTTONUP, WM_TIMER, WNDCLASSW,
    WS_OVERLAPPED,
};
use windows::core::{HSTRING, PCWSTR, w};

use super::{icon, updates};
use crate::config;
use crate::fivem::watcher;
use crate::net::api;
use crate::storage::{self, history, settings};
use crate::system::{autostart, shell};

const WINDOW_CLASS: PCWSTR = w!("ALL4ONE.CrashReporter.Tray");
const WM_TRAY: u32 = WM_APP + 1;
const WM_PROCESS_EXIT: u32 = WM_APP + 2;
const WM_UPDATE_RESULT: u32 = WM_APP + 3;
const TRAY_ID: u32 = 1;

const MENU_PAUSE: usize = 10;
const MENU_AUTO_CLOSE: usize = 11;
const MENU_AUTOSTART: usize = 12;
const MENU_HISTORY: usize = 13;
const MENU_SOURCE: usize = 14;
const MENU_PRIVACY: usize = 15;
const MENU_WITHDRAW: usize = 16;
const MENU_EXIT: usize = 17;
const MENU_UPDATES_POLICY: usize = 18;

static WINDOW: AtomicIsize = AtomicIsize::new(0);
static TASKBAR_CREATED: AtomicIsize = AtomicIsize::new(0);
static FIVEM_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn run() {
    let Some(hwnd) = create_window() else { return };
    WINDOW.store(hwnd.0 as isize, Ordering::SeqCst);
    add_icon(hwnd);
    watcher::start();
    updates::schedule(hwnd);

    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    watcher::stop();
    remove_icon(hwnd);
}

pub fn close_running_instance() {
    unsafe {
        if let Ok(hwnd) = FindWindowW(WINDOW_CLASS, PCWSTR::null()) {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

pub fn set_fivem_running(running: bool) {
    FIVEM_RUNNING.store(running, Ordering::SeqCst);
    if let Some(hwnd) = window() {
        let mut data = icon_data(hwnd);
        data.uFlags = NIF_TIP | NIF_SHOWTIP;
        copy_wide(&mut data.szTip, &tooltip());
        unsafe {
            let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
        }
    }
}

pub fn post_process_exit(pid: u32) {
    if let Some(hwnd) = window() {
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_PROCESS_EXIT, WPARAM(pid as usize), LPARAM(0));
        }
    }
}

pub fn post_update_result(result: usize, manual: bool) {
    if let Some(hwnd) = window() {
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_UPDATE_RESULT, WPARAM(result), LPARAM(manual as isize));
        }
    }
}

pub fn notify(title: &str, text: &str) {
    let Some(hwnd) = window() else { return };
    let mut data = icon_data(hwnd);
    data.uFlags = NIF_INFO;
    data.dwInfoFlags = NIIF_INFO | NIIF_NOSOUND;
    copy_wide(&mut data.szInfoTitle, title);
    copy_wide(&mut data.szInfo, text);
    unsafe {
        let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
    }
}

fn window() -> Option<HWND> {
    let raw = WINDOW.load(Ordering::SeqCst);
    (raw != 0).then_some(HWND(raw as *mut _))
}

fn create_window() -> Option<HWND> {
    unsafe {
        let instance = GetModuleHandleW(None).ok()?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance.into(),
            lpszClassName: WINDOW_CLASS,
            ..Default::default()
        };
        RegisterClassW(&class);
        TASKBAR_CREATED.store(RegisterWindowMessageW(w!("TaskbarCreated")) as isize, Ordering::SeqCst);
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            WINDOW_CLASS,
            &HSTRING::from(config::APP_NAME),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .ok()
    }
}

fn icon_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

fn add_icon(hwnd: HWND) {
    let mut data = icon_data(hwnd);
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = icon::small();
    copy_wide(&mut data.szTip, &tooltip());
    unsafe {
        let _ = Shell_NotifyIconW(NIM_ADD, &data);
        data.Anonymous.uVersion = NOTIFYICON_VERSION_4;
        let _ = Shell_NotifyIconW(NIM_SETVERSION, &data);
    }
}

fn remove_icon(hwnd: HWND) {
    unsafe {
        let _ = Shell_NotifyIconW(NIM_DELETE, &icon_data(hwnd));
    }
}

fn status_text() -> &'static str {
    if settings::get_bool(settings::PAUSED, false) {
        "Pe pauza, crash-urile nu se trimit"
    } else if FIVEM_RUNNING.load(Ordering::SeqCst) {
        "FiveM pornit, urmaresc crash-urile"
    } else {
        "Astept pornirea FiveM"
    }
}

fn tooltip() -> String {
    format!("{}\n{}", config::APP_NAME, status_text())
}

unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match message {
        WM_TRAY => {
            let event = (lparam.0 & 0xFFFF) as u32;
            if matches!(event, WM_CONTEXTMENU | WM_RBUTTONUP | WM_LBUTTONUP) {
                show_menu(hwnd);
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            handle_command(hwnd, wparam.0 & 0xFFFF);
            LRESULT(0)
        }
        WM_PROCESS_EXIT => {
            watcher::on_process_exit(wparam.0 as u32);
            LRESULT(0)
        }
        WM_UPDATE_RESULT => {
            updates::on_result(wparam.0, lparam.0 != 0);
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == updates::TIMER_ID => {
            updates::on_timer(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            LRESULT(0)
        }
        _ if message as isize == TASKBAR_CREATED.load(Ordering::SeqCst) => {
            add_icon(hwnd);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn show_menu(hwnd: HWND) {
    let paused = settings::get_bool(settings::PAUSED, false);
    let auto_close = settings::get_bool(settings::AUTO_CLOSE, true);
    let autostart = settings::get_bool(settings::AUTOSTART, true);

    unsafe {
        let Ok(menu) = CreatePopupMenu() else { return };
        let title = format!("{} {}", config::APP_NAME, config::APP_VERSION);
        add_item(menu, 0, &title, false, true);
        add_item(menu, 0, status_text(), false, true);
        add_separator(menu);
        updates::append_menu(menu);
        add_item(menu, MENU_PAUSE, "Pune pe pauza", paused, false);
        add_item(
            menu,
            MENU_AUTO_CLOSE,
            "Inchide automat fereastra de eroare FiveM",
            auto_close,
            false,
        );
        add_item(menu, MENU_AUTOSTART, "Porneste odata cu Windows", autostart, false);
        updates::append_settings(menu);
        add_separator(menu);
        add_item(menu, MENU_HISTORY, "Istoricul rapoartelor", false, false);
        add_item(menu, MENU_SOURCE, "Codul sursa", false, false);
        add_item(menu, MENU_PRIVACY, "Politica de confidentialitate", false, false);
        add_item(menu, MENU_UPDATES_POLICY, "Politica de actualizari", false, false);
        add_separator(menu);
        add_item(menu, MENU_WITHDRAW, "Retrage acordul si sterge datele", false, false);
        add_item(menu, MENU_EXIT, "Iesire", false, false);

        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let _ = SetForegroundWindow(hwnd);
        let _ = TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_BOTTOMALIGN,
            cursor.x,
            cursor.y,
            None,
            hwnd,
            None,
        );
        let _ = PostMessageW(Some(hwnd), WM_NULL, WPARAM(0), LPARAM(0));
        let _ = DestroyMenu(menu);
    }
}

pub(super) unsafe fn add_item(menu: HMENU, id: usize, text: &str, checked: bool, disabled: bool) {
    let mut flags = MF_STRING;
    if checked {
        flags |= MF_CHECKED;
    }
    if disabled {
        flags |= MF_GRAYED;
    }
    let _ = unsafe { AppendMenuW(menu, flags, id, &HSTRING::from(text)) };
}

pub(super) unsafe fn add_separator(menu: HMENU) {
    let _ = unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()) };
}

fn handle_command(hwnd: HWND, id: usize) {
    if updates::handle_command(hwnd, id) {
        return;
    }
    match id {
        MENU_PAUSE => {
            settings::set(settings::PAUSED, !settings::get_bool(settings::PAUSED, false));
            set_fivem_running(FIVEM_RUNNING.load(Ordering::SeqCst));
        }
        MENU_AUTO_CLOSE => settings::set(settings::AUTO_CLOSE, !settings::get_bool(settings::AUTO_CLOSE, true)),
        MENU_AUTOSTART => {
            let enabled = !settings::get_bool(settings::AUTOSTART, true);
            settings::set(settings::AUTOSTART, enabled);
            autostart::set_enabled(enabled);
        }
        MENU_HISTORY => {
            let file = history::file();
            if !file.exists() {
                storage::paths::ensure_data_dir();
                let _ = fs::write(
                    &file,
                    "Istoricul rapoartelor ALL4ONE Crash Reporter\n\nNiciun raport trimis.\n",
                );
            }
            shell::open_path(&file);
        }
        MENU_SOURCE => shell::open_url(config::REPOSITORY_URL),
        MENU_PRIVACY => shell::open_url(config::PRIVACY_URL),
        MENU_UPDATES_POLICY => shell::open_url(config::UPDATES_URL),
        MENU_WITHDRAW => {
            if confirm_withdraw(hwnd) {
                api::forget_device();
                autostart::set_enabled(false);
                storage::purge();
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
            }
        }
        MENU_EXIT => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        _ => {}
    }
}

fn confirm_withdraw(hwnd: HWND) -> bool {
    let text = HSTRING::from(
        "Aplicatia se va opri, nu va mai porni cu Windows si iti va sterge datele locale \
         (setari, cheia acestui PC, istoricul).\n\nRapoartele deja trimise raman pana la stergerea \
         automata dupa 90 de zile sau pana ceri stergerea pe Discord.\n\nContinui?",
    );
    let answer = unsafe {
        MessageBoxW(
            Some(hwnd),
            &text,
            &HSTRING::from(config::APP_NAME),
            MB_YESNO | MB_ICONQUESTION,
        )
    };
    answer == IDYES
}

fn copy_wide<const N: usize>(target: &mut [u16; N], text: &str) {
    let wide: Vec<u16> = text.encode_utf16().take(N - 1).collect();
    target[..wide.len()].copy_from_slice(&wide);
    target[wide.len()] = 0;
}
