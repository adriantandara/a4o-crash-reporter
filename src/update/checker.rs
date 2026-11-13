use std::sync::Mutex;

use serde_json::Value;

use super::manifest::{self, ReleaseManifest};
use super::version;
use crate::config;
use crate::net::http;

static AVAILABLE: Mutex<Option<ReleaseManifest>> = Mutex::new(None);

pub fn available() -> Option<ReleaseManifest> {
    AVAILABLE.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn check() -> Result<Option<ReleaseManifest>, String> {
    let response = http::send(
        "GET",
        config::UPDATE_PATH,
        &[("Accept", "application/json".into())],
        &[],
    )
    .map_err(|e| e.0)?;
    if response.status != 200 {
        return Err(format!("serverul a raspuns cu {}", response.status));
    }
    let value: Value = serde_json::from_slice(&response.body).map_err(|_| "raspuns invalid de la server")?;
    let current = &value["current"];
    let found = if current.is_object() {
        let manifest = manifest::verify(
            current["manifest"].as_str().unwrap_or_default(),
            current["signature"].as_str().unwrap_or_default(),
        )?;
        version::is_newer(&manifest.version, config::APP_VERSION).then_some(manifest)
    } else {
        None
    };
    *AVAILABLE.lock().unwrap_or_else(|e| e.into_inner()) = found.clone();
    Ok(found)
}
