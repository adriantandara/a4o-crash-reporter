use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime};

use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_MODE_BACKGROUND_BEGIN};

use super::bundle::Bundle;
use super::downloads;
use crate::fivem::dialog::{CrashDialog, SAVE_BUTTON};
use crate::fivem::{crashometry, install};
use crate::net::api::{self, Submitted};
use crate::storage::{history, paths, settings};
use crate::ui::tray;
use crate::{config, log_error, log_info};

const SAVE_TIMEOUT: Duration = Duration::from_secs(15);
const AUTOMATIC_SAVE_WAIT: Duration = Duration::from_secs(90);
const MANUAL_SAVE_WAIT: Duration = Duration::from_secs(600);
const CLOCK_TOLERANCE: Duration = Duration::from_secs(10);

static BUSY: AtomicBool = AtomicBool::new(false);
static HANDLED_DIALOGS: Mutex<Vec<isize>> = Mutex::new(Vec::new());

enum Outcome {
    Sent(String, Submitted),
    OtherServer(String),
}

pub fn start(dialog: HWND, executable: PathBuf) {
    if settings::get_bool(settings::PAUSED, false) {
        return;
    }
    let raw_hwnd = dialog.0 as isize;
    {
        let mut handled = HANDLED_DIALOGS.lock().unwrap_or_else(|e| e.into_inner());
        if handled.contains(&raw_hwnd) {
            return;
        }
        handled.push(raw_hwnd);
        if handled.len() > 32 {
            handled.remove(0);
        }
    }
    if BUSY.swap(true, Ordering::SeqCst) {
        return;
    }

    let spawned = thread::Builder::new().name("crash-report".into()).spawn(move || {
        unsafe {
            let _ = SetThreadPriority(GetCurrentThread(), THREAD_MODE_BACKGROUND_BEGIN);
        }
        let result = process_crash(raw_hwnd, &executable);
        BUSY.store(false, Ordering::SeqCst);
        finish(result);
    });
    if spawned.is_err() {
        BUSY.store(false, Ordering::SeqCst);
    }
}

fn process_crash(raw_hwnd: isize, executable: &Path) -> Result<Outcome, String> {
    let app_dir = install::app_dir(executable);
    let server = crashometry::last_server(&app_dir).ok_or("ultimul server FiveM nu a putut fi citit")?;
    if !config::is_allowed_server(&server) {
        return Ok(Outcome::OtherServer(server));
    }
    log_info!("crash FiveM detectat pe {server}");

    let downloads_dir = paths::downloads_dir().ok_or("folderul Descarcari nu a fost gasit")?;
    let since = SystemTime::now() - CLOCK_TOLERANCE;
    let dialog = CrashDialog::attach(raw_hwnd)?;

    let wait = match downloads::find_recent_archive(&downloads_dir, since) {
        Some(_) => AUTOMATIC_SAVE_WAIT,
        None => match dialog.press(SAVE_BUTTON, SAVE_TIMEOUT) {
            Ok(()) => AUTOMATIC_SAVE_WAIT,
            Err(error) => {
                log_error!("nu am putut apasa \"{SAVE_BUTTON}\": {error}; astept salvarea manuala");
                MANUAL_SAVE_WAIT
            }
        },
    };

    let archive =
        downloads::wait_for_archive(&downloads_dir, since, wait).ok_or("FiveM nu a salvat arhiva de crash")?;
    if settings::get_bool(settings::AUTO_CLOSE, true) {
        dialog.close();
    }
    drop(dialog);

    let bundle = Bundle::from_archive(&archive)?;
    let submitted = api::submit(&bundle)?;
    Ok(Outcome::Sent(bundle.label().to_owned(), submitted))
}

fn finish(result: Result<Outcome, String>) {
    match result {
        Ok(Outcome::Sent(crash, Submitted::New(report_id))) => {
            log_info!("raport {report_id} trimis pentru {crash}");
            history::record("trimis", &crash, &format!("raport #{report_id}"));
            tray::notify("Raport trimis", "Multumim! Crash-ul a ajuns la echipa ALL4ONE.");
        }
        Ok(Outcome::Sent(crash, Submitted::Duplicate)) => {
            history::record("deja trimis", &crash, "");
        }
        Ok(Outcome::OtherServer(server)) => {
            log_info!("crash ignorat, ultimul server a fost {server}");
        }
        Err(error) => {
            log_error!("raportul nu a fost trimis: {error}");
            history::record("netrimis", "-", &error);
            tray::notify("Raportul nu a fost trimis", &error);
        }
    }
}
