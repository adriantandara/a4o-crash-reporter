pub const APP_NAME: &str = "ALL4ONE Crash Reporter";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const USER_AGENT: &str = concat!("A4O-CrashReporter/", env!("CARGO_PKG_VERSION"));

pub const API_HOST: &str = "ucp.all4one.ro";
pub const API_PORT: u16 = 443;
pub const API_PREFIX: &str = "/api/crash/v1";

pub const ALLOWED_SERVERS: &[&str] = &["82.22.174.206:30120", "82.22.174.206:30121"];

pub const REPOSITORY_URL: &str = "https://github.com/adriantandara/a4o-crash-reporter";
pub const PRIVACY_URL: &str = "https://github.com/adriantandara/a4o-crash-reporter/blob/main/PRIVACY.md";

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
