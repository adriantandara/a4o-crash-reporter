use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde_json::json;
use sha2::{Digest, Sha256};

const PRODUCT: &str = "a4o-crash-reporter";
const FILE_PREFIX: &str = "ALL4ONE-Crash-Reporter";
const MANIFEST: &str = "release.json";
const SIGNATURE: &str = "release.json.sig";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
        ["keygen", path] => keygen(Path::new(path)),
        ["sign", key, version, dir] => sign(Path::new(key), version, Path::new(dir), false),
        ["sign", key, version, dir, "--authenticode"] => sign(Path::new(key), version, Path::new(dir), true),
        ["verify", public_key, dir] => verify(public_key, Path::new(dir)),
        _ => Err(usage()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    [
        "release-sign keygen <fisier-cheie>",
        "release-sign sign <fisier-cheie> <versiune> <folder-release> [--authenticode]",
        "release-sign verify <cheie-publica> <folder-release>",
    ]
    .join("\n")
}

fn keygen(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Err(format!("{} exista deja", path.display()));
    }
    let mut seed = [0u8; 32];
    getrandom::getrandom(&mut seed).map_err(|e| e.to_string())?;
    let key = SigningKey::from_bytes(&seed);
    write_private(path, &hex(&seed))?;
    println!("{}", base64_url(key.verifying_key().as_bytes()));
    Ok(())
}

fn sign(key_path: &Path, version: &str, dir: &Path, authenticode: bool) -> Result<(), String> {
    let key = load_key(key_path)?;
    let mut files = serde_json::Map::new();
    for (kind, extension) in [("msi", "msi"), ("exe", "exe")] {
        let name = format!("{FILE_PREFIX}-{version}.{extension}");
        let bytes = fs::read(dir.join(&name)).map_err(|e| format!("{name}: {e}"))?;
        files.insert(
            kind.into(),
            json!({ "name": name, "size": bytes.len(), "sha256": hex(&Sha256::digest(&bytes)) }),
        );
    }
    let manifest = json!({
        "product": PRODUCT,
        "version": version,
        "files": files,
        "authenticode": authenticode,
    });
    let bytes = serde_json::to_vec_pretty(&manifest).map_err(|e| e.to_string())?;
    let signature = key.sign(&bytes);
    fs::write(dir.join(MANIFEST), &bytes).map_err(|e| e.to_string())?;
    fs::write(dir.join(SIGNATURE), base64_url(&signature.to_bytes())).map_err(|e| e.to_string())?;
    println!("{}", String::from_utf8_lossy(&bytes));
    println!("semnat cu {}", base64_url(key.verifying_key().as_bytes()));
    Ok(())
}

fn verify(public_key: &str, dir: &Path) -> Result<(), String> {
    let public = decode_base64_url(public_key).ok_or("cheie publica invalida")?;
    let public: [u8; 32] = public.try_into().map_err(|_| "cheie publica invalida")?;
    let key = VerifyingKey::from_bytes(&public).map_err(|e| e.to_string())?;
    let manifest = fs::read(dir.join(MANIFEST)).map_err(|e| e.to_string())?;
    let signature = fs::read_to_string(dir.join(SIGNATURE)).map_err(|e| e.to_string())?;
    let signature = decode_base64_url(signature.trim()).ok_or("semnatura invalida")?;
    let signature = Signature::from_slice(&signature).map_err(|e| e.to_string())?;
    key.verify(&manifest, &signature)
        .map_err(|_| "semnatura NU corespunde".to_string())?;
    println!("semnatura valida");
    Ok(())
}

fn load_key(path: &Path) -> Result<SigningKey, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let seed = unhex(text.trim()).ok_or("cheie privata invalida")?;
    let seed: [u8; 32] = seed.try_into().map_err(|_| "cheie privata invalida")?;
    Ok(SigningKey::from_bytes(&seed))
}

fn write_private(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, contents).map_err(|e| e.to_string())?;
    restrict(path.to_path_buf());
    Ok(())
}

#[cfg(unix)]
fn restrict(path: PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict(_path: PathBuf) {}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn base64_url(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n =
            (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        for i in 0..chunk.len() + 1 {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

fn decode_base64_url(text: &str) -> Option<Vec<u8>> {
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for c in text.bytes() {
        let value = ALPHABET.iter().position(|&a| a == c)? as u32;
        bits = bits << 6 | value;
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push((bits >> count) as u8);
        }
    }
    Some(out)
}
