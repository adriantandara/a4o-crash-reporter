use std::fs;
use std::path::Path;

use miniz_oxide::inflate::decompress_slice_iter_to_slice;

const END_OF_DIRECTORY: u32 = 0x0605_4b50;
const DIRECTORY_ENTRY: u32 = 0x0201_4b50;
const LOCAL_HEADER: u32 = 0x0403_4b50;
const STORED: u16 = 0;
const DEFLATED: u16 = 8;

pub struct Archive {
    data: Vec<u8>,
    entries: Vec<Entry>,
}

pub struct Entry {
    pub name: String,
    pub size: usize,
    method: u16,
    compressed_size: usize,
    header_offset: usize,
}

impl Archive {
    pub fn open(path: &Path, max_size: u64) -> Result<Self, String> {
        let size = fs::metadata(path).map_err(|e| e.to_string())?.len();
        if size > max_size {
            return Err("arhiva de crash este prea mare".into());
        }
        let data = fs::read(path).map_err(|e| e.to_string())?;
        let entries = read_directory(&data).ok_or("arhiva de crash este invalida")?;
        Ok(Self { data, entries })
    }

    pub fn find(&self, matches: impl Fn(&str) -> bool) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|entry| !entry.name.contains(['/', '\\']) && matches(&entry.name))
    }

    pub fn read(&self, entry: &Entry, limit: usize) -> Result<Vec<u8>, String> {
        if entry.size > limit {
            return Err(format!("{} depaseste limita de marime", entry.name));
        }
        let data = &self.data;
        let offset = entry.header_offset;
        if read_u32(data, offset) != Some(LOCAL_HEADER) {
            return Err(format!("{} are antetul invalid", entry.name));
        }
        let name_len = read_u16(data, offset + 26).ok_or("antet invalid")? as usize;
        let extra_len = read_u16(data, offset + 28).ok_or("antet invalid")? as usize;
        let start = offset + 30 + name_len + extra_len;
        let raw = data
            .get(start..start + entry.compressed_size)
            .ok_or("date lipsa in arhiva")?;

        match entry.method {
            STORED => Ok(raw.to_vec()),
            DEFLATED => {
                let mut output = vec![0u8; entry.size];
                let written = decompress_slice_iter_to_slice(&mut output, std::iter::once(raw), false, true)
                    .map_err(|_| format!("{} nu a putut fi dezarhivat", entry.name))?;
                output.truncate(written);
                Ok(output)
            }
            _ => Err(format!("{} foloseste o compresie necunoscuta", entry.name)),
        }
    }
}

fn read_directory(data: &[u8]) -> Option<Vec<Entry>> {
    let search_start = data.len().saturating_sub(22 + u16::MAX as usize);
    let end = (search_start..=data.len().checked_sub(22)?)
        .rev()
        .find(|&i| read_u32(data, i) == Some(END_OF_DIRECTORY))?;

    let count = read_u16(data, end + 10)? as usize;
    let mut cursor = read_u32(data, end + 16)? as usize;
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        if read_u32(data, cursor)? != DIRECTORY_ENTRY {
            return None;
        }
        let name_len = read_u16(data, cursor + 28)? as usize;
        let extra_len = read_u16(data, cursor + 30)? as usize;
        let comment_len = read_u16(data, cursor + 32)? as usize;
        let name = data.get(cursor + 46..cursor + 46 + name_len)?;
        entries.push(Entry {
            name: String::from_utf8_lossy(name).into_owned(),
            method: read_u16(data, cursor + 10)?,
            compressed_size: read_u32(data, cursor + 20)? as usize,
            size: read_u32(data, cursor + 24)? as usize,
            header_offset: read_u32(data, cursor + 42)? as usize,
        });
        cursor += 46 + name_len + extra_len + comment_len;
    }
    Some(entries)
}

fn read_u16(data: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(offset..offset + 2)?.try_into().ok()?))
}

fn read_u32(data: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(offset..offset + 4)?.try_into().ok()?))
}
