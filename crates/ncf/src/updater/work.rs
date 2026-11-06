use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use super::{assets, detach, old_exe, work_dir, Install, Plan};
use crate::Os;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Wait,
    Download,
    Check,
    Install,
    Reopen,
}

pub const STEPS: [Step; 5] = [Step::Wait, Step::Download, Step::Check, Step::Install, Step::Reopen];

impl Step {
    pub fn label(&self, plan: &Plan) -> String {
        match self {
            Step::Wait => format!("Waiting for {} to close", plan.name),
            Step::Download => format!("Downloading {}", plan.version),
            Step::Check => "Checking the download".to_string(),
            Step::Install => "Installing".to_string(),
            Step::Reopen => format!("Opening {}", plan.name),
        }
    }

    pub fn index(&self) -> usize {
        STEPS.iter().position(|s| s == self).unwrap_or(0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Step(Step),
    Progress(f32),
    Line(String),
    Done,
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    pub step: Option<Step>,
    pub progress: f32,
    pub detail: String,
    pub outcome: Option<Result<(), String>>,
}

impl State {
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::Step(s) => {
                self.step = Some(s);
                self.progress = 0.0;
                self.detail.clear();
            }
            Event::Progress(p) => self.progress = p.clamp(0.0, 1.0),
            Event::Line(l) => self.detail = l,
            Event::Done => {
                self.progress = 1.0;
                self.outcome = Some(Ok(()));
            }
            Event::Failed(why) => self.outcome = Some(Err(why)),
        }
    }

    pub fn installing(&self) -> bool {
        self.outcome.is_none() && self.step == Some(Step::Install)
    }

    pub fn finished(&self) -> bool {
        self.outcome.is_some()
    }
}

pub fn log(plan: &Plan, line: &str) {
    let _ = std::fs::create_dir_all(&plan.data);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(plan.data.join("update.log")) {
        let _ = writeln!(f, "{} {line}", stamp(now_secs()));
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn stamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rest = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}{m:02}{d:02}-{:02}{:02}{:02}", rest / 3600, rest % 3600 / 60, rest % 60)
}

pub fn clean_line(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for n in chars.by_ref() {
                    if n.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        out.push(c);
    }
    out.trim().trim_start_matches("::").trim().to_string()
}

pub fn sha256_hex(path: &Path) -> Result<String, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
    let mut ctx = ring::digest::Context::new(&ring::digest::SHA256);
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut buf).map_err(|e| format!("could not read {}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
    }
    Ok(ctx.finish().as_ref().iter().map(|b| format!("{b:02x}")).collect())
}

pub fn expected_sum(text: &str) -> Option<String> {
    let word = text.split_whitespace().next()?.to_ascii_lowercase();
    (word.len() == 64 && word.chars().all(|c| c.is_ascii_hexdigit())).then_some(word)
}

pub fn local_path(source: &str) -> Option<PathBuf> {
    let rest = source.strip_prefix("file://")?;
    let rest = if rest.len() > 3 && rest.starts_with('/') && rest.as_bytes()[2] == b':' { &rest[1..] } else { rest };
    Some(PathBuf::from(rest))
}

fn cancelled(cancel: &AtomicBool) -> Result<(), String> {
    if cancel.load(Ordering::SeqCst) {
        Err("Cancelled. Nothing was changed.".to_string())
    } else {
        Ok(())
    }
}

fn running(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    if cfg!(windows) {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/NH"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).split_whitespace().any(|w| w == pid.to_string()))
            .unwrap_or(false)
    } else {
        Command::new("kill").args(["-0", &pid.to_string()]).stderr(Stdio::null()).status().map(|s| s.success()).unwrap_or(false)
    }
}

