mod auth;
mod client;
mod engine;
mod error;
mod http;
mod outbox;
mod time;
mod types;
mod vault;
pub mod view;

#[cfg(any(test, feature = "testing"))]
pub mod fake;
#[cfg(test)]
mod tests;

pub use auth::{Code, Session, SignUp};
pub use client::{Client, Device};
pub use engine::{Engine, Event, Heartbeat, Mood, SyncState, Worker};
pub use error::CloudError;
pub use http::{Reply, Request, Transport, Ureq};
pub use outbox::{Outbox, Pending};
pub use time::{format_time, parse_http_date, parse_time, unix_now};
pub use types::*;
pub use vault::Vault;
pub use view::{Account, Panel};

pub const NATIVECLOUD_URL: &str = "";
pub const NATIVECLOUD_ANON_KEY: &str = "";
pub const TERMS_URL: &str = "https://nativecompute.net/cloud/terms";
pub const PRIVACY_URL: &str = "https://nativecompute.net/cloud/privacy";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CloudConfig {
    pub url: String,
    pub anon_key: String,
}

impl CloudConfig {
    pub fn new(url: &str, anon_key: &str) -> CloudConfig {
        CloudConfig { url: url.trim().trim_end_matches('/').to_string(), anon_key: anon_key.trim().to_string() }
    }

    pub fn nativecloud() -> CloudConfig {
        CloudConfig::new(NATIVECLOUD_URL, NATIVECLOUD_ANON_KEY).with_env()
    }

    pub fn with_env(self) -> CloudConfig {
        let url = std::env::var("NCF_CLOUD_URL").ok().filter(|v| !v.trim().is_empty());
        let key = std::env::var("NCF_CLOUD_ANON_KEY").ok().filter(|v| !v.trim().is_empty());
        CloudConfig::new(url.as_deref().unwrap_or(&self.url), key.as_deref().unwrap_or(&self.anon_key))
    }

    pub fn is_set(&self) -> bool {
        self.url.starts_with("http") && !self.anon_key.is_empty()
    }

    pub fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.url, path)
    }
}

pub fn time_now_text() -> String {
    format_time(unix_now())
}

pub fn password_rules(password: &str) -> [(&'static str, bool); 4] {
    [
        ("10+ characters", password.chars().count() >= 10),
        ("Upper case", password.chars().any(|c| c.is_uppercase())),
        ("Lower case", password.chars().any(|c| c.is_lowercase())),
        ("A digit", password.chars().any(|c| c.is_ascii_digit())),
    ]
}

pub fn password_ok(password: &str) -> bool {
    password_rules(password).iter().all(|(_, ok)| *ok)
}

pub fn email_ok(email: &str) -> bool {
    let e = email.trim();
    match e.split_once('@') {
        Some((user, host)) => !user.is_empty() && host.contains('.') && !host.starts_with('.') && !host.ends_with('.') && !e.contains(char::is_whitespace),
        None => false,
    }
}

pub fn new_id() -> String {
    use ring::rand::SecureRandom;
    let mut b = [0u8; 16];
    if ring::rand::SystemRandom::new().fill(&mut b).is_err() {
        let n = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        b = n.to_le_bytes();
    }
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

pub fn is_id(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36 && b.iter().enumerate().all(|(i, c)| if matches!(i, 8 | 13 | 18 | 23) { *c == b'-' } else { c.is_ascii_hexdigit() })
}
