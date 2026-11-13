use std::fs;
use std::path::Path;

const MAX_FILE_SIZE: u64 = 1024 * 1024;

pub fn last_server(app_dir: &Path) -> Option<String> {
    let path = app_dir.join("data").join("cache").join("crashometry");
    if fs::metadata(&path).ok()?.len() > MAX_FILE_SIZE {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    last_value(&bytes, "last_server")
}

fn last_value(bytes: &[u8], wanted: &str) -> Option<String> {
    let mut found = None;
    let mut rest = bytes;
    while rest.len() >= 8 {
        let key_len = u32::from_le_bytes(rest[0..4].try_into().ok()?) as usize;
        let value_len = u32::from_le_bytes(rest[4..8].try_into().ok()?) as usize;
        let end = 8usize.checked_add(key_len)?.checked_add(value_len)?;
        if end > rest.len() {
            break;
        }
        let key = &rest[8..8 + key_len];
        let value = &rest[8 + key_len..end];
        if key == wanted.as_bytes() {
            found = String::from_utf8(value.to_vec()).ok();
        }
        rest = &rest[end..];
    }
    found
}

#[cfg(test)]
mod tests {
    use super::last_value;

    #[test]
    fn keeps_last_value_for_key() {
        let mut data = Vec::new();
        for (key, value) in [
            ("last_server", "1.1.1.1:1"),
            ("gpu", "x"),
            ("last_server", "82.22.174.206:30120"),
        ] {
            data.extend((key.len() as u32).to_le_bytes());
            data.extend((value.len() as u32).to_le_bytes());
            data.extend(key.as_bytes());
            data.extend(value.as_bytes());
        }
        assert_eq!(last_value(&data, "last_server").as_deref(), Some("82.22.174.206:30120"));
    }

    #[test]
    #[ignore]
    fn reads_real_cache() {
        let dir = std::env::var("A4O_TEST_FIVEM").expect("A4O_TEST_FIVEM");
        println!("last_server={:?}", super::last_server(dir.as_ref()));
    }
}
