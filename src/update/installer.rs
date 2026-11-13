use std::fs::{self, File};
use std::io::Write;

use sha2::{Digest, Sha256};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, w};

use super::manifest::ReleaseManifest;
use crate::net::{encoding::hex, http};
use crate::storage::paths;
use crate::system::trust;

pub fn download_and_launch(manifest: &ReleaseManifest) -> Result<(), String> {
    let dir = paths::data_dir().join("updates");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(&manifest.installer_name);

    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut size = 0u64;
    http::download(&manifest.installer_url(), |chunk| {
        size += chunk.len() as u64;
        if size > manifest.installer_size {
            return Err("fisierul descarcat este mai mare decat cel anuntat".into());
        }
        hasher.update(chunk);
        file.write_all(chunk).map_err(|e| e.to_string())
    })
    .map_err(|e| e.0)?;
    file.sync_all().map_err(|e| e.to_string())?;
    drop(file);

    let reject = |reason: &str| {
        let _ = fs::remove_file(&path);
        Err(reason.to_owned())
    };
    if size != manifest.installer_size || hex(&hasher.finalize()) != manifest.installer_sha256 {
        return reject("fisierul descarcat nu corespunde semnaturii versiunii");
    }
    if manifest.authenticode && !trust::is_signed(&path) {
        return reject("installer-ul nu are o semnatura digitala valida");
    }

    let arguments = HSTRING::from(format!("/i \"{}\" /passive /norestart", path.display()));
    let result = unsafe { ShellExecuteW(None, w!("open"), w!("msiexec.exe"), &arguments, None, SW_SHOWNORMAL) };
    if result.0 as isize <= 32 {
        return reject("installer-ul nu a putut fi pornit");
    }
    Ok(())
}