fn wait_parent(plan: &Plan, cancel: &AtomicBool) -> Result<(), String> {
    let until = Instant::now() + Duration::from_secs(120);
    while running(plan.parent) {
        cancelled(cancel)?;
        if Instant::now() > until {
            return Err(format!("{} is still open, so nothing was changed. Quit it and try again.", plan.name));
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    Ok(())
}

fn fetch(plan: &Plan, name: &str, to: &Path, tx: &Sender<Event>, cancel: &AtomicBool, report: bool) -> Result<(), String> {
    if let Some(dir) = local_path(&plan.source) {
        let from = dir.join(name);
        std::fs::copy(&from, to).map_err(|e| format!("could not get {name} from {}: {e}", dir.display()))?;
        if report {
            let _ = tx.send(Event::Progress(1.0));
        }
        return Ok(());
    }
    let url = format!("{}/{name}", plan.source);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(30))
        .timeout_read(Duration::from_secs(60))
        .user_agent(&format!("NativeCompute-updater/{}", plan.version))
        .build();
    let mut last = String::new();
    for _ in 0..3 {
        cancelled(cancel)?;
        match agent.get(&url).call() {
            Ok(resp) => {
                let total = resp.header("Content-Length").and_then(|v| v.parse::<u64>().ok());
                let mut reader = resp.into_reader();
                let mut file = std::fs::File::create(to).map_err(|e| format!("could not save {name}: {e}"))?;
                let mut buf = vec![0u8; 1 << 16];
                let mut got = 0u64;
                let ok = loop {
                    cancelled(cancel)?;
                    match reader.read(&mut buf) {
                        Ok(0) => break true,
                        Ok(n) => {
                            file.write_all(&buf[..n]).map_err(|e| format!("could not save {name}: {e}"))?;
                            got += n as u64;
                            if let (true, Some(total)) = (report, total) {
                                let _ = tx.send(Event::Progress(got as f32 / total.max(1) as f32));
                            }
                        }
                        Err(e) => {
                            last = e.to_string();
                            break false;
                        }
                    }
                };
                if ok && total.map(|t| t == got).unwrap_or(true) {
                    return Ok(());
                }
            }
            Err(ureq::Error::Status(404, _)) => return Err(format!("{name} isn't part of release {}.", plan.version)),
            Err(e) => last = e.to_string(),
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(format!("could not download {name}: {last}. Check your connection and try again."))
}

fn stream(mut cmd: Command, plan: &Plan, tx: &Sender<Event>) -> Result<(), String> {
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("could not run the installer: {e}"))?;
    let (ltx, lrx) = std::sync::mpsc::channel::<String>();
    let mut readers = Vec::new();
    if let Some(out) = child.stdout.take() {
        let ltx = ltx.clone();
        readers.push(std::thread::spawn(move || {
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                let _ = ltx.send(line);
            }
        }));
    }
    if let Some(err) = child.stderr.take() {
        let ltx = ltx.clone();
        readers.push(std::thread::spawn(move || {
            for line in std::io::BufReader::new(err).lines().map_while(Result::ok) {
                let _ = ltx.send(line);
            }
        }));
    }
    drop(ltx);
    let mut last = String::new();
    for line in lrx {
        let line = clean_line(&line);
        if line.is_empty() {
            continue;
        }
        log(plan, &line);
        let _ = tx.send(Event::Line(line.clone()));
        last = line;
    }
    for r in readers {
        let _ = r.join();
    }
    let status = child.wait().map_err(|e| format!("the installer did not finish: {e}"))?;
    if status.success() {
        Ok(())
    } else if last.is_empty() {
        Err(format!("the installer stopped ({status})."))
    } else {
        Err(last)
    }
}

fn find(dir: &Path, name: &str, depth: usize) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut dirs = Vec::new();
    for e in entries.flatten() {
        let p = e.path();
        if p.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case(name)).unwrap_or(false) && p.is_file() {
            return Some(p);
        }
        if p.is_dir() {
            dirs.push(p);
        }
    }
    if depth == 0 {
        return None;
    }
    dirs.into_iter().find_map(|d| find(&d, name, depth - 1))
}

pub fn check_runs(bin: &Path, plan: &Plan) -> Result<(), String> {
    let out = Command::new(bin)
        .arg("--version")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("the new version does not run on this computer ({e}). Nothing was changed."))?;
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if out.status.success() && text.split_whitespace().any(|w| w.trim_start_matches(['v', 'V']) == plan.version) {
        Ok(())
    } else {
        Err(format!("the new version does not run on this computer, so nothing was changed. It said: {}", text.trim()))
    }
}

pub fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_dir(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), &target)?;
        }
    }
    Ok(())
}

pub fn back_up(data: &Path) -> Result<Option<PathBuf>, String> {
    if !data.is_dir() {
        return Ok(None);
    }
    let name = format!("{}-backup-{}", data.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "data".into()), stamp(now_secs()));
    let to = data.with_file_name(name);
    copy_dir(data, &to).map_err(|e| format!("could not copy your library to {}, so nothing was changed: {e}", to.display()))?;
    Ok(Some(to))
}

