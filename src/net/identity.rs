use std::fs;
use std::path::PathBuf;

use ed25519_dalek::{Signer, SigningKey};
use windows::Win32::Foundation::{HLOCAL, LocalFree};
use windows::Win32::Security::Cryptography::{
    BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom, CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData,
    CryptUnprotectData,
};
use windows::core::PCWSTR;

use super::encoding::base64_url;
use crate::storage::paths;

const ENTROPY: &[u8] = b"ALL4ONE.CrashReporter.DeviceKey.v1";

pub struct DeviceKey {
    signing_key: SigningKey,
}

impl DeviceKey {
    pub fn load_or_create() -> Result<Self, String> {
        let seed = match fs::read(file()).ok().and_then(|data| unprotect(&data)) {
            Some(seed) if seed.len() == 32 => seed,
            _ => create()?,
        };
        let seed: [u8; 32] = seed.try_into().map_err(|_| "cheie invalida")?;
        Ok(Self {
            signing_key: SigningKey::from_bytes(&seed),
        })
    }

    pub fn public_key(&self) -> String {
        base64_url(self.signing_key.verifying_key().as_bytes())
    }

    pub fn sign(&self, message: &[u8]) -> String {
        base64_url(&self.signing_key.sign(message).to_bytes())
    }
}

pub fn random_bytes<const N: usize>() -> Result<[u8; N], String> {
    let mut bytes = [0u8; N];
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(|e| e.message())?;
    Ok(bytes)
}

fn file() -> PathBuf {
    paths::data_dir().join("device.key")
}

fn create() -> Result<Vec<u8>, String> {
    let seed = random_bytes::<32>()?;
    let protected = protect(&seed).ok_or("cheia dispozitivului nu a putut fi protejata")?;
    paths::ensure_data_dir();
    fs::write(file(), protected).map_err(|e| e.to_string())?;
    Ok(seed.to_vec())
}

fn protect(data: &[u8]) -> Option<Vec<u8>> {
    transform(data, true)
}

fn unprotect(data: &[u8]) -> Option<Vec<u8>> {
    transform(data, false)
}

fn transform(data: &[u8], encrypt: bool) -> Option<Vec<u8>> {
    let input = blob(data);
    let entropy = blob(ENTROPY);
    let mut output = CRYPT_INTEGER_BLOB::default();
    unsafe {
        let result = if encrypt {
            CryptProtectData(
                &input,
                PCWSTR::null(),
                Some(&entropy),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                None,
                Some(&entropy),
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        };
        result.ok()?;
        let bytes = std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(output.pbData as *mut _)));
        Some(bytes)
    }
}

fn blob(data: &[u8]) -> CRYPT_INTEGER_BLOB {
    CRYPT_INTEGER_BLOB {
        cbData: data.len() as u32,
        pbData: data.as_ptr() as *mut u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_matches_server_vector() {
        let key = DeviceKey {
            signing_key: SigningKey::from_bytes(&[7u8; 32]),
        };
        let body_hash = "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a";
        let payload =
            format!("A4OCR1\nPOST\n/api/crash/v1/claim\n1794510651\n00112233445566778899aabbccddeeff\n{body_hash}");
        assert_eq!(key.public_key(), "6kpsY-KcUgq-9VB7Ey7F-ZVHdq6-vnuSQh7qaRRG0iw");
        assert_eq!(
            key.sign(payload.as_bytes()),
            "b-hA2prXUd5B4C09qNYkTbzIDjoWDZ7AlF-36GX17daIO4_lY05hURcdFFW25akkrD4UdWbYFjQzdaSlTzufCg"
        );
    }
}
