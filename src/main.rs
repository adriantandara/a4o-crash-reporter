#![windows_subsystem = "windows"]

mod config;
mod fivem;
mod net;
mod report;
mod storage;
mod system;
mod ui;
mod update;

use system::instance::SingleInstance;

fn main() {
    if std::env::args().any(|arg| arg == "--purge") {
        ui::tray::close_running_instance();
        std::thread::sleep(std::time::Duration::from_secs(1));
        system::autostart::set_enabled(false);
        storage::purge();
        return;
    }

    let Some(_instance) = SingleInstance::acquire() else {
        return;
    };
    if !ui::consent::ensure() {
        return;
    }

    log_info!("pornit, versiunea {}", config::APP_VERSION);
    ui::tray::run();
    log_info!("oprit");
}
