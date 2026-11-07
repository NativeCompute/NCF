use std::path::Path;

use eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, Sense, Stroke, Vec2};

use super::engine::Mood;
use super::*;
use crate::alerts::Tone;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};
use crate::{ui, AppInfo};

pub const SYNCED_LOOK_KEYS: [&str; 5] = ["light_mode", "animations", "font_scale", "splash", "check_updates"];

pub struct Ask {
    pub title: String,
    pub body: String,
    pub yes: String,
    pub danger: bool,
    pub action: String,
}

fn hue_colour(hue: i32) -> Color32 {
    let h = hue.rem_euclid(360) as f32 / 360.0;
    if pal().dark {
        theme::hsl(h, 0.42, 0.42)
    } else {
        theme::hsl(h, 0.48, 0.58)
    }
}

pub fn initials(name: &str) -> String {
    let mut out = String::new();
    for word in name.split_whitespace().take(2) {
        if let Some(c) = word.chars().next() {
            out.extend(c.to_uppercase());
        }
    }
    if out.is_empty() {
        out.push('?');
    }
    out
}

pub fn avatar(ui: &mut egui::Ui, name: &str, hue: i32, size: f32) -> egui::Response {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let r = size * 0.5;
    ui.painter().circle_filled(rect.center(), r, hue_colour(hue));
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, initials(name), theme::font_semi((r * 0.9).max(7.0)), Color32::from_white_alpha(235));
    ui.painter().circle_stroke(rect.center(), r - 0.5, Stroke::new(1.0_f32, theme::wash(if p.dark { Color32::WHITE } else { Color32::BLACK }, 28)));
    resp.on_hover_text(name)
}

pub fn readout(ui: &mut egui::Ui, state: &SyncState, now: i64) -> egui::Response {
    let p = pal();
    let (text, mood) = state.readout(now);
    let col = match mood {
        Mood::Calm => p.text_3,
        Mood::Busy => p.signal_hi,
        Mood::Warn => p.warn,
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        let (r, _) = ui.allocate_exact_size(Vec2::splat(13.0), Sense::hover());
        icons::draw(ui.painter(), r, Icon::Cloud, col);
        ui::hud(ui, &text, Some(col));
    })
    .response
}

pub fn meter(ui: &mut egui::Ui, label: &str, used: i64, max: i64, shown: &str) {
    let p = pal();
    let w = ui.available_width().min(460.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 34.0), Sense::hover());
    let painter = ui.painter();
    painter.text(Pos2::new(rect.left(), rect.top() + 9.0), egui::Align2::LEFT_CENTER, label, theme::font_med(theme::T_SM), p.text);
    painter.text(Pos2::new(rect.right(), rect.top() + 9.0), egui::Align2::RIGHT_CENTER, shown, theme::font_hud(theme::T_MICRO + 0.5), p.text_3);
    let bar = Rect::from_min_size(Pos2::new(rect.left(), rect.top() + 24.0), Vec2::new(w, 4.0));
    painter.rect_filled(bar, egui::Rounding::same(1.0), p.line);
    let k = if max <= 0 { 0.0 } else { (used as f32 / max as f32).clamp(0.0, 1.0) };
    let col = if k >= 0.95 {
        p.danger
    } else if k >= 0.8 {
        p.warn
    } else {
        p.signal
    };
    if k > 0.0 {
        painter.rect_filled(Rect::from_min_size(bar.min, Vec2::new((w * k).max(2.0), 4.0)), egui::Rounding::same(1.0), col);
    }
}

pub fn megabytes(bytes: i64) -> String {
    let mb = bytes as f64 / 1_048_576.0;
    if mb >= 1024.0 {
        format!("{:.1} GB", mb / 1024.0)
    } else if mb >= 10.0 {
        format!("{mb:.0} MB")
    } else {
        format!("{mb:.1} MB")
    }
}

fn field(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) -> egui::Response {
    ui::field(ui, text, hint, width - 24.0)
}

