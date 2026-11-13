use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    HMENU, IDYES, KillTimer, MB_ICONQUESTION, MB_YESNO, MessageBoxW, SetTimer,
};
use windows::core::HSTRING;

use super::tray;
use crate::storage::settings;
use crate::system::shell;
use crate::update::{checker, installer};
use crate::{config, log_error, log_info};

pub const TIMER_ID: usize = 1;

const FIRST_CHECK_MS: u32 = 2 * 60 * 1000;
const CHECK_INTERVAL_MS: u32 = 12 * 60 * 60 * 1000;

const MENU_INSTALL: usize = 30;
const MENU_NOTES: usize = 31;
const MENU_SKIP: usize = 32;
const MENU_CHECK_NOW: usize = 33;
const MENU_AUTO_CHECK: usize = 34;

const RESULT_NONE: usize = 0;
const RESULT_FOUND: usize = 1;
const RESULT_FAILED: usize = 2;

static BUSY: AtomicBool = AtomicBool::new(false);

pub fn schedule(hwnd: HWND) {
    unsafe { SetTimer(Some(hwnd), TIMER_ID, FIRST_CHECK_MS, None) };
}

pub fn on_timer(hwnd: HWND) {
    unsafe {
        let _ = KillTimer(Some(hwnd), TIMER_ID);
        SetTimer(Some(hwnd), TIMER_ID, CHECK_INTERVAL_MS, None);
    }
    if settings::get_bool(settings::UPDATE_CHECK, true) {
        start_check(false);
    }
}

pub fn on_result(result: usize, manual: bool) {
    match (result, checker::available()) {
        (RESULT_FOUND, Some(update)) => {
            let skipped = settings::get_string(settings::SKIPPED_VERSION).as_deref() == Some(update.version.as_str());
            let notified = settings::get_string(settings::NOTIFIED_VERSION).as_deref() == Some(update.version.as_str());
            if manual || (!skipped && !notified) {
                settings::set(settings::NOTIFIED_VERSION, update.version.as_str());
                tray::notify(
                    &format!("Versiunea {} este disponibila", update.version),
                    "Deschide meniul aplicatiei ca sa vezi ce s-a schimbat si sa alegi daca o instalezi.",
                );
            }
        }
        (RESULT_FAILED, _) if manual => tray::notify(
            "Verificarea a esuat",
            "Nu am putut verifica actualizarile. Incearca mai tarziu.",
        ),
        (_, _) if manual => tray::notify(
            "Nicio actualizare",
            &format!("Ai ultima versiune ({}).", config::APP_VERSION),
        ),
        _ => {}
    }
}

pub unsafe fn append_menu(menu: HMENU) {
    if let Some(update) = checker::available() {
        unsafe {
            tray::add_item(
                menu,
                MENU_INSTALL,
                &format!("Instaleaza versiunea {}", update.version),
                false,
                BUSY.load(Ordering::SeqCst),
            );
            tray::add_item(
                menu,
                MENU_NOTES,
                &format!("Ce s-a schimbat in {}", update.version),
                false,
                false,
            );
            tray::add_item(
                menu,
                MENU_SKIP,
                &format!("Ignora versiunea {}", update.version),
                false,
                false,
            );
            tray::add_separator(menu);
        }
    }
}

pub unsafe fn append_settings(menu: HMENU) {
    unsafe {
        tray::add_item(
            menu,
            MENU_AUTO_CHECK,
            "Cauta actualizari automat",
            settings::get_bool(settings::UPDATE_CHECK, true),
            false,
        );
        tray::add_item(
            menu,
            MENU_CHECK_NOW,
            "Cauta actualizari acum",
            false,
            BUSY.load(Ordering::SeqCst),
        );
    }
}

pub fn handle_command(hwnd: HWND, id: usize) -> bool {
    match id {
        MENU_INSTALL => install(hwnd),
        MENU_NOTES => {
            if let Some(update) = checker::available() {
                shell::open_url(&update.notes_url());
            }
        }
        MENU_SKIP => {
            if let Some(update) = checker::available() {
                settings::set(settings::SKIPPED_VERSION, update.version.as_str());
            }
        }
        MENU_CHECK_NOW => start_check(true),
        MENU_AUTO_CHECK => settings::set(
            settings::UPDATE_CHECK,
            !settings::get_bool(settings::UPDATE_CHECK, true),
        ),
        _ => return false,
    }
    true
}

fn start_check(manual: bool) {
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    let spawned = thread::Builder::new().name("update-check".into()).spawn(move || {
        let result = match checker::check() {
            Ok(Some(update)) => {
                log_info!("versiune noua disponibila: {}", update.version);
                RESULT_FOUND
            }
            Ok(None) => RESULT_NONE,
            Err(error) => {
                log_error!("verificarea actualizarilor a esuat: {error}");
                RESULT_FAILED
            }
        };
        BUSY.store(false, Ordering::SeqCst);
        tray::post_update_result(result, manual);
    });
    if spawned.is_err() {
        BUSY.store(false, Ordering::SeqCst);
    }
}

fn install(hwnd: HWND) {
    let Some(update) = checker::available() else { return };
    let text = HSTRING::from(format!(
        "Instalezi versiunea {} a {}?\n\nInstaller-ul se descarca de pe GitHub, se verifica semnatura si \
         amprenta SHA-256 si apoi porneste. Aplicatia se inchide si reporneste dupa instalare.\n\n\
         Actualizarea nu este obligatorie.",
        update.version,
        config::APP_NAME
    ));
    let answer = unsafe {
        MessageBoxW(
            Some(hwnd),
            &text,
            &HSTRING::from(config::APP_NAME),
            MB_YESNO | MB_ICONQUESTION,
        )
    };
    if answer != IDYES || BUSY.swap(true, Ordering::SeqCst) {
        return;
    }
    tray::notify("Se descarca actualizarea", &format!("Versiunea {}", update.version));
    let spawned = thread::Builder::new().name("update-install".into()).spawn(move || {
        let result = installer::download_and_launch(&update);
        BUSY.store(false, Ordering::SeqCst);
        match result {
            Ok(()) => {
                log_info!("installer-ul pentru {} a pornit", update.version);
                tray::close_running_instance();
            }
            Err(error) => {
                log_error!("actualizarea la {} a esuat: {error}", update.version);
                tray::notify("Actualizarea nu a reusit", &error);
            }
        }
    });
    if spawned.is_err() {
        BUSY.store(false, Ordering::SeqCst);
    }
}
