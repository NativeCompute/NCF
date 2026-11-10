use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

use super::*;

pub const OFFLINE_LABEL: &str = "Your offline version";
pub const LOCKED_LABEL: &str = "Your unsaved version";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Heartbeat {
    pub feed: Option<Duration>,
    pub watching: Duration,
    pub lease: Duration,
}

impl Heartbeat {
    pub fn sync() -> Heartbeat {
        Heartbeat { feed: Some(Duration::from_secs(30)), watching: Duration::from_secs(5), lease: Duration::from_secs(30) }
    }

    pub fn off() -> Heartbeat {
        Heartbeat { feed: None, watching: Duration::from_secs(5), lease: Duration::from_secs(30) }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SyncState {
    pub signed_in: bool,
    pub online: bool,
    pub waiting: usize,
    pub last_sync: Option<i64>,
    pub kept: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mood {
    Calm,
    Busy,
    Warn,
}

impl SyncState {
    pub fn readout(&self, now: i64) -> (String, Mood) {
        if !self.signed_in {
            return ("SIGNED OUT".into(), Mood::Warn);
        }
        if !self.online {
            return if self.waiting > 0 { (format!("OFFLINE · {} WAITING", self.waiting), Mood::Warn) } else { ("OFFLINE".into(), Mood::Warn) };
        }
        if self.waiting > 0 {
            return (format!("SYNCING · {}", self.waiting), Mood::Busy);
        }
        match self.last_sync {
            Some(t) => (format!("SYNCED · {}", time::ago(now - t).to_uppercase()), Mood::Calm),
            None => ("SYNCED".into(), Mood::Calm),
        }
    }
}

pub enum Job<M> {
    Run(Box<dyn FnOnce(&Client) -> M + Send>),
    Save(Pending),
    Hold(String),
    Release(String),
    Watch(Option<String>),
    Cursor(i64),
    Poke,
    Stop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event<M> {
    Reply(M),
    Synced { project: String, version: i64 },
    Kept { project: String, snapshot: Option<String>, theirs: Option<Box<Project>>, lease: Option<Lease> },
    Dropped { project: String, error: CloudError },
    LeaseLost { project: String, lease: Option<Lease> },
    Feed(Feed),
    Resync,
    State(SyncState),
    SignedOut,
}

pub struct Worker<M> {
    client: Client,
    device: Device,
    outbox: Outbox,
    beat: Heartbeat,
    held: Vec<String>,
    watching: Option<String>,
    cursor: Option<i64>,
    last_feed: Option<Instant>,
    last_lease: Option<Instant>,
    next_flush: Option<Instant>,
    backoff: Duration,
    lineage: HashMap<String, (Vec<i64>, i64)>,
    state: SyncState,
    told: Option<SyncState>,
    out: Vec<Event<M>>,
}

impl<M> Worker<M> {
    pub fn new(client: Client, device: Device, outbox: Outbox, beat: Heartbeat) -> Worker<M> {
        let state = SyncState { signed_in: client.signed_in(), online: true, waiting: outbox.len(), ..Default::default() };
        Worker {
            client,
            device,
            next_flush: if outbox.is_empty() { None } else { Some(Instant::now()) },
            outbox,
            beat,
            held: Vec::new(),
            watching: None,
            cursor: None,
            last_feed: None,
            last_lease: None,
            backoff: Duration::from_secs(2),
            lineage: HashMap::new(),
            state,
            told: None,
            out: Vec::new(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn outbox(&self) -> &Outbox {
        &self.outbox
    }

    pub fn state(&self) -> &SyncState {
        &self.state
    }

    pub fn held(&self) -> &[String] {
        &self.held
    }

    pub fn cursor(&self) -> Option<i64> {
        self.cursor
    }

    pub fn handle(&mut self, job: Job<M>, now: Instant) -> bool {
        match job {
            Job::Run(f) => {
                let m = f(&self.client);
                self.out.push(Event::Reply(m));
            }
            Job::Save(mut p) => {
                if let Some((seen, latest)) = self.lineage.get(&p.project) {
                    if seen.contains(&p.base_version) {
                        p.base_version = *latest;
                    }
                }
                self.outbox.put(p);
                self.next_flush = Some(now);
                self.flush(now);
            }
            Job::Hold(p) => {
                if !self.held.contains(&p) {
                    self.held.push(p);
                }
            }
            Job::Release(p) => {
                self.held.retain(|h| *h != p);
                if self.client.signed_in() {
                    let _ = self.client.release_lease(&p, &self.device);
                }
            }
            Job::Watch(p) => {
                self.watching = p;
                self.last_feed = None;
            }
            Job::Cursor(seq) => self.cursor = Some(seq),
            Job::Poke => {
                self.last_feed = None;
                self.next_flush = Some(now);
                self.backoff = Duration::from_secs(2);
            }
            Job::Stop => {
                for p in std::mem::take(&mut self.held) {
                    let _ = self.client.release_lease(&p, &self.device);
                }
                return false;
            }
        }
        self.tell();
        true
    }

    pub fn tick(&mut self, now: Instant) {
        self.state.signed_in = self.client.signed_in();
        if self.state.signed_in {
            if self.next_flush.map(|t| now >= t).unwrap_or(false) {
                self.flush(now);
            }
            if !self.held.is_empty() && self.last_lease.map(|t| now.duration_since(t) >= self.beat.lease).unwrap_or(true) {
                self.last_lease = Some(now);
                self.renew();
            }
            if let Some(every) = self.beat.feed {
                let every = if self.watching.is_some() { self.beat.watching.min(every) } else { every };
                if self.last_feed.map(|t| now.duration_since(t) >= every).unwrap_or(true) {
                    self.last_feed = Some(now);
                    self.feed();
                }
            }
        }
        self.tell();
    }

    pub fn drain(&mut self) -> Vec<Event<M>> {
        std::mem::take(&mut self.out)
    }

    fn tell(&mut self) {
        self.state.signed_in = self.client.signed_in();
        self.state.waiting = self.outbox.len();
        if self.told.as_ref() != Some(&self.state) {
            self.told = Some(self.state.clone());
            self.out.push(Event::State(self.state.clone()));
        }
    }

    fn failed(&mut self, e: &CloudError, now: Instant) {
        match e {
            CloudError::SignedOut => {
                self.state.signed_in = false;
                self.out.push(Event::SignedOut);
                self.next_flush = None;
            }
            CloudError::Offline => {
                self.state.online = false;
                self.next_flush = Some(now + self.backoff);
                self.backoff = (self.backoff * 2).min(Duration::from_secs(60));
            }
            _ => {
                self.next_flush = Some(now + self.backoff);
                self.backoff = (self.backoff * 2).min(Duration::from_secs(60));
            }
        }
    }

    fn went_well(&mut self) {
        self.state.online = true;
        self.backoff = Duration::from_secs(2);
    }

    pub fn flush(&mut self, now: Instant) {
        if !self.client.signed_in() {
            self.next_flush = None;
            return;
        }
        self.next_flush = None;
        for p in self.outbox.items().to_vec() {
            let saved = self.client.save_project(&p.project, p.base_version, &p.body, p.title.as_deref(), p.meta.as_ref(), Some(&self.device));
            match saved {
                Ok(Saved::Saved { version }) => {
                    self.went_well();
                    self.outbox.remove(&p.project);
                    let entry = self.lineage.entry(p.project.clone()).or_insert_with(|| (Vec::new(), version));
                    entry.0.push(p.base_version);
                    entry.0.push(version);
                    entry.1 = version;
                    self.state.last_sync = Some(self.client.server_now());
                    self.out.push(Event::Synced { project: p.project, version });
                }
                Ok(Saved::Conflict(theirs)) => {
                    self.went_well();
                    self.lineage.remove(&p.project);
                    let snapshot = self.client.take_snapshot(&p.project, OFFLINE_LABEL, "offline", Some(&p.body)).ok();
                    self.outbox.remove(&p.project);
                    self.state.kept += 1;
                    self.out.push(Event::Kept { project: p.project, snapshot, theirs: Some(Box::new(theirs)), lease: None });
                }
                Ok(Saved::Locked(lease)) => {
                    self.went_well();
                    self.lineage.remove(&p.project);
                    self.held.retain(|h| *h != p.project);
                    let snapshot = self.client.take_snapshot(&p.project, LOCKED_LABEL, "conflict", Some(&p.body)).ok();
                    self.outbox.remove(&p.project);
                    self.state.kept += 1;
                    self.out.push(Event::Kept { project: p.project, snapshot, theirs: None, lease });
                }
                Err(e) if e.is_retryable() || e == CloudError::SignedOut => {
                    self.failed(&e, now);
                    return;
                }
                Err(e) => {
                    self.went_well();
                    self.lineage.remove(&p.project);
                    self.outbox.remove(&p.project);
                    self.out.push(Event::Dropped { project: p.project, error: e });
                }
            }
        }
    }

    fn renew(&mut self) {
        for p in self.held.clone() {
            match self.client.renew_lease(&p, &self.device) {
                Ok(r) if r.held => self.went_well(),
                Ok(r) => {
                    self.went_well();
                    self.held.retain(|h| *h != p);
                    self.out.push(Event::LeaseLost { project: p, lease: r.lease });
                }
                Err(CloudError::NotFound) | Err(CloudError::Forbidden) => {
                    self.held.retain(|h| *h != p);
                    self.out.push(Event::LeaseLost { project: p, lease: None });
                }
                Err(e) => {
                    let now = Instant::now();
                    self.failed(&e, now);
                    return;
                }
            }
        }
    }

    fn feed(&mut self) {
        let since = self.cursor.unwrap_or(i64::MAX);
        match self.client.changes_since(since) {
            Ok(feed) => {
                self.went_well();
                match self.cursor {
                    None => self.cursor = Some(feed.latest),
                    Some(c) if c > 0 && c < feed.oldest.saturating_sub(1) => {
                        self.cursor = Some(feed.latest);
                        self.out.push(Event::Resync);
                    }
                    Some(c) if c > feed.latest => {
                        self.cursor = Some(feed.latest);
                        self.out.push(Event::Resync);
                    }
                    Some(_) => {
                        if let Some(last) = feed.changes.last() {
                            self.cursor = Some(last.seq);
                            self.out.push(Event::Feed(feed));
                        }
                    }
                }
            }
            Err(e) => {
                let now = Instant::now();
                self.failed(&e, now);
            }
        }
    }
}

enum Backend<M> {
    Thread { tx: Sender<Job<M>>, rx: Receiver<Event<M>> },
    Inline(Box<Worker<M>>),
}

pub struct Engine<M> {
    backend: Backend<M>,
    client: Client,
    device: Device,
    state: SyncState,
}

impl<M: Send + 'static> Engine<M> {
    pub fn spawn(worker: Worker<M>, ctx: Option<eframe::egui::Context>) -> Engine<M> {
        let client = worker.client.clone();
        let device = worker.device.clone();
        let state = worker.state.clone();
        let (tx, jobs) = channel::<Job<M>>();
        let (events, rx) = channel::<Event<M>>();
        std::thread::Builder::new()
            .name("nativecloud".into())
            .spawn(move || {
                let mut w = worker;
                loop {
                    let alive = match jobs.recv_timeout(Duration::from_millis(500)) {
                        Ok(job) => w.handle(job, Instant::now()),
                        Err(RecvTimeoutError::Timeout) => true,
                        Err(RecvTimeoutError::Disconnected) => {
                            w.handle(Job::Stop, Instant::now());
                            false
                        }
                    };
                    if alive {
                        w.tick(Instant::now());
                    }
                    let out = w.drain();
                    let any = !out.is_empty();
                    for e in out {
                        if events.send(e).is_err() {
                            return;
                        }
                    }
                    if any {
                        if let Some(c) = &ctx {
                            c.request_repaint();
                        }
                    }
                    if !alive {
                        return;
                    }
                }
            })
            .ok();
        Engine { backend: Backend::Thread { tx, rx }, client, device, state }
    }

    pub fn inline(worker: Worker<M>) -> Engine<M> {
        let client = worker.client.clone();
        let device = worker.device.clone();
        let state = worker.state.clone();
        Engine { backend: Backend::Inline(Box::new(worker)), client, device, state }
    }

    fn send(&mut self, job: Job<M>) {
        match &mut self.backend {
            Backend::Thread { tx, .. } => {
                let _ = tx.send(job);
            }
            Backend::Inline(w) => {
                w.handle(job, Instant::now());
            }
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn state(&self) -> &SyncState {
        &self.state
    }

    pub fn run(&mut self, f: impl FnOnce(&Client) -> M + Send + 'static) {
        self.send(Job::Run(Box::new(f)));
    }

    pub fn save(&mut self, p: Pending) {
        self.send(Job::Save(p));
    }

    pub fn hold(&mut self, project: &str) {
        self.send(Job::Hold(project.to_string()));
    }

    pub fn release(&mut self, project: &str) {
        self.send(Job::Release(project.to_string()));
    }

    pub fn watch(&mut self, project: Option<&str>) {
        self.send(Job::Watch(project.map(str::to_string)));
    }

    pub fn set_cursor(&mut self, seq: i64) {
        self.send(Job::Cursor(seq));
    }

    pub fn poke(&mut self) {
        self.send(Job::Poke);
    }

    pub fn worker(&mut self) -> Option<&mut Worker<M>> {
        match &mut self.backend {
            Backend::Inline(w) => Some(w),
            Backend::Thread { .. } => None,
        }
    }

    pub fn poll(&mut self) -> Vec<Event<M>> {
        let events: Vec<Event<M>> = match &mut self.backend {
            Backend::Thread { rx, .. } => rx.try_iter().collect(),
            Backend::Inline(w) => {
                w.tick(Instant::now());
                w.drain()
            }
        };
        for e in &events {
            if let Event::State(s) = e {
                self.state = s.clone();
            }
        }
        events
    }
}

impl<M> Drop for Engine<M> {
    fn drop(&mut self) {
        if let Backend::Thread { tx, .. } = &self.backend {
            let _ = tx.send(Job::Stop);
        }
    }
}
