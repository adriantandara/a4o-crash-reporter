pub use target::*;

#[cfg(not(feature = "local"))]
mod target {
    pub const APP_NAME: &str = "ALL4ONE Crash Reporter";
    pub const DATA_FOLDER: &str = "CrashReporter";
    pub const API_HOST: &str = "ucp.all4one.ro";
    pub const API_PORT: u16 = 443;
    pub const API_SECURE: bool = true;
    pub const ALLOWED_SERVERS: &[&str] = &["82.22.174.206:30120", "82.22.174.206:30121"];
    pub const UPDATE_PATH: &str = "/crash-reporter/latest";
}

#[cfg(feature = "local")]
mod target {
    pub const APP_NAME: &str = "ALL4ONE Crash Reporter (test local)";
    pub const DATA_FOLDER: &str = "CrashReporterLocal";
    pub const API_HOST: &str = "127.0.0.1";
    pub const API_PORT: u16 = 4700;
    pub const API_SECURE: bool = false;
    pub const ALLOWED_SERVERS: &[&str] = &["127.0.0.1:30120"];
    pub const UPDATE_PATH: &str = "/api/public/crash-reporter/latest";
}

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const USER_AGENT: &str = concat!("A4O-CrashReporter/", env!("CARGO_PKG_VERSION"));

pub const API_PREFIX: &str = "/api/crash/v1";

pub const REPOSITORY_URL: &str = "https://github.com/adriantandara/a4o-crash-reporter";
pub const PRIVACY_URL: &str = "https://github.com/adriantandara/a4o-crash-reporter/blob/main/PRIVACY.md";

pub const UPDATES_URL: &str = "https://github.com/adriantandara/a4o-crash-reporter/blob/main/UPDATES.md";
pub const RELEASE_KEYS: [&str; 2] = [
    "XovA_mOeVHqYFtCC7zEbm6gzMKfaRnH_t5EAXw_c3_g",
    "X3iryo3lFBVLuH7V_e5L6ZD3Ngd_06_Oy2P0JCB9mhc",
];

pub const CONSENT_VERSION: u64 = 1;

pub const MAX_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;
pub const MAX_DUMP_BYTES: usize = 96 * 1024 * 1024;
pub const MAX_LOG_BYTES: usize = 8 * 1024 * 1024;

pub fn is_allowed_server(address: &str) -> bool {
    let address = address.trim().trim_end_matches('/');
    let address = address
        .strip_prefix("http://")
        .or_else(|| address.strip_prefix("https://"))
        .unwrap_or(address);
    ALLOWED_SERVERS
        .iter()
        .any(|allowed| address.eq_ignore_ascii_case(allowed))
}
