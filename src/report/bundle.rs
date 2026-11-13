use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

use miniz_oxide::deflate::compress_to_vec_zlib;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use super::archive::Archive;
use super::redact::redact_user_paths;
use crate::config;
use crate::net::encoding::hex;

const COMPRESSION_LEVEL: u8 = 6;
const MAX_METADATA_BYTES: usize = 64 * 1024;

pub struct Bundle {
    pub server: String,
    pub crash_hash: String,
    pub crash_key: String,
    pub crashed_at: u64,
    pub metadata: Map<String, Value>,
    pub dump: Vec<u8>,
    pub dump_size: usize,
    pub dump_sha256: String,
    pub log: Option<Vec<u8>>,
}

impl Bundle {
    pub fn from_archive(path: &Path) -> Result<Self, String> {
        let archive = Archive::open(path, config::MAX_ARCHIVE_BYTES)?;

        let metadata_entry = archive
            .find(|name| name == "crashometry.json")
            .ok_or("arhiva nu contine crashometry.json")?;
        let metadata = clean_metadata(&archive.read(metadata_entry, MAX_METADATA_BYTES)?)?;

        let server = text_field(&metadata, "crashometry_last_server");
        if !config::is_allowed_server(&server) {
            return Err(format!("crash-ul este de pe alt server ({server})"));
        }

        let dump_entry = archive
            .find(|name| name.to_ascii_lowercase().ends_with(".dmp"))
            .ok_or("arhiva nu contine fisierul .dmp")?;
        let dump = archive.read(dump_entry, config::MAX_DUMP_BYTES)?;
        if !dump.starts_with(b"MDMP") {
            return Err("fisierul .dmp nu este un minidump valid".into());
        }

        let log = archive
            .find(|name| name.starts_with("CitizenFX_log") && name.ends_with(".log"))
            .and_then(|entry| archive.read(entry, config::MAX_LOG_BYTES).ok())
            .map(|bytes| redact_user_paths(&String::from_utf8_lossy(&bytes)))
            .map(|text| compress_to_vec_zlib(text.as_bytes(), COMPRESSION_LEVEL));

        Ok(Self {
            crash_hash: text_field(&metadata, "crash_hash"),
            crash_key: text_field(&metadata, "crash_hash_key"),
            crashed_at: modified_unix(path),
            server,
            metadata,
            dump_size: dump.len(),
            dump_sha256: hex(&Sha256::digest(&dump)),
            dump: compress_to_vec_zlib(&dump, COMPRESSION_LEVEL),
            log,
        })
    }

    pub fn label(&self) -> &str {
        if self.crash_hash.is_empty() {
            "crash necunoscut"
        } else {
            &self.crash_hash
        }
    }
}

fn clean_metadata(bytes: &[u8]) -> Result<Map<String, Value>, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "crashometry.json este invalid")?;
    let mut map = value.as_object().cloned().ok_or("crashometry.json este invalid")?;
    map.retain(|key, value| !key.to_ascii_lowercase().contains("rockstar") && value.is_string());
    Ok(map)
}

fn text_field(map: &Map<String, Value>, key: &str) -> String {
    map.get(key).and_then(Value::as_str).unwrap_or_default().to_owned()
}

fn modified_unix(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or_else(crate::system::time::unix_now)
}

#[cfg(test)]
mod tests {
    use super::Bundle;

    #[test]
    #[ignore]
    fn reads_real_archive() {
        let path = std::env::var("A4O_TEST_ARCHIVE").expect("A4O_TEST_ARCHIVE");
        let bundle = Bundle::from_archive(path.as_ref()).unwrap();
        println!(
            "server={} hash={} key={}",
            bundle.server, bundle.crash_hash, bundle.crash_key
        );
        println!(
            "dump={} -> {} bytes, sha256={}",
            bundle.dump_size,
            bundle.dump.len(),
            bundle.dump_sha256
        );
        println!("log={:?} bytes", bundle.log.as_ref().map(Vec::len));
        println!("metadata keys={:?}", bundle.metadata.keys().collect::<Vec<_>>());
        assert!(!bundle.metadata.keys().any(|k| k.to_lowercase().contains("rockstar")));
    }
}
