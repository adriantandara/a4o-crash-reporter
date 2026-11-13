use std::ffi::c_void;
use std::ptr;

use windows::Win32::Networking::WinHttp::{
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2,
    WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3, WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_OPTION_SECURE_PROTOCOLS,
    WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE, WinHttpCloseHandle, WinHttpConnect, WinHttpOpen,
    WinHttpOpenRequest, WinHttpQueryHeaders, WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest,
    WinHttpSetOption, WinHttpSetTimeouts, WinHttpWriteData,
};
use windows::core::{HSTRING, PCWSTR};

use crate::config;

const HTTPS_PORT: u16 = 443;

struct Endpoint<'a> {
    host: &'a str,
    port: u16,
    secure: bool,
}
const CHUNK_SIZE: usize = 64 * 1024;
const MAX_RESPONSE_BYTES: usize = 256 * 1024;

pub struct Response {
    pub status: u32,
    pub body: Vec<u8>,
}

pub struct RequestError(pub String);

pub fn send(method: &str, path: &str, headers: &[(&str, String)], body: &[u8]) -> Result<Response, RequestError> {
    let api = Endpoint {
        host: config::API_HOST,
        port: config::API_PORT,
        secure: config::API_SECURE,
    };
    let exchange = Exchange::start(&api, method, path, headers, body)?;
    let status = exchange.status()?;
    let mut body = Vec::new();
    exchange.read(|chunk| {
        if body.len() + chunk.len() > MAX_RESPONSE_BYTES {
            return Err("raspunsul serverului este prea mare".into());
        }
        body.extend_from_slice(chunk);
        Ok(())
    })?;
    Ok(Response { status, body })
}

pub fn download(url: &str, mut sink: impl FnMut(&[u8]) -> Result<(), String>) -> Result<(), RequestError> {
    let rest = url
        .strip_prefix("https://")
        .ok_or_else(|| RequestError("doar adrese HTTPS".into()))?;
    let (host, path) = rest.split_at(rest.find('/').unwrap_or(rest.len()));
    let endpoint = Endpoint {
        host,
        port: HTTPS_PORT,
        secure: true,
    };
    let exchange = Exchange::start(&endpoint, "GET", if path.is_empty() { "/" } else { path }, &[], &[])?;
    let status = exchange.status()?;
    if status != 200 {
        return Err(RequestError(format!("descarcarea a raspuns cu {status}")));
    }
    exchange.read(&mut sink)
}

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

struct Exchange {
    request: Handle,
    _connection: Handle,
    _session: Handle,
}

impl Exchange {
    fn start(
        endpoint: &Endpoint,
        method: &str,
        path: &str,
        headers: &[(&str, String)],
        body: &[u8],
    ) -> Result<Self, RequestError> {
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
                WinHttpConnect(session.0, &HSTRING::from(endpoint.host), endpoint.port, 0),
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
                    if endpoint.secure {
                        WINHTTP_FLAG_SECURE
                    } else {
                        WINHTTP_OPEN_REQUEST_FLAGS(0)
                    },
                ),
                "cerere",
            )?;

            let header_text: String = headers
                .iter()
                .map(|(name, value)| format!("{name}: {value}\r\n"))
                .collect();
            let header_wide: Vec<u16> = header_text.encode_utf16().collect();
            let header_arg = (!header_wide.is_empty()).then_some(header_wide.as_slice());
            WinHttpSendRequest(request.0, header_arg, None, 0, body.len() as u32, 0).map_err(fail("trimitere"))?;

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

            Ok(Self {
                request,
                _connection: connection,
                _session: session,
            })
        }
    }

    fn status(&self) -> Result<u32, RequestError> {
        let mut status = 0u32;
        let mut size = size_of::<u32>() as u32;
        unsafe {
            WinHttpQueryHeaders(
                self.request.0,
                WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
                PCWSTR::null(),
                Some(&mut status as *mut u32 as *mut c_void),
                &mut size,
                ptr::null_mut(),
            )
        }
        .map_err(fail("status"))?;
        Ok(status)
    }

    fn read(&self, mut sink: impl FnMut(&[u8]) -> Result<(), String>) -> Result<(), RequestError> {
        let mut buffer = vec![0u8; CHUNK_SIZE];
        loop {
            let mut read = 0;
            unsafe {
                WinHttpReadData(
                    self.request.0,
                    buffer.as_mut_ptr() as *mut c_void,
                    buffer.len() as u32,
                    &mut read,
                )
            }
            .map_err(fail("citire"))?;
            if read == 0 {
                return Ok(());
            }
            sink(&buffer[..read as usize]).map_err(RequestError)?;
        }
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
