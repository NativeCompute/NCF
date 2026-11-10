use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ring::rand::SecureRandom;

use super::client::enc;
use super::CloudConfig;

pub const WAIT: Duration = Duration::from_secs(300);

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 2);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16 | (*chunk.get(1).unwrap_or(&0) as u32) << 8 | *chunk.get(2).unwrap_or(&0) as u32;
        let take = chunk.len() + 1;
        for k in 0..take {
            out.push(ALPHABET[((n >> (18 - 6 * k)) & 63) as usize] as char);
        }
    }
    out
}

pub fn challenge_of(verifier: &str) -> String {
    base64url(ring::digest::digest(&ring::digest::SHA256, verifier.as_bytes()).as_ref())
}

fn random(n: usize) -> Option<Vec<u8>> {
    let mut b = vec![0u8; n];
    ring::rand::SystemRandom::new().fill(&mut b).ok()?;
    Some(b)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

impl Pkce {
    pub fn try_new() -> Option<Pkce> {
        let verifier = base64url(&random(32)?);
        let challenge = challenge_of(&verifier);
        Some(Pkce { verifier, challenge, state: base64url(&random(16)?) })
    }

    pub fn new() -> Pkce {
        Pkce::try_new().expect("the system has no secure random source")
    }
}

impl Default for Pkce {
    fn default() -> Self {
        Pkce::new()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callback {
    Code(String),
    Cancelled,
    TimedOut,
}

pub struct BrowserSignIn {
    pub pkce: Pkce,
    pub url: String,
    pub port: u16,
    pub started: Instant,
    stop: Arc<AtomicBool>,
    rx: Receiver<Callback>,
    over: bool,
}

pub fn login_url(cfg: &CloudConfig, app: &str, device: &str, pkce: &Pkce, port: u16, signup: bool) -> String {
    let mut url = format!(
        "{}?app={}&device={}&challenge={}&state={}&port={port}",
        cfg.web("/login"),
        enc(app),
        enc(device),
        enc(&pkce.challenge),
        enc(&pkce.state)
    );
    if signup {
        url.push_str("&mode=signup");
    }
    url
}

impl BrowserSignIn {
    pub fn start(cfg: &CloudConfig, info: &crate::AppInfo, device: &str, signup: bool, wake: impl Fn() + Send + 'static) -> std::io::Result<BrowserSignIn> {
        let pkce = Pkce::try_new().ok_or_else(|| std::io::Error::other("the system has no secure random source"))?;
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        listener.set_nonblocking(true)?;
        let port = listener.local_addr()?.port();
        let url = login_url(cfg, info.id, device, &pkce, port, signup);
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = channel();
        let (flag, state, name) = (stop.clone(), pkce.state.clone(), info.name.to_string());
        std::thread::Builder::new().name("nativecloud-sign-in".into()).spawn(move || {
            let deadline = Instant::now() + WAIT;
            let result = loop {
                if flag.load(Ordering::Relaxed) {
                    return;
                }
                if Instant::now() >= deadline {
                    break Callback::TimedOut;
                }
                match listener.accept() {
                    Ok((stream, _)) => {
                        if let Some(cb) = answer(stream, &state, &name, &flag) {
                            break cb;
                        }
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(40)),
                }
            };
            let _ = tx.send(result);
            wake();
        })?;
        Ok(BrowserSignIn { pkce, url, port, started: Instant::now(), stop, rx, over: false })
    }

    pub fn verifier(&self) -> &str {
        &self.pkce.verifier
    }

    pub fn poll(&mut self) -> Option<Callback> {
        if self.over {
            return None;
        }
        match self.rx.try_recv() {
            Ok(cb) => {
                self.over = true;
                Some(cb)
            }
            Err(TryRecvError::Disconnected) => {
                self.over = true;
                Some(Callback::Cancelled)
            }
            Err(TryRecvError::Empty) => None,
        }
    }

    pub fn cancel(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub fn left(&self) -> Duration {
        WAIT.saturating_sub(self.started.elapsed())
    }
}

impl Drop for BrowserSignIn {
    fn drop(&mut self) {
        self.cancel();
    }
}

fn dec(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                match u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("zz"), 16) {
                    Ok(x) => {
                        out.push(x);
                        i += 3;
                        continue;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b'+' => out.push(b' '),
            c => out.push(c),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub fn query_of(target: &str) -> (String, Vec<(String, String)>) {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let pairs = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (dec(k), dec(v))
        })
        .collect();
    (path.to_string(), pairs)
}

fn read_head(stream: &mut TcpStream, stop: &AtomicBool) -> Option<String> {
    stream.set_nonblocking(false).ok()?;
    let until = Instant::now() + Duration::from_secs(3);
    let mut buf = Vec::new();
    let mut chunk = [0u8; 1024];
    while buf.len() < 8192 {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() || stop.load(Ordering::Relaxed) {
            return None;
        }
        stream.set_read_timeout(Some(left.min(Duration::from_millis(250)))).ok()?;
        let n = match stream.read(&mut chunk) {
            Ok(n) => n,
            Err(e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => continue,
            Err(_) => return None,
        };
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
    }
    let text = String::from_utf8_lossy(&buf).into_owned();
    text.lines().next().map(str::to_string)
}

fn answer(mut stream: TcpStream, state: &str, app: &str, stop: &AtomicBool) -> Option<Callback> {
    let line = read_head(&mut stream, stop)?;
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
    if method != "GET" {
        send(&mut stream, 405, &page("Not here", "This address only finishes a sign-in.", false));
        return None;
    }
    let (path, query) = query_of(target);
    if path != "/callback" {
        send(&mut stream, 404, &page("Not here", "This address only finishes a sign-in.", false));
        return None;
    }
    let get = |k: &str| query.iter().find(|(x, _)| x == k).map(|(_, v)| v.clone()).unwrap_or_default();
    if get("state") != state {
        send(&mut stream, 400, &page("That link is out of date", &format!("Start signing in again from {app}."), false));
        return None;
    }
    if !get("error").is_empty() {
        send(&mut stream, 200, &page("Sign-in cancelled", &format!("Nothing changed. You can close this tab and go back to {app}."), false));
        return Some(Callback::Cancelled);
    }
    let code = get("code");
    if code.is_empty() || code.len() > 32 || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        send(&mut stream, 400, &page("That link is incomplete", &format!("Start signing in again from {app}."), false));
        return None;
    }
    send(&mut stream, 200, &page(&format!("You're signed in to {app}"), &format!("You can close this tab and go back to {app}."), true));
    Some(Callback::Code(code))
}

fn send(stream: &mut TcpStream, status: u16, body: &str) {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Method Not Allowed",
    };
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: default-src 'none'; style-src 'unsafe-inline'\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body.as_bytes());
    let _ = stream.flush();
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

pub fn page(title: &str, body: &str, ok: bool) -> String {
    let mark = if ok { "#5ee0a0" } else { "#8a8fa8" };
    let (title, body) = (escape(title), escape(body));
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · NativeCloud</title>
<style>
:root {{ color-scheme: dark; --void: #000000; --panel: #0c0e16; --line: rgba(255,255,255,0.122); --text: #f2f3f7; --text-2: #a6abc2; --text-3: #8a8fa8; --signal: #3b4cff; }}
@media (prefers-color-scheme: light) {{ :root {{ color-scheme: light; --void: #f4f5f8; --panel: #ffffff; --line: rgba(10,12,22,0.12); --text: #0b0d16; --text-2: #4a4f66; --text-3: #6a6f88; }} }}
* {{ box-sizing: border-box; }}
html, body {{ margin: 0; height: 100%; background: var(--void); color: var(--text); font: 15px/1.5 "Inter", system-ui, -apple-system, "Segoe UI", sans-serif; }}
main {{ min-height: 100%; display: grid; place-items: center; padding: 24px 16px; }}
section {{ width: 100%; max-width: 420px; background: var(--panel); border: 1px solid var(--line); border-radius: 2px; padding: 28px 24px; }}
.hud {{ font: 600 11px/1 ui-monospace, "JetBrains Mono", Menlo, monospace; letter-spacing: 0.12em; text-transform: uppercase; color: var(--text-3); display: flex; align-items: center; gap: 8px; }}
.dot {{ width: 8px; height: 8px; border-radius: 50%; background: {mark}; }}
h1 {{ font-size: 22px; line-height: 1.25; margin: 18px 0 6px; font-weight: 650; }}
p {{ margin: 0; color: var(--text-2); }}
.rule {{ height: 2px; width: 32px; background: var(--signal); margin-top: 22px; }}
</style>
</head>
<body>
<main>
<section>
<div class="hud"><span class="dot"></span>NativeCloud</div>
<h1>{title}</h1>
<p>{body}</p>
<div class="rule"></div>
</section>
</main>
</body>
</html>
"#
    )
}
