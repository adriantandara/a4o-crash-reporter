use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::Value;

use super::version;
use crate::config;
use crate::net::encoding::decode_base64;

const PRODUCT: &str = "a4o-crash-reporter";

#[derive(Clone)]
pub struct ReleaseManifest {
    pub version: String,
    pub installer_name: String,
    pub installer_size: u64,
    pub installer_sha256: String,
    pub authenticode: bool,
}

impl ReleaseManifest {
    pub fn installer_url(&self) -> String {
        format!(
            "{}/releases/download/v{}/{}",
            config::REPOSITORY_URL,
            self.version,
            self.installer_name
        )
    }

    pub fn notes_url(&self) -> String {
        format!("{}/releases/tag/v{}", config::REPOSITORY_URL, self.version)
    }
}

pub fn verify(manifest_base64: &str, signature_base64: &str) -> Result<ReleaseManifest, String> {
    let manifest = decode_base64(manifest_base64).ok_or("manifest invalid")?;
    let signature = decode_base64(signature_base64).ok_or("semnatura invalida")?;
    let signature = Signature::from_slice(&signature).map_err(|_| "semnatura invalida")?;

    let trusted = config::RELEASE_KEYS.iter().any(|key| {
        decode_base64(key)
            .and_then(|bytes| <[u8; 32]>::try_from(bytes).ok())
            .and_then(|bytes| VerifyingKey::from_bytes(&bytes).ok())
            .is_some_and(|key| key.verify(&manifest, &signature).is_ok())
    });
    if !trusted {
        return Err("semnatura actualizarii nu este valida".into());
    }

    parse(&manifest)
}

fn parse(bytes: &[u8]) -> Result<ReleaseManifest, String> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| "manifest invalid")?;
    let version = value["version"].as_str().unwrap_or_default().to_owned();
    if value["product"] != PRODUCT || version::parse(&version).is_none() {
        return Err("manifestul nu este pentru aceasta aplicatie".into());
    }

    let installer = &value["files"]["msi"];
    let name = installer["name"].as_str().unwrap_or_default();
    let sha256 = installer["sha256"].as_str().unwrap_or_default();
    let size = installer["size"].as_u64().unwrap_or(0);
    let expected_name = format!("ALL4ONE-Crash-Reporter-{version}.msi");
    if name != expected_name || sha256.len() != 64 || !sha256.bytes().all(|b| b.is_ascii_hexdigit()) || size == 0 {
        return Err("manifestul are date invalide pentru installer".into());
    }

    Ok(ReleaseManifest {
        installer_name: expected_name,
        installer_size: size,
        installer_sha256: sha256.to_ascii_lowercase(),
        authenticode: value["authenticode"].as_bool().unwrap_or(false),
        version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"{"product":"a4o-crash-reporter","version":"0.2.0","files":{"msi":{"name":"ALL4ONE-Crash-Reporter-0.2.0.msi","size":10,"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}},"authenticode":false}"#;

    #[test]
    fn parses_valid_manifest() {
        let manifest = parse(MANIFEST.as_bytes()).unwrap();
        assert_eq!(manifest.version, "0.2.0");
        assert_eq!(
            manifest.installer_url(),
            "https://github.com/adriantandara/a4o-crash-reporter/releases/download/v0.2.0/ALL4ONE-Crash-Reporter-0.2.0.msi"
        );
    }

    #[test]
    fn rejects_foreign_installer_name() {
        let tampered = MANIFEST.replace("ALL4ONE-Crash-Reporter-0.2.0.msi", "../evil.msi");
        assert!(parse(tampered.as_bytes()).is_err());
    }

    #[test]
    fn rejects_unsigned_manifest() {
        let signature = crate::net::encoding::base64_url(&[0u8; 64]);
        let manifest = crate::net::encoding::base64_url(MANIFEST.as_bytes());
        assert!(verify(&manifest, &signature).is_err());
    }
}

#[cfg(test)]
mod integration {
    use super::verify;

    #[test]
    #[ignore]
    fn verifies_guard_response() {
        let path = std::env::var("A4O_TEST_LATEST").expect("A4O_TEST_LATEST");
        let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let current = &value["current"];
        let manifest = verify(
            current["manifest"].as_str().unwrap(),
            current["signature"].as_str().unwrap(),
        )
        .unwrap();
        println!(
            "verificat: {} {} {} bytes",
            manifest.version,
            manifest.installer_url(),
            manifest.installer_size
        );

        let mut tampered = current["manifest"].as_str().unwrap().to_owned();
        tampered.replace_range(10..11, if &tampered[10..11] == "A" { "B" } else { "A" });
        assert!(verify(&tampered, current["signature"].as_str().unwrap()).is_err());
    }

    #[test]
    #[ignore]
    fn downloads_from_github() {
        let mut bytes = Vec::new();
        crate::net::http::download(
            "https://github.com/adriantandara/a4o-crash-reporter/raw/main/LICENSE",
            |chunk| {
                bytes.extend_from_slice(chunk);
                Ok(())
            },
        )
        .unwrap_or_else(|e| panic!("{}", e.0));
        println!("descarcat {} bytes", bytes.len());
        assert!(bytes.starts_with(b"MIT License"));
    }
}
