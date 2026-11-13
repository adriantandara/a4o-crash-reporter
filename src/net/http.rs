use std::ffi::c_void;
use std::ptr;

use windows::Win32::Networking::WinHttp::{
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2,
    WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3, WINHTTP_OPTION_SECURE_PROTOCOLS, WINHTTP_QUERY_FLAG_NUMBER,
    WINHTTP_QUERY_STATUS_CODE, WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
    WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetOption,
    WinHttpSetTimeouts, WinHttpWriteData,
};
use windows::core::{HSTRING, PCWSTR};

use crate::config;

const CHUNK_SIZE: usize = 64 * 1024;
const MAX_RESPONSE_BYTES: usize = 256 * 1024;

pub struct Response {
    pub status: u32,
    pub body: Vec<u8>,
}

pub struct RequestError(pub String);

struct Handle(*mut c_void);

impl Handle {
    fn new(raw: *mut c_void, step: &str) -> Result<Self, RequestError> {
        if raw.is_null() {
            Err(RequestError(format!(
                "{step}: {}",
                windows::core::Error::from_thread().message()
            )))
        } else {
            Ok(Self(raw))
        }
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = WinHttpCloseHandle(self.0);
        }
    }
}

pub fn send(method: &str, path: &str, headers: &[(&str, String)], body: &[u8]) -> Result<Response, RequestError> {
    unsafe {
        let session = Handle::new(
            WinHttpOpen(
                &HSTRING::from(config::USER_AGENT),
                WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
                PCWSTR::null(),
                PCWSTR::null(),
                0,
            ),
            "deschidere sesiune",
        )?;
        restrict_protocols(&session);
        WinHttpSetTimeouts(session.0, 10_000, 15_000, 60_000, 60_000).map_err(fail("timeout"))?;

        let connection = Handle::new(
            WinHttpConnect(session.0, &HSTRING::from(config::API_HOST), config::API_PORT, 0),
            "conectare",
        )?;
        let request = Handle::new(
            WinHttpOpenRequest(
                connection.0,
                &HSTRING::from(method),
                &HSTRING::from(path),
                PCWSTR::null(),
                PCWSTR::null(),
                ptr::null(),
                WINHTTP_FLAG_SECURE,
            ),
            "cerere",
        )?;

        let header_text: String = headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}\r\n"))
            .collect();
        let header_wide: Vec<u16> = header_text.encode_utf16().collect();
        WinHttpSendRequest(request.0, Some(&header_wide), None, 0, body.len() as u32, 0).map_err(fail("trimitere"))?;

        for chunk in body.chunks(CHUNK_SIZE) {
            let mut written = 0;
            WinHttpWriteData(
                request.0,
                Some(chunk.as_ptr() as *const c_void),
                chunk.len() as u32,
                &mut written,
            )
            .map_err(fail("upload"))?;
        }
        WinHttpReceiveResponse(request.0, ptr::null_mut()).map_err(fail("raspuns"))?;

        let mut status = 0u32;
        let mut status_size = size_of::<u32>() as u32;
        WinHttpQueryHeaders(
            request.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            Some(&mut status as *mut u32 as *mut c_void),
            &mut status_size,
            ptr::null_mut(),
        )
        .map_err(fail("status"))?;

        let mut body = Vec::new();
        let mut buffer = vec![0u8; 8 * 1024];
        loop {
            let mut read = 0;
            WinHttpReadData(
                request.0,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len() as u32,
                &mut read,
            )
            .map_err(fail("citire"))?;
            if read == 0 || body.len() >= MAX_RESPONSE_BYTES {
                break;
            }
            body.extend_from_slice(&buffer[..read as usize]);
        }

        Ok(Response { status, body })
    }
}

fn fail(step: &'static str) -> impl Fn(windows::core::Error) -> RequestError {
    move |error| RequestError(format!("{step}: {}", error.message()))
}

unsafe fn restrict_protocols(session: &Handle) {
    for protocols in [
        WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2 | WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3,
        WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2,
    ] {
        let value = protocols.to_le_bytes();
        if unsafe { WinHttpSetOption(Some(session.0), WINHTTP_OPTION_SECURE_PROTOCOLS, Some(&value)) }.is_ok() {
            return;
        }
    }
}
