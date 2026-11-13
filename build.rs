use std::env;
use std::fs;
use std::path::PathBuf;

const RT_ICON: u16 = 3;
const RT_GROUP_ICON: u16 = 14;
const RT_VERSION: u16 = 16;
const RT_MANIFEST: u16 = 24;
const LANG_EN_US: u16 = 0x0409;

fn main() {
    println!("cargo:rerun-if-changed=assets/a4o.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let icon = fs::read("assets/a4o.ico").expect("assets/a4o.ico");
    let manifest = fs::read("assets/app.manifest").expect("assets/app.manifest");

    let mut res = ResourceFile::new();
    let mut group = icon[0..6].to_vec();
    for (index, entry) in icon[6..].chunks(16).take(read_u16(&icon, 4) as usize).enumerate() {
        let id = index as u16 + 1;
        let size = read_u32(entry, 8) as usize;
        let offset = read_u32(entry, 12) as usize;
        res.add(RT_ICON, id, &icon[offset..offset + size]);
        group.extend(&entry[0..12]);
        group.extend(id.to_le_bytes());
    }
    res.add(RT_GROUP_ICON, 1, &group);
    res.add(RT_MANIFEST, 1, &manifest);
    res.add(RT_VERSION, 1, &version_info());

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("resources.res");
    fs::write(&out, res.bytes).unwrap();
    println!("cargo:rustc-link-arg-bins={}", out.display());
}

struct ResourceFile {
    bytes: Vec<u8>,
}

impl ResourceFile {
    fn new() -> Self {
        let mut file = Self { bytes: Vec::new() };
        file.add(0, 0, &[]);
        file
    }

    fn add(&mut self, kind: u16, id: u16, data: &[u8]) {
        let lang = if kind == 0 { 0 } else { LANG_EN_US };
        let flags: u16 = if kind == 0 { 0 } else { 0x1030 };
        let b = &mut self.bytes;
        b.extend((data.len() as u32).to_le_bytes());
        b.extend(32u32.to_le_bytes());
        b.extend([0xFF, 0xFF]);
        b.extend(kind.to_le_bytes());
        b.extend([0xFF, 0xFF]);
        b.extend(id.to_le_bytes());
        b.extend(0u32.to_le_bytes());
        b.extend(flags.to_le_bytes());
        b.extend(lang.to_le_bytes());
        b.extend(0u32.to_le_bytes());
        b.extend(0u32.to_le_bytes());
        b.extend(data);
        pad(b);
    }
}

fn pad(bytes: &mut Vec<u8>) {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn utf16(text: &str) -> Vec<u8> {
    text.encode_utf16().chain([0]).flat_map(u16::to_le_bytes).collect()
}

fn version_node(key: &str, value: &[u8], value_len: u16, is_text: bool, children: &[Vec<u8>]) -> Vec<u8> {
    let mut b = vec![0, 0];
    b.extend(value_len.to_le_bytes());
    b.extend((is_text as u16).to_le_bytes());
    b.extend(utf16(key));
    pad(&mut b);
    b.extend(value);
    for child in children {
        pad(&mut b);
        b.extend(child);
    }
    let len = b.len() as u16;
    b[0..2].copy_from_slice(&len.to_le_bytes());
    b
}

fn version_info() -> Vec<u8> {
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    let parts: Vec<u16> = version.split('.').map(|p| p.parse().unwrap_or(0)).collect();
    let (major, minor, patch) = (parts[0] as u32, parts[1] as u32, parts[2] as u32);
    let packed_high = (major << 16) | minor;
    let packed_low = patch << 16;

    let mut fixed = Vec::new();
    for value in [
        0xFEEF04BDu32,
        0x00010000,
        packed_high,
        packed_low,
        packed_high,
        packed_low,
        0x3F,
        0,
        0x00040004,
        1,
        0,
        0,
        0,
    ] {
        fixed.extend(value.to_le_bytes());
    }

    let strings = [
        ("CompanyName", "ALL4ONE Romania"),
        ("FileDescription", "ALL4ONE Crash Reporter"),
        ("FileVersion", version.as_str()),
        ("InternalName", "a4o-crash-reporter"),
        ("LegalCopyright", "MIT License, ALL4ONE Romania"),
        ("OriginalFilename", "a4o-crash-reporter.exe"),
        ("ProductName", "ALL4ONE Crash Reporter"),
        ("ProductVersion", version.as_str()),
    ];
    let entries: Vec<Vec<u8>> = strings
        .iter()
        .map(|(key, value)| {
            let text = utf16(value);
            version_node(key, &text, (text.len() / 2) as u16, true, &[])
        })
        .collect();
    let table = version_node("040904B0", &[], 0, true, &entries);
    let string_info = version_node("StringFileInfo", &[], 0, true, &[table]);

    let mut translation = Vec::new();
    translation.extend(LANG_EN_US.to_le_bytes());
    translation.extend(1200u16.to_le_bytes());
    let var = version_node("Translation", &translation, 4, false, &[]);
    let var_info = version_node("VarFileInfo", &[], 0, true, &[var]);

    version_node("VS_VERSION_INFO", &fixed, 52, false, &[string_info, var_info])
}