pub fn swap(exe: &Path, new: &Path, slug: &str, patience: Duration) -> Result<(), String> {
    let old = old_exe(exe, slug);
    let _ = std::fs::remove_file(&old);
    let until = Instant::now() + patience;
    loop {
        match std::fs::rename(exe, &old) {
            Ok(()) => break,
            Err(e) if Instant::now() > until => return Err(format!("could not move the old version aside ({e}). Nothing was changed.")),
            Err(_) => std::thread::sleep(Duration::from_millis(500)),
        }
    }
    if let Err(e) = std::fs::copy(new, exe) {
        let _ = std::fs::remove_file(exe);
        let back = std::fs::rename(&old, exe);
        return Err(match back {
            Ok(()) => format!("could not put the new version in place ({e}). The old one is back."),
            Err(b) => format!("could not put the new version in place ({e}), and the old one is at {} ({b}).", old.display()),
        });
    }
    Ok(())
}

fn unpack_zip(zip: &Path, into: &Path) -> Result<(), String> {
    std::fs::create_dir_all(into).map_err(|e| e.to_string())?;
    let tar = Command::new("tar").arg("-xf").arg(zip).arg("-C").arg(into).stdin(Stdio::null()).output();
    if matches!(&tar, Ok(o) if o.status.success()) {
        return Ok(());
    }
    let script = format!(
        "Expand-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
        zip.display().to_string().replace('\'', "''"),
        into.display().to_string().replace('\'', "''")
    );
    let ps = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).stdin(Stdio::null()).output();
    match ps {
        Ok(o) if o.status.success() => Ok(()),
        _ => Err("could not unpack the download. Nothing was changed.".to_string()),
    }
}

fn install(plan: &Plan, dir: &Path, files: &[String], tx: &Sender<Event>) -> Result<(), String> {
    match &plan.install {
        Install::MacBundle { app } => {
            let apps = app.parent().ok_or("the app is not in a folder")?;
            let mut cmd = Command::new("/bin/bash");
            cmd.arg(dir.join(&files[2]))
                .current_dir(dir)
                .env("NCF_INSTALL_SOURCE", format!("file://{}", dir.display()))
                .env("NCF_INSTALL_APP_DIR", apps)
                .env("NCF_INSTALL_NO_OPEN", "1")
                .env("NCF_INSTALL_WAIT", "60");
            stream(cmd, plan, tx)
        }
        Install::LinuxLocal { .. } => {
            let unpacked = dir.join("unpacked");
            let _ = std::fs::remove_dir_all(&unpacked);
            std::fs::create_dir_all(&unpacked).map_err(|e| e.to_string())?;
            let ok = Command::new("tar").arg("-xzf").arg(dir.join(&files[0])).arg("-C").arg(&unpacked).status().map(|s| s.success()).unwrap_or(false);
            if !ok {
                return Err("could not unpack the download. Nothing was changed.".to_string());
            }
            let script = find(&unpacked, "install.sh", 2).ok_or("the download has no install.sh. Nothing was changed.")?;
            let root = script.parent().ok_or("the download is laid out oddly")?.to_path_buf();
            check_runs(&root.join(&plan.slug), plan)?;
            let mut cmd = Command::new("bash");
            cmd.arg(&script).current_dir(&root);
            stream(cmd, plan, tx)
        }
        Install::WindowsExe { exe } => {
            let unpacked = dir.join("unpacked");
            let _ = std::fs::remove_dir_all(&unpacked);
            unpack_zip(&dir.join(&files[0]), &unpacked)?;
            let new = find(&unpacked, &format!("{}.exe", plan.slug), 2).ok_or("the download has no program in it. Nothing was changed.")?;
            check_runs(&new, plan)?;
            if let Some(to) = back_up(&plan.data)? {
                let line = format!("Your library was copied to {} first", to.display());
                log(plan, &line);
                let _ = tx.send(Event::Line(line));
            }
            swap(exe, &new, &plan.slug, Duration::from_secs(30))?;
            let line = format!("Installed to {}", exe.display());
            log(plan, &line);
            let _ = tx.send(Event::Line(line));
            Ok(())
        }
    }
}

pub fn update(plan: &Plan, tx: &Sender<Event>, cancel: &AtomicBool) -> Result<(), String> {
    let step = |s: Step| {
        log(plan, &s.label(plan));
        let _ = tx.send(Event::Step(s));
    };
    step(Step::Wait);
    wait_parent(plan, cancel)?;

    step(Step::Download);
    let dir = work_dir(&plan.slug).join("download");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not make {}: {e}", dir.display()))?;
    let files = assets(&plan.slug, &plan.version, plan.install.os());
    if files.is_empty() {
        return Err("this platform can't update itself".to_string());
    }
    for (k, name) in files.iter().enumerate() {
        fetch(plan, name, &dir.join(name), tx, cancel, k == 0)?;
    }

    step(Step::Check);
    cancelled(cancel)?;
    let want = std::fs::read_to_string(dir.join(&files[1])).ok().as_deref().and_then(expected_sum).ok_or("the checksum file is unreadable. Nothing was changed.")?;
    let got = sha256_hex(&dir.join(&files[0]))?;
    if want != got {
        return Err("the download is damaged (its checksum does not match). Nothing was changed. Try again.".to_string());
    }
    let _ = tx.send(Event::Line("Checksum matches".to_string()));
    cancelled(cancel)?;

    step(Step::Install);
    install(plan, &dir, &files, tx)?;
    let _ = std::fs::remove_dir_all(&dir);
    log(plan, &format!("Updated to {}", plan.version));
    Ok(())
}