fn message(ui: &mut egui::Ui, error: &Option<String>, note: &Option<String>) {
    let p = pal();
    if let Some(e) = error {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(e).font(theme::font(theme::T_CAP)).color(p.danger_hi));
    } else if let Some(n) = note {
        ui.add_space(4.0);
        ui.label(egui::RichText::new(n).font(theme::font(theme::T_CAP)).color(p.text_2));
    }
}

fn file_name(title: &str) -> String {
    let clean: String = title.chars().map(|c| if c.is_alphanumeric() || " -_.,'()".contains(c) { c } else { '-' }).collect();
    let clean = clean.trim().trim_matches('.').to_string();
    if clean.is_empty() {
        "Untitled".into()
    } else {
        clean.chars().take(80).collect()
    }
}

pub fn write_export(dir: &Path, spaces: &[Workspace], projects: &[Project]) -> Result<usize, String> {
    let mut n = 0;
    for p in projects {
        let space = spaces.iter().find(|w| w.id == p.workspace_id).map(Workspace::shown).unwrap_or_else(|| "Other".into());
        let mut folder = dir.join(file_name(&space));
        if p.in_trash() {
            folder = folder.join("Recently deleted");
        }
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
        let ext = if p.app_id == "blackline" { "md" } else { "txt" };
        let base = file_name(&p.title);
        let mut path = folder.join(format!("{base}.{ext}"));
        let mut k = 2;
        while path.exists() {
            path = folder.join(format!("{base} {k}.{ext}"));
            k += 1;
        }
        std::fs::write(&path, p.body_text()).map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(n)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    SignedOut,
    Waiting,
    SignedIn,
}

pub struct Loaded {
    pub me: Me,
    pub spaces: Vec<Space>,
    pub orgs: Vec<OrgView>,
    pub invites: Vec<Invite>,
}

pub enum Msg {
    Signed(Result<(), CloudError>),
    Loaded(Result<Box<Loaded>, CloudError>),
    Done(String, Result<(), CloudError>),
    Invited(String, Result<bool, CloudError>),
    SignedOut(Result<(), CloudError>),
    Web(String),
    Pulled(Result<Option<Settings>, CloudError>),
    Pushed(Result<(), CloudError>),
}

pub struct Account {
    info: AppInfo,
    engine: Engine<Msg>,
    ctx: Option<egui::Context>,
    pub me: Option<Me>,
    pub spaces: Vec<Space>,
    pub orgs: Vec<OrgView>,
    pub invites: Vec<Invite>,
    pub sync_settings: bool,
    browser: Option<BrowserSignIn>,
    code: String,
    error: Option<String>,
    note: Option<String>,
    busy: bool,
    org: Option<String>,
    invite_email: String,
    invite_role: OrgRole,
    notices: Vec<(String, String, Tone)>,
    asks: Vec<Ask>,
    urls: Vec<String>,
    changed: Option<bool>,
    pulled: Option<String>,
    last_push: String,
}

pub fn plan_name(id: &str) -> String {
    match id {
        "free" => "Free".into(),
        "team" => "Team".into(),
        "studio" => "Studio".into(),
        "enterprise" => "Enterprise".into(),
        other => {
            let mut c = other.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        }
    }
}

fn countdown(left: std::time::Duration) -> String {
    let s = left.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

impl Account {
    pub fn new(info: AppInfo, cfg: CloudConfig, ctx: Option<&egui::Context>) -> Account {
        let folder = crate::platform::app_folder(info.name);
        let client = Client::for_app(cfg, &info);
        let device = Device::load_or_create(&folder);
        let engine = Engine::spawn(Worker::new(client, device, Outbox::memory(), Heartbeat::off()), ctx.cloned());
        let mut a = Account::with_engine(info, engine);
        a.ctx = ctx.cloned();
        a
    }

    pub fn with_engine(info: AppInfo, engine: Engine<Msg>) -> Account {
        let mut a = Account {
            info,
            engine,
            ctx: None,
            me: None,
            spaces: Vec::new(),
            orgs: Vec::new(),
            invites: Vec::new(),
            sync_settings: false,
            browser: None,
            code: String::new(),
            error: None,
            note: None,
            busy: false,
            org: None,
            invite_email: String::new(),
            invite_role: OrgRole::Member,
            notices: Vec::new(),
            asks: Vec::new(),
            urls: Vec::new(),
            changed: None,
            pulled: None,
            last_push: String::new(),
        };
        if a.signed_in() {
            a.after_sign_in(false);
        }
        a
    }

    pub fn client(&self) -> Client {
        self.engine.client().clone()
    }

    pub fn device(&self) -> &Device {
        self.engine.device()
    }

    pub fn signed_in(&self) -> bool {
        self.engine.client().signed_in()
    }

    pub fn stage(&self) -> Stage {
        if self.signed_in() {
            Stage::SignedIn
        } else if self.browser.is_some() {
            Stage::Waiting
        } else {
            Stage::SignedOut
        }
    }

    pub fn busy(&self) -> bool {
        self.busy
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn login_url(&self) -> Option<&str> {
        self.browser.as_ref().map(|b| b.url.as_str())
    }

    pub fn sign_in_port(&self) -> Option<u16> {
        self.browser.as_ref().map(|b| b.port)
    }

    pub fn sign_in_challenge(&self) -> Option<&str> {
        self.browser.as_ref().map(|b| b.pkce.challenge.as_str())
    }

    pub fn sign_in_state(&self) -> Option<&str> {
        self.browser.as_ref().map(|b| b.pkce.state.as_str())
    }

    pub fn selected_org(&self) -> Option<&str> {
        self.org.as_deref()
    }

    pub fn select_org(&mut self, org: Option<&str>) {
        self.org = org.map(str::to_string);
    }

    pub fn take_changed(&mut self) -> Option<bool> {
        self.changed.take()
    }

    pub fn take_notices(&mut self) -> Vec<(String, String, Tone)> {
        std::mem::take(&mut self.notices)
    }

    pub fn take_asks(&mut self) -> Vec<Ask> {
        std::mem::take(&mut self.asks)
    }

    pub fn take_urls(&mut self) -> Vec<String> {
        std::mem::take(&mut self.urls)
    }

    pub fn take_pulled_settings(&mut self) -> Option<String> {
        self.pulled.take()
    }

    pub fn set_code(&mut self, code: &str) {
        self.code = code.into();
    }

    pub fn set_invite(&mut self, email: &str, role: OrgRole) {
        self.invite_email = email.into();
        self.invite_role = role;
    }

    fn ask(&mut self, title: &str, body: &str, yes: &str, danger: bool, action: String) {
        self.asks.push(Ask { title: title.into(), body: body.into(), yes: yes.into(), danger, action });
    }

    fn say(&mut self, title: &str, body: &str, tone: Tone) {
        self.notices.push((title.into(), body.into(), tone));
    }

    fn after_sign_in(&mut self, fresh: bool) {
        let device = self.device().clone();
        self.engine.run(move |c| Msg::Done("device".into(), c.register_device(&device)));
        self.reload();
        if self.sync_settings {
            self.pull_settings();
        }
        if fresh {
            self.changed = Some(true);
            self.code.clear();
            self.browser = None;
        }
    }

    pub fn reload(&mut self) {
        self.engine.run(|c| {
            let load = || -> Result<Box<Loaded>, CloudError> { Ok(Box::new(Loaded { me: c.me()?, spaces: c.spaces()?, orgs: c.orgs()?, invites: c.my_invites()? })) };
            Msg::Loaded(load())
        });
    }

    pub fn pull_settings(&mut self) {
        let app = self.info.id.to_string();
        self.engine.run(move |c| Msg::Pulled(c.app_settings(&app)));
    }

    pub fn push_settings(&mut self, text: &str) {
        if !self.sync_settings || !self.signed_in() || text == self.last_push {
            return;
        }
        self.last_push = text.to_string();
        let (app, body) = (self.info.id.to_string(), text.to_string());
        self.engine.run(move |c| Msg::Pushed(c.put_app_settings(&app, &body)));
    }

    pub fn poll(&mut self) {
        if let Some(cb) = self.browser.as_mut().and_then(BrowserSignIn::poll) {
            match cb {
                Callback::Code(code) => self.redeem(code),
                Callback::Cancelled => {
                    self.browser = None;
                    self.busy = false;
                    self.note = Some("Signing in was cancelled. Nothing changed.".into());
                }
                Callback::TimedOut => {
                    self.browser = None;
                    self.busy = false;
                    self.error = Some("Signing in took too long. Start again when you're ready.".into());
                }
            }
        }
        for e in self.engine.poll() {
            if let Event::Reply(m) = e {
                self.handle(m);
            }
        }
    }

    // signing in here---

    pub fn sign_in_with_browser(&mut self, signup: bool) {
        self.error = None;
        self.note = None;
        self.code.clear();
        let wake = self.ctx.clone();
        let cfg = self.engine.client().config().clone();
        match BrowserSignIn::start(&cfg, &self.info, &self.device().name, signup, move || {
            if let Some(c) = &wake {
                c.request_repaint();
            }
        }) {
            Ok(b) => {
                self.urls.push(b.url.clone());
                self.browser = Some(b);
            }
            Err(e) => self.error = Some(format!("This app couldn't get ready to sign in ({e}). Try again.")),
        }
    }

    pub fn cancel_sign_in(&mut self) {
        if let Some(b) = self.browser.take() {
            b.cancel();
        }
        self.busy = false;
        self.error = None;
        self.code.clear();
    }

    pub fn reopen_sign_in(&mut self) {
        if let Some(u) = self.login_url().map(str::to_string) {
            self.urls.push(u);
        }
    }

    pub fn submit_code(&mut self) {
        self.error = None;
        let code: String = self.code.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>().to_uppercase();
        if code.len() != 8 {
            self.error = Some("Enter the 8-character code the web page shows.".into());
            return;
        }
        if self.browser.is_none() {
            self.error = Some("Start signing in first, then enter the code the web page shows.".into());
            return;
        }
        self.redeem(code);
    }

    fn redeem(&mut self, code: String) {
        let Some(verifier) = self.browser.as_ref().map(|b| b.verifier().to_string()) else {
            return;
        };
        self.busy = true;
        self.engine.run(move |c| Msg::Signed(c.redeem_code(&code, &verifier).map(|_| ())));
    }

    pub fn open_on_web(&mut self, path: &str) {
        let path = path.to_string();
        if !self.signed_in() {
            self.urls.push(self.engine.client().config().web(&path));
            return;
        }
        self.engine.run(move |c| Msg::Web(c.open_on_web_url(&path)));
    }

    pub fn sign_out(&mut self) {
        self.busy = true;
        self.engine.run(|c| Msg::SignedOut(c.sign_out(false)));
    }

    fn signed_out(&mut self) {
        self.me = None;
        self.spaces.clear();
        self.orgs.clear();
        self.invites.clear();
        self.org = None;
        self.changed = Some(false);
    }

    fn handle(&mut self, m: Msg) {
        match m {
            Msg::Signed(Ok(())) => {
                self.busy = false;
                self.error = None;
                self.after_sign_in(true);
                self.say("Signed in", "This computer syncs with NativeCloud now.", Tone::Ok);
            }
            Msg::Signed(Err(e)) => {
                self.busy = false;
                self.code.clear();
                self.error = Some(match e {
                    CloudError::NotFound | CloudError::Forbidden | CloudError::Invalid(_) => {
                        "That code is wrong, used or expired. Check it, or start again from this app.".to_string()
                    }
                    other => other.message(),
                });
            }
            Msg::Loaded(Ok(l)) => {
                self.busy = false;
                self.me = Some(l.me);
                self.spaces = l.spaces;
                self.orgs = l.orgs;
                self.invites = l.invites;
                let known = self.org.as_ref().map(|o| self.orgs.iter().any(|x| &x.org.id == o)).unwrap_or(false);
                if !known {
                    self.org = self.orgs.first().map(|o| o.org.id.clone());
                }
            }
            Msg::Done(what, Ok(())) => {
                self.busy = false;
                self.error = None;
                match what.as_str() {
                    "device" => {}
                    "joined" => self.say("You joined", "Its spaces are in your library now.", Tone::Ok),
                    "declined" => self.say("Invite declined", "Nothing changed for you.", Tone::Info),
                    _ => {}
                }
                if what != "device" {
                    self.changed = Some(true);
                    self.reload();
                }
            }
            Msg::Invited(email, Ok(sent)) => {
                self.busy = false;
                self.invite_email.clear();
                if sent {
                    self.say("Invite sent", &format!("{email} gets an email with a link to join."), Tone::Ok);
                } else {
                    self.say("Invite saved", &format!("Ask {email} to sign in to NativeCloud with that address. The invite is waiting for them."), Tone::Info);
                }
                self.reload();
            }
            Msg::Web(url) => self.urls.push(url),
            Msg::SignedOut(r) => {
                self.busy = false;
                self.signed_out();
                if let Err(e) = r {
                    if !e.is_offline() {
                        self.say("Signed out here", &e.message(), Tone::Warn);
                    }
                }
            }
            Msg::Pulled(Ok(Some(s))) => {
                self.last_push = s.body.clone();
                self.pulled = Some(s.body);
            }
            Msg::Pulled(Ok(None)) => self.last_push.clear(),
            Msg::Pulled(Err(_)) | Msg::Pushed(Ok(())) => {}
            Msg::Pushed(Err(_)) => self.last_push.clear(),
            Msg::Loaded(Err(e)) | Msg::Done(_, Err(e)) | Msg::Invited(_, Err(e)) => {
                self.busy = false;
                if e == CloudError::SignedOut && self.me.is_some() {
                    self.signed_out();
                    self.say(e.title(), &e.message(), Tone::Warn);
                } else if self.signed_in() && self.me.is_some() {
                    self.say(e.title(), &e.message(), if e.is_offline() { Tone::Warn } else { Tone::Danger });
                } else {
                    self.error = Some(e.message());
                }
            }
        }
    }

    pub fn answer(&mut self, action: &str) -> bool {
        let parts: Vec<&str> = action.split(':').collect();
        match parts.as_slice() {
            ["ncf-cloud-decline", id] => {
                let id = id.to_string();
                self.act("declined", move |c| c.decline_invite(&id));
            }
            ["ncf-cloud-signout"] => self.sign_out(),
            _ => return false,
        }
        true
    }

    fn act(&mut self, what: &str, f: impl FnOnce(&Client) -> Result<(), CloudError> + Send + 'static) {
        self.busy = true;
        let what = what.to_string();
        self.engine.run(move |c| Msg::Done(what, f(c)));
    }

    pub fn join(&mut self, invite: &str) {
        let id = invite.to_string();
        self.act("joined", move |c| c.accept_invite(&id).map(|_| ()));
    }

    pub fn send_invite(&mut self) {
        let Some(org) = self.org.clone() else {
            return;
        };
        let email = self.invite_email.trim().to_lowercase();
        if !email_ok(&email) {
            self.say("Check that", &CloudError::Invalid("email".into()).message(), Tone::Warn);
            return;
        }
        self.busy = true;
        let (role, shown) = (self.invite_role, email.clone());
        self.engine.run(move |c| Msg::Invited(shown, c.invite_member(&org, &email, role, &[]).map(|id| c.send_invite(&id).unwrap_or(false))));
    }

    // signed out here---

    fn sign_in_card(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        let w = ui.available_width().min(380.0);
        let web = self.engine.client().config().web_url.trim_start_matches("https://").to_string();
        ui::section(ui, "NativeCloud");
        match self.browser.as_ref().map(BrowserSignIn::left) {
            None => {
                ui.label(egui::RichText::new("Sign in").font(theme::font_semi(theme::T_H)).color(p.text));
                ui.add_space(2.0);
                ui::caption(ui, &format!("One account for every NativeCompute app. You sign in on {web} in your browser, then come straight back here."));
                ui.add_space(12.0);
                if ui::button_sized(ui, "Sign in with your browser", Some(Icon::ArrowRight), true, Some(w)) {
                    self.sign_in_with_browser(false);
                }
                ui.add_space(6.0);
                if ui::button_sized(ui, "Create an account", Some(Icon::Plus), false, Some(w)) {
                    self.sign_in_with_browser(true);
                }
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui::caption(ui, "Passwords are only ever typed on the web. Read the");
                    if ui::link(ui, "terms") {
                        self.urls.push(TERMS_URL.into());
                    }
                    ui::caption(ui, "and");
                    if ui::link(ui, "privacy policy") {
                        self.urls.push(PRIVACY_URL.into());
                    }
                });
            }
            Some(left) => {
                ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));
                ui.label(egui::RichText::new("Finish in your browser").font(theme::font_semi(theme::T_H)).color(p.text));
                ui.add_space(2.0);
                ui::caption(ui, &format!("We opened {web}. Sign in there and press Continue. You come back here signed in."));
                ui.add_space(6.0);
                let tag = if self.busy { "SIGNING IN…".to_string() } else { format!("WAITING · {}", countdown(left)) };
                ui::hud(ui, &tag, Some(p.signal_hi));
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui::button(ui, "Open the page again", Some(Icon::ArrowRight), false) {
                        self.reopen_sign_in();
                    }
                    if ui::quiet_button(ui, "Cancel", None) {
                        self.cancel_sign_in();
                    }
                });
                ui::separator(ui);
                ui::caption(ui, "Signing in on another device, or the page shows a code? Type it here.");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut self.code)
                            .hint_text("ABCD-2345")
                            .char_limit(9)
                            .font(theme::font_mono(theme::T_H))
                            .desired_width(170.0)
                            .margin(egui::Margin::symmetric(12.0, 8.0)),
                    );
                    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if (ui::button(ui, "Use code", Some(Icon::Check), true) || enter) && !self.busy {
                        self.submit_code();
                    }
                });
            }
        }
        message(ui, &self.error, &self.note);
    }

    // signed in here---

    pub fn account_ui(&mut self, ui: &mut egui::Ui) {
        if !self.signed_in() {
            if self.me.is_some() {
                self.signed_out();
            }
            self.sign_in_card(ui);
            return;
        }
        let p = pal();
        let Some(me) = self.me.clone() else {
            ui::section(ui, "NativeCloud");
            ui::caption(ui, "Loading your account…");
            if let Some(e) = self.error.clone() {
                ui::caption(ui, &e);
                if ui::button(ui, "Try again", Some(Icon::Refresh), false) {
                    self.error = None;
                    self.reload();
                }
            }
            return;
        };

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            avatar(ui, &me.shown(), me.avatar_hue, 44.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(egui::RichText::new(me.shown()).font(theme::font_semi(theme::T_H)).color(p.text));
                ui.label(egui::RichText::new(&me.email).font(theme::font(theme::T_SM)).color(p.text_2));
                let orgs = me.orgs.len();
                let tag = match orgs {
                    0 => "My cloud".to_string(),
                    1 => format!("My cloud · {}", me.orgs[0].name),
                    n => format!("My cloud · {n} organizations"),
                };
                ui::hud(ui, &tag, None);
            });
        });
        ui.add_space(4.0);

        self.invites_banner(ui);

        ui::separator(ui);
        ui::section(ui, "My cloud");
        let (plan, used) = (&me.plan, &me.usage);
        meter(ui, "Cloud scripts", used.projects, plan.max_projects, &format!("{} OF {}", used.projects, plan.max_projects));
        meter(ui, "Storage", used.bytes, plan.max_bytes, &format!("{} OF {}", megabytes(used.bytes), megabytes(plan.max_bytes)));
        ui::caption(ui, &format!("{} plan. Each organization has its own plan for the scripts you write together.", plan_name(&plan.id)));

        ui::separator(ui);
        ui::section(ui, "Sync");
        let mut on = self.sync_settings;
        if ui::toggle_row(ui, "Sync my settings", "Keeps appearance and shortcuts the same on every computer you sign in on.", &mut on) {
            self.sync_settings = on;
            if on {
                self.last_push.clear();
                self.pull_settings();
            }
        }

        ui::separator(ui);
        ui::section(ui, "This computer");
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&self.device().name).font(theme::font_med(theme::T_SM)).color(p.text));
            ui::hud(ui, "Signed in", Some(p.ok));
        });
        ui::caption(ui, "Rename your computers or sign them out from the web.");

        ui::separator(ui);
        ui::section(ui, "Your account");
        ui::caption(ui, "Your profile, email, password, computers, data export and deleting your account are all on the web.");
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui::button(ui, "Manage account on the web", Some(Icon::ArrowRight), true) && !self.busy {
                self.open_on_web("/account");
            }
            if ui::button(ui, "Sign out", Some(Icon::SignOut), false) && !self.busy {
                self.sign_out();
            }
        });
        if me.staff.is_some() {
            ui.add_space(6.0);
            if ui::link(ui, "Open the staff console") {
                self.open_on_web("/staff");
            }
        }
        message(ui, &self.error, &None);
    }

    fn invites_banner(&mut self, ui: &mut egui::Ui) {
        if self.invites.is_empty() {
            return;
        }
        ui::separator(ui);
        ui::section(ui, "Invites for you");
        for inv in self.invites.clone() {
            let who = if inv.invited_by.trim().is_empty() { "Someone".to_string() } else { inv.invited_by.clone() };
            ui::body(ui, &format!("{who} invited you to {} as {}.", inv.org, inv.role.label().to_lowercase()));
            ui.horizontal(|ui| {
                if ui::button(ui, "Join", Some(Icon::Check), true) && !self.busy {
                    self.join(&inv.id);
                }
                if ui::quiet_button(ui, "Decline", None) && !self.busy {
                    self.ask(&format!("Decline the invite to {}?", inv.org), "They can invite you again later.", "Decline", false, format!("ncf-cloud-decline:{}", inv.id));
                }
            });
            ui.add_space(4.0);
        }
    }

    // organizations here---

    pub fn orgs_ui(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        if !self.signed_in() {
            ui::section(ui, "Organizations");
            ui::caption(ui, "Sign in to NativeCloud to write with a team. Everyone sees the same scripts, one person writes at a time and the rest watch it change.");
            ui.add_space(8.0);
            if ui::button(ui, "Sign in", Some(Icon::ArrowRight), true) {
                self.urls.push("ncf:settings:Account".into());
            }
            return;
        }
        let w = ui.available_width().min(460.0);
        self.invites_banner(ui);
        ui::section(ui, "Your organizations");
        if self.orgs.is_empty() {
            ui::caption(ui, "No organizations yet. Create one on the web, then invite people by email.");
        }
        for o in self.orgs.clone() {
            let selected = self.org.as_deref() == Some(o.org.id.as_str());
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 36.0), Sense::click());
            let hot = crate::anim::ease(ui.ctx(), resp.id, resp.hovered() || selected, crate::anim::HOVER);
            theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, selected);
            icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 16.0, rect.center().y), Vec2::splat(14.0)), Icon::Users, p.text_2);
            ui.painter().text(Pos2::new(rect.left() + 34.0, rect.center().y), egui::Align2::LEFT_CENTER, &o.org.name, theme::font_med(theme::T_SM), p.text);
            let n = o.members.len();
            let tag = format!("{} · {} · {n} {}", o.org.role.label(), plan_name(&o.org.plan), if n == 1 { "person" } else { "people" }).to_uppercase();
            let tw = theme::tracked_width(ui.ctx(), &tag, theme::font_hud(theme::T_MICRO), 0.8);
            theme::tracked_text(ui.painter(), Pos2::new(rect.right() - 12.0 - tw, rect.center().y), &tag, theme::font_hud(theme::T_MICRO), p.text_3, 0.8);
            if resp.clicked() {
                self.org = Some(o.org.id.clone());
            }
        }
        ui.add_space(6.0);
        if ui::quiet_button(ui, "Create an organization on the web", Some(Icon::Plus)) {
            self.open_on_web("/new");
        }

        let Some(o) = self.orgs.iter().find(|o| Some(&o.org.id) == self.org.as_ref()).cloned() else {
            return;
        };
        let me = self.engine.client().user_id();
        let manage = o.org.role.can_manage();
        ui::separator(ui);
        ui.label(egui::RichText::new(o.org.name.to_uppercase()).font(theme::font_display(theme::T_H + 6.0)).color(p.text));
        ui.add_space(2.0);
        let mut line = format!("/o/{} · {} plan", o.org.slug, plan_name(&o.org.plan));
        if let Some(t) = o.org.trial_ends_at.as_deref().filter(|t| t.len() >= 10) {
            line.push_str(&format!(" · trial until {}", &t[..10]));
        }
        if o.org.suspended {
            line.push_str(" · suspended");
        }
        ui::hud(ui, &line, if o.org.suspended { Some(p.warn) } else { None });

        ui::section(ui, "Spaces");
        for space in &o.spaces {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&space.name).font(theme::font_med(theme::T_SM)).color(p.text));
                let role = self.spaces.iter().find(|s| s.workspace.id == space.id).map(|s| s.my_role.label()).unwrap_or("Not in it");
                let tag = if space.archived_at.is_some() { format!("{role} · archived") } else { role.to_string() };
                ui::hud(ui, &tag, None);
            });
        }

        ui::section(ui, "People");
        for (m, person) in &o.members {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                avatar(ui, &person.shown(), person.hue, 26.0);
                let you = if m.user_id == me { " (you)" } else { "" };
                ui.label(egui::RichText::new(format!("{}{you}", person.shown())).font(theme::font_med(theme::T_SM)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui::hud(ui, m.role.label(), None);
                });
            });
            ui.add_space(2.0);
        }
        if o.org.role == OrgRole::Guest {
            ui::caption(ui, "Guests see only the people in their spaces.");
        }

        if manage {
            if !o.invites.is_empty() {
                ui.add_space(4.0);
                let n = o.invites.len();
                ui::hud(ui, &format!("{n} waiting to join"), Some(p.signal_hi));
            }
            ui.add_space(6.0);
            ui::section(ui, "Quick invite");
            field(ui, &mut self.invite_email, "name@example.com", w);
            ui.add_space(6.0);
            let roles: Vec<OrgRole> = [OrgRole::Member, OrgRole::Admin].into_iter().filter(|r| *r != OrgRole::Admin || o.org.role == OrgRole::Owner).collect();
            if !roles.contains(&self.invite_role) {
                self.invite_role = OrgRole::Member;
            }
            if roles.len() > 1 {
                let labels: Vec<&str> = roles.iter().map(|r| r.label()).collect();
                let current = roles.iter().position(|r| *r == self.invite_role).unwrap_or(0);
                if let Some(k) = ui::tabs(ui, &labels, current) {
                    self.invite_role = roles[k];
                }
            }
            ui::caption(ui, &format!("{} Guests, chosen spaces and many emails at once are on the web.", self.invite_role.hint()));
            ui.add_space(6.0);
            if ui::button(ui, "Send invite", Some(Icon::ArrowRight), true) && !self.busy {
                self.send_invite();
            }
        }

        ui::separator(ui);
        ui.horizontal_wrapped(|ui| {
            let slug = o.org.slug.clone();
            if ui::button(ui, if manage { "Manage members on the web" } else { "Open on the web" }, Some(Icon::ArrowRight), false) && !self.busy {
                self.open_on_web(&if manage { format!("/o/{slug}/members") } else { format!("/o/{slug}") });
            }
            if manage && ui::quiet_button(ui, "Settings on the web", Some(Icon::Settings)) {
                self.open_on_web(&format!("/o/{slug}/settings"));
            }
        });
    }
}

pub fn apply_synced(text: &str, mut read: impl FnMut(&str, &str), app_keys: &[&str]) {
    for (k, v) in crate::settings::pairs(text) {
        if SYNCED_LOOK_KEYS.contains(&k.as_str()) || app_keys.contains(&k.as_str()) {
            read(&k, &v);
        }
    }
}

pub fn synced_text(full: &str, app_keys: &[&str]) -> String {
    crate::settings::pairs(full)
        .into_iter()
        .filter(|(k, _)| SYNCED_LOOK_KEYS.contains(&k.as_str()) || app_keys.contains(&k.as_str()))
        .map(|(k, v)| format!("{k} = {v}\n"))
        .collect()
}
