use std::thread;
use std::time::Duration;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::encoding::hex;
use super::http::{self, Response};
use super::identity::{DeviceKey, random_bytes};
use crate::config;
use crate::report::bundle::Bundle;
use crate::system::time::unix_now;

const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(15), Duration::from_secs(60)];
const SIGNATURE_VERSION: &str = "A4OCR1";

pub enum Submitted {
    New(String),
    Duplicate,
}

enum Failure {
    Temporary(String),
    Final(String),
}

pub fn submit(bundle: &Bundle) -> Result<Submitted, String> {
    let key = DeviceKey::load_or_create()?;
    let mut delays = RETRY_DELAYS.iter();
    loop {
        match try_submit(&key, bundle) {
            Ok(submitted) => return Ok(submitted),
            Err(Failure::Final(message)) => return Err(message),
            Err(Failure::Temporary(message)) => match delays.next() {
                Some(delay) => thread::sleep(*delay),
                None => return Err(message),
            },
        }
    }
}

pub fn forget_device() {
    if let Ok(key) = DeviceKey::load_or_create() {
        let _ = call(
            &key,
            "POST",
            &endpoint("/device/forget"),
            "application/json",
            b"{}",
            None,
        );
    }
}

fn try_submit(key: &DeviceKey, bundle: &Bundle) -> Result<Submitted, Failure> {
    let claim = json!({
        "app_version": config::APP_VERSION,
        "server": bundle.server,
        "crash_hash": bundle.crash_hash,
        "crash_key": bundle.crash_key,
        "crashed_at": bundle.crashed_at,
        "dump_size": bundle.dump_size,
        "dump_sha256": bundle.dump_sha256,
        "has_log": bundle.log.is_some(),
        "metadata": bundle.metadata,
    });
    let body = serde_json::to_vec(&claim).map_err(|e| Failure::Final(e.to_string()))?;
    let response = call(key, "POST", &endpoint("/claim"), "application/json", &body, None)?;

    match response.status {
        200 | 201 => {}
        409 => return Ok(Submitted::Duplicate),
        _ => return Err(explain(&response)),
    }

    let answer: Value =
        serde_json::from_slice(&response.body).map_err(|_| Failure::Final("raspuns invalid de la server".into()))?;
    let report_id = answer["report_id"].as_str().filter(|id| is_safe_id(id));
    let token = answer["upload_token"].as_str();
    let (Some(report_id), Some(token)) = (report_id, token) else {
        return Err(Failure::Final("raspuns invalid de la server".into()));
    };

    let report_path = endpoint(&format!("/reports/{report_id}"));
    upload(key, &format!("{report_path}/dump"), &bundle.dump, token)?;
    if let Some(log) = &bundle.log {
        upload(key, &format!("{report_path}/log"), log, token)?;
    }
    let done = call(
        key,
        "POST",
        &format!("{report_path}/complete"),
        "application/json",
        b"{}",
        Some(token),
    )?;
    if done.status != 200 {
        return Err(explain(&done));
    }
    Ok(Submitted::New(report_id.to_owned()))
}

fn upload(key: &DeviceKey, path: &str, data: &[u8], token: &str) -> Result<(), Failure> {
    let response = call(key, "PUT", path, "application/zlib", data, Some(token))?;
    match response.status {
        200 | 201 | 204 => Ok(()),
        _ => Err(explain(&response)),
    }
}

fn call(
    key: &DeviceKey,
    method: &str,
    path: &str,
    content_type: &str,
    body: &[u8],
    token: Option<&str>,
) -> Result<Response, Failure> {
    let timestamp = unix_now().to_string();
    let nonce = hex(&random_bytes::<16>().map_err(Failure::Final)?);
    let payload = format!(
        "{SIGNATURE_VERSION}\n{method}\n{path}\n{timestamp}\n{nonce}\n{}",
        hex(&Sha256::digest(body))
    );

    let mut headers = vec![
        ("Content-Type", content_type.to_owned()),
        ("X-A4O-Key", key.public_key()),
        ("X-A4O-Timestamp", timestamp),
        ("X-A4O-Nonce", nonce),
        ("X-A4O-Signature", key.sign(payload.as_bytes())),
        ("X-A4O-Version", config::APP_VERSION.to_owned()),
    ];
    if let Some(token) = token {
        headers.push(("Authorization", format!("Bearer {token}")));
    }

    let response = http::send(method, path, &headers, body).map_err(|e| Failure::Temporary(e.0))?;
    if response.status >= 500 {
        return Err(Failure::Temporary(format!("serverul a raspuns cu {}", response.status)));
    }
    Ok(response)
}

fn explain(response: &Response) -> Failure {
    let message = match response.status {
        401 => "semnatura cererii a fost respinsa (verifica ora din Windows)",
        403 => "acest PC nu mai poate trimite rapoarte",
        404 => "serverul nu a gasit o deconectare recenta cu crash pentru tine",
        410 => "timpul pentru trimiterea raportului a expirat",
        413 => "raportul este prea mare",
        429 => "prea multe rapoarte trimise, incearca mai tarziu",
        _ => "raportul a fost respins de server",
    };
    Failure::Final(message.into())
}

fn endpoint(path: &str) -> String {
    format!("{}{path}", config::API_PREFIX)
}

fn is_safe_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