pub fn reopen(plan: &Plan) -> Result<(), String> {
    let mut cmd = match &plan.install {
        Install::MacBundle { app } => {
            let mut c = Command::new("open");
            c.arg(app);
            c
        }
        Install::LinuxLocal { bin } => Command::new(bin),
        Install::WindowsExe { exe } => Command::new(exe),
    };
    if let Some(dir) = plan.install.path().parent() {
        cmd.current_dir(dir);
    }
    detach(&mut cmd);
    cmd.spawn().map(|_| ()).map_err(|e| format!("could not open {}: {e}", plan.name))
}

pub type Worker = std::thread::JoinHandle<Result<(), String>>;

pub fn spawn(plan: Plan, cancel: std::sync::Arc<AtomicBool>) -> (std::sync::mpsc::Receiver<Event>, Worker) {
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = update(&plan, &tx, &cancel);
        match &result {
            Ok(()) => {
                let _ = tx.send(Event::Done);
            }
            Err(why) => {
                log(&plan, &format!("FAILED: {why}"));
                let _ = tx.send(Event::Failed(why.clone()));
            }
        }
        result
    });
    (rx, worker)
}

pub fn simulate(plan: Plan, demo: crate::debug::Demo, cancel: std::sync::Arc<AtomicBool>) -> (std::sync::mpsc::Receiver<Event>, Worker) {
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let nap = |ms: u64| std::thread::sleep(Duration::from_millis(ms));
        let result = (|| -> Result<(), String> {
            let _ = tx.send(Event::Step(Step::Wait));
            nap(700);
            cancelled(&cancel)?;
            let _ = tx.send(Event::Step(Step::Download));
            let steps = if demo == crate::debug::Demo::Slow { 400 } else { 40 };
            for k in 1..=steps {
                cancelled(&cancel)?;
                nap(50);
                let _ = tx.send(Event::Progress(k as f32 / steps as f32));
            }
            let _ = tx.send(Event::Step(Step::Check));
            nap(600);
            cancelled(&cancel)?;
            if demo == crate::debug::Demo::Fails {
                return Err("the download is damaged (its checksum does not match). Nothing was changed. Try again.".to_string());
            }
            let _ = tx.send(Event::Line("Checksum matches".to_string()));
            let _ = tx.send(Event::Step(Step::Install));
            nap(500);
            let _ = tx.send(Event::Line(format!("Your library was copied to {}-backup first", plan.data.display())));
            nap(900);
            let _ = tx.send(Event::Line(format!("Installed to {}", plan.install.path().display())));
            nap(400);
            Ok(())
        })();
        match &result {
            Ok(()) => {
                let _ = tx.send(Event::Done);
            }
            Err(why) => {
                let _ = tx.send(Event::Failed(why.clone()));
            }
        }
        result
    });
    (rx, worker)
}

pub fn print(plan: &Plan, rx: std::sync::mpsc::Receiver<Event>) {
    for event in rx {
        match event {
            Event::Step(s) => println!(":: {}", s.label(plan)),
            Event::Line(l) => println!("   {l}"),
            Event::Progress(_) => {}
            Event::Done => println!(":: Updated to {}", plan.version),
            Event::Failed(why) => eprintln!(":: Could not update: {why}"),
        }
    }
}

pub fn finish(plan: &Plan, worker: Worker, reopened: bool) -> i32 {
    let result = worker.join().unwrap_or_else(|_| Err("the updater stopped unexpectedly".to_string()));
    if plan.reopen && !reopened {
        if let Err(e) = reopen(plan) {
            log(plan, &e);
        }
    }
    if result.is_ok() {
        0
    } else {
        1
    }
}

pub fn headless(plan: &Plan) -> i32 {
    let cancel = std::sync::Arc::new(AtomicBool::new(false));
    let (rx, worker) = spawn(plan.clone(), cancel);
    print(plan, rx);
    finish(plan, worker, false)
}

pub fn os_name(os: Os) -> &'static str {
    match os {
        Os::Mac => "macOS",
        Os::Linux => "Linux",
        Os::Windows => "Windows",
        Os::Web => "the web",
    }
}
