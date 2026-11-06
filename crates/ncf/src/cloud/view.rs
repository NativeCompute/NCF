use std::path::{Path, PathBuf};

use eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, Sense, Stroke, Vec2};

use super::engine::Mood;
use super::*;
use crate::alerts::Tone;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};
use crate::{ui, AppInfo};

pub const SYNCED_LOOK_KEYS: [&str; 5] = ["light_mode", "animations", "font_scale", "splash", "check_updates"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    SignIn,
    SignUp,
    Code(Code),
    Forgot,
    NewPassword,
}

pub struct Loaded {
    pub me: Me,
    pub teams: Vec<Team>,
    pub invites: Vec<Invite>,
    pub devices: Vec<DeviceRow>,
}

pub enum Msg {
    Signed(Result<(), CloudError>),
    SignedUp(Result<SignUp, CloudError>),
    Verified(Code, Result<(), CloudError>),
    Sent(Code, Result<(), CloudError>),
    Loaded(Result<Box<Loaded>, CloudError>),
    Done(String, Result<(), CloudError>),
    Invited(String, Result<bool, CloudError>),
    Created(Result<String, CloudError>),
    Exported(Result<(PathBuf, usize), String>),
    Deleted(Result<(), CloudError>),
    SignedOut(Result<(), CloudError>),
    Pulled(Result<Option<Settings>, CloudError>),
    Pushed(Result<(), CloudError>),
}

pub struct Ask {
    pub title: String,
    pub body: String,
    pub yes: String,
    pub danger: bool,
    pub action: String,
}

pub struct Account {
    info: AppInfo,
    engine: Engine<Msg>,
    pub me: Option<Me>,
    pub teams: Vec<Team>,
    pub invites: Vec<Invite>,
    pub devices: Vec<DeviceRow>,
    pub sync_settings: bool,
    panel: Panel,
    email: String,
    password: String,
    name: String,
    code: String,
    show_password: bool,
    error: Option<String>,
    note: Option<String>,
    busy: bool,
    profile_name: String,
    profile_hue: i32,
    new_email: String,
    email_code: String,
    email_pending: Option<String>,
    new_password: String,
    reauth: Option<String>,
    team: Option<String>,
    new_team: String,
    rename: String,
    invite_email: String,
    invite_role: Role,
    role_menu: Option<String>,
    notices: Vec<(String, String, Tone)>,
    asks: Vec<Ask>,
    urls: Vec<String>,
    changed: Option<bool>,
    pulled: Option<String>,
    last_push: String,
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

fn role_icon(r: Role) -> Icon {
    match r {
        Role::Owner | Role::Admin => Icon::Settings,
        Role::Editor => Icon::Pencil,
        Role::Viewer => Icon::Eye,
    }
}

fn secret(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32, shown: &mut bool) -> egui::Response {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let r = ui.add(egui::TextEdit::singleline(text).hint_text(hint).password(!*shown).desired_width(width - 56.0).margin(egui::Margin::symmetric(12.0, 8.0)));
        if ui::icon_button(ui, Icon::Eye, if *shown { "Hide password" } else { "Show password" }, 28.0) {
            *shown = !*shown;
        }
        r
    })
    .inner
}

fn rules(ui: &mut egui::Ui, password: &str) {
    let p = pal();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;
        for (label, ok) in password_rules(password) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                let (r, _) = ui.allocate_exact_size(Vec2::splat(11.0), Sense::hover());
                let col = if ok { p.ok } else { p.text_3 };
                if ok {
                    icons::draw(ui.painter(), r, Icon::Check, col);
                } else {
                    ui.painter().circle_stroke(r.center(), 3.0, Stroke::new(1.0_f32, col));
                }
                ui::hud(ui, label, Some(col));
            });
        }
    });
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

fn export_dir() -> PathBuf {
    let base = dirs::download_dir().or_else(dirs::home_dir).unwrap_or_else(std::env::temp_dir);
    let day = time::format_time(unix_now());
    base.join(format!("NativeCloud export {}", &day[..10]))
}

impl Account {
    pub fn new(info: AppInfo, cfg: CloudConfig, ctx: Option<&egui::Context>) -> Account {
        let folder = crate::platform::app_folder(info.name);
        let client = Client::for_app(cfg, &info);
        let device = Device::load_or_create(&folder);
        let engine = Engine::spawn(Worker::new(client, device, Outbox::memory(), Heartbeat::off()), ctx.cloned());
        Account::with_engine(info, engine)
    }

    pub fn with_engine(info: AppInfo, engine: Engine<Msg>) -> Account {
        let mut a = Account {
            info,
            engine,
            me: None,
            teams: Vec::new(),
            invites: Vec::new(),
            devices: Vec::new(),
            sync_settings: false,
            panel: Panel::SignIn,
            email: String::new(),
            password: String::new(),
            name: String::new(),
            code: String::new(),
            show_password: false,
            error: None,
            note: None,
            busy: false,
            profile_name: String::new(),
            profile_hue: 0,
            new_email: String::new(),
            email_code: String::new(),
            email_pending: None,
            new_password: String::new(),
            reauth: None,
            team: None,
            new_team: String::new(),
            rename: String::new(),
            invite_email: String::new(),
            invite_role: Role::Editor,
            role_menu: None,
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

    pub fn busy(&self) -> bool {
        self.busy
    }

    pub fn panel(&self) -> Panel {
        self.panel
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn selected_team(&self) -> Option<&str> {
        self.team.as_deref()
    }

    pub fn select_team(&mut self, ws: Option<&str>) {
        self.team = ws.map(str::to_string);
        self.rename = self.teams.iter().find(|t| Some(t.workspace.id.as_str()) == ws).map(|t| t.workspace.name.clone()).unwrap_or_default();
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

    pub fn show(&mut self, panel: Panel) {
        self.panel = panel;
        self.error = None;
        self.note = None;
    }

    pub fn fill(&mut self, email: &str, password: &str, name: &str) {
        self.email = email.into();
        self.password = password.into();
        self.name = name.into();
    }

    pub fn set_code(&mut self, code: &str) {
        self.code = code.into();
    }

    pub fn set_invite(&mut self, email: &str, role: Role) {
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
            self.password.clear();
            self.code.clear();
            self.panel = Panel::SignIn;
        }
    }

    pub fn reload(&mut self) {
        self.engine.run(|c| {
            let load = || -> Result<Box<Loaded>, CloudError> { Ok(Box::new(Loaded { me: c.me()?, teams: c.teams()?, invites: c.my_invites()?, devices: c.devices()? })) };
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
        for e in self.engine.poll() {
            if let Event::Reply(m) = e {
                self.handle(m);
            }
        }
    }

    fn signed_out(&mut self) {
        self.me = None;
        self.teams.clear();
        self.invites.clear();
        self.devices.clear();
        self.team = None;
        self.changed = Some(false);
        self.panel = Panel::SignIn;
    }

    fn handle(&mut self, m: Msg) {
        match m {
            Msg::Signed(Ok(())) => {
                self.busy = false;
                self.error = None;
                self.after_sign_in(true);
            }
            Msg::Signed(Err(CloudError::Unconfirmed)) => {
                self.busy = false;
                let email = self.email.trim().to_lowercase();
                self.engine.run(move |c| Msg::Sent(Code::SignUp, c.resend(Code::SignUp, &email)));
                self.show(Panel::Code(Code::SignUp));
                self.note = Some("Confirm your email first. We've sent you a new code.".into());
            }
            Msg::SignedUp(Ok(SignUp::NeedsCode)) => {
                self.busy = false;
                self.show(Panel::Code(Code::SignUp));
            }
            Msg::SignedUp(Ok(SignUp::SignedIn)) => {
                self.busy = false;
                self.after_sign_in(true);
            }
            Msg::Verified(Code::Recovery, Ok(())) => {
                self.busy = false;
                self.changed = Some(true);
                self.show(Panel::NewPassword);
                self.reload();
            }
            Msg::Verified(Code::EmailChange, Ok(())) => {
                self.busy = false;
                self.email_pending = None;
                self.email_code.clear();
                self.new_email.clear();
                self.say("Email changed", "Use your new email next time you sign in.", Tone::Ok);
                self.reload();
            }
            Msg::Verified(Code::SignUp, Ok(())) => {
                self.busy = false;
                self.say("Welcome to NativeCloud", "Your account is ready.", Tone::Ok);
                self.after_sign_in(true);
            }
            Msg::Sent(Code::EmailChange, Ok(())) => {
                self.busy = false;
                self.email_pending = Some(self.new_email.trim().to_lowercase());
            }
            Msg::Sent(Code::Recovery, Ok(())) => {
                self.busy = false;
                self.show(Panel::Code(Code::Recovery));
            }
            Msg::Sent(Code::SignUp, Ok(())) => {
                self.busy = false;
                if self.note.is_none() {
                    self.note = Some("A new code is on its way.".into());
                }
            }
            Msg::Loaded(Ok(l)) => {
                self.busy = false;
                self.profile_name = l.me.display_name.clone();
                self.profile_hue = l.me.avatar_hue;
                self.me = Some(l.me);
                self.teams = l.teams;
                self.invites = l.invites;
                self.devices = l.devices;
                if let Some(t) = &self.team {
                    if !self.teams.iter().any(|x| &x.workspace.id == t) {
                        self.team = None;
                    }
                }
            }
            Msg::Done(what, Ok(())) => {
                self.busy = false;
                self.error = None;
                match what.as_str() {
                    "device" => {}
                    "password" => {
                        self.new_password.clear();
                        if self.panel == Panel::NewPassword {
                            self.panel = Panel::SignIn;
                        }
                        self.say("Password changed", "Use it next time you sign in.", Tone::Ok);
                    }
                    "profile" => self.say("Profile saved", "Teammates see the new name.", Tone::Ok),
                    "left" => self.say("You left the team", "Its scripts are no longer on your list.", Tone::Info),
                    "joined" => self.say("You joined the team", "Its scripts are in your library now.", Tone::Ok),
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
                    self.say("Invite sent", &format!("{email} gets an email with the next steps."), Tone::Ok);
                } else {
                    self.say("Invite saved", &format!("Ask {email} to sign in to NativeCloud with that address. The invite is waiting for them."), Tone::Info);
                }
                self.reload();
            }
            Msg::Created(Ok(id)) => {
                self.busy = false;
                self.new_team.clear();
                self.team = Some(id);
                self.changed = Some(true);
                self.reload();
            }
            Msg::Exported(Ok((dir, n))) => {
                self.busy = false;
                self.say("Export ready", &format!("{n} files in {}", dir.display()), Tone::Ok);
                self.urls.push(dir.display().to_string());
            }
            Msg::Exported(Err(e)) => {
                self.busy = false;
                self.say("Couldn't export", &e, Tone::Danger);
            }
            Msg::Deleted(Ok(())) => {
                self.busy = false;
                self.reauth = None;
                self.error = None;
                self.signed_out();
                self.say("Account deleted", "Your cloud data is gone. Scripts on this computer are untouched.", Tone::Info);
            }
            Msg::Deleted(Err(CloudError::Reauth)) | Msg::Deleted(Err(CloudError::Auth(_))) if self.signed_in() => {
                self.busy = false;
                self.reauth = Some(String::new());
                self.error = Some(CloudError::Reauth.message());
            }
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
            Msg::Signed(Err(e))
            | Msg::SignedUp(Err(e))
            | Msg::Verified(_, Err(e))
            | Msg::Sent(_, Err(e))
            | Msg::Loaded(Err(e))
            | Msg::Done(_, Err(e))
            | Msg::Invited(_, Err(e))
            | Msg::Created(Err(e))
            | Msg::Deleted(Err(e)) => {
                self.busy = false;
                if e == CloudError::SignedOut && self.me.is_some() {
                    self.signed_out();
                    self.say(e.title(), &e.message(), Tone::Warn);
                } else if self.signed_in() && self.panel != Panel::NewPassword {
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
            ["ncf-cloud-signout-all"] => {
                self.busy = true;
                self.engine.run(|c| Msg::SignedOut(c.sign_out(true)));
            }
            ["ncf-cloud-delete"] => {
                self.busy = true;
                self.engine.run(|c| Msg::Deleted(c.delete_account()));
            }
            ["ncf-cloud-leave", ws] => {
                let ws = ws.to_string();
                self.act("left", move |c| c.leave_workspace(&ws));
            }
            ["ncf-cloud-delete-team", ws] => {
                let ws = ws.to_string();
                self.act("team-deleted", move |c| c.delete_workspace(&ws));
            }
            ["ncf-cloud-remove", ws, user] => {
                let (ws, user) = (ws.to_string(), user.to_string());
                self.act("removed", move |c| c.remove_member(&ws, &user));
            }
            ["ncf-cloud-transfer", ws, user] => {
                let (ws, user) = (ws.to_string(), user.to_string());
                self.act("transferred", move |c| c.transfer_ownership(&ws, &user));
            }
            ["ncf-cloud-forget", id] => {
                let id = id.to_string();
                self.act("forgot", move |c| c.forget_device(&id));
            }
            _ => return false,
        }
        true
    }

    fn act(&mut self, what: &str, f: impl FnOnce(&Client) -> Result<(), CloudError> + Send + 'static) {
        self.busy = true;
        let what = what.to_string();
        self.engine.run(move |c| Msg::Done(what, f(c)));
    }

    // signed out here---

    pub fn submit(&mut self) {
        self.error = None;
        self.note = None;
        let (email, password, name, code) = (self.email.trim().to_lowercase(), self.password.clone(), self.name.clone(), self.code.clone());
        match self.panel {
            Panel::SignIn => {
                if !email_ok(&email) || password.is_empty() {
                    self.error = Some("Enter your email and password.".into());
                    return;
                }
                self.engine.run(move |c| Msg::Signed(c.sign_in(&email, &password).map(|_| ())));
            }
            Panel::SignUp => {
                if !email_ok(&email) {
                    self.error = Some(CloudError::Invalid("email".into()).message());
                    return;
                }
                if !password_ok(&password) {
                    self.error = Some(CloudError::Auth("weak_password".into()).message());
                    return;
                }
                self.engine.run(move |c| Msg::SignedUp(c.sign_up(&email, &password, &name)));
            }
            Panel::Code(kind) => {
                if code.chars().filter(char::is_ascii_digit).count() < 6 {
                    self.error = Some("Enter the 6-digit code from the email.".into());
                    return;
                }
                self.engine.run(move |c| Msg::Verified(kind, c.verify(kind, &email, &code).map(|_| ())));
            }
            Panel::Forgot => {
                if !email_ok(&email) {
                    self.error = Some(CloudError::Invalid("email".into()).message());
                    return;
                }
                self.engine.run(move |c| Msg::Sent(Code::Recovery, c.recover(&email)));
            }
            Panel::NewPassword => {
                let pw = self.new_password.clone();
                if !password_ok(&pw) {
                    self.error = Some(CloudError::Auth("weak_password".into()).message());
                    return;
                }
                self.engine.run(move |c| Msg::Done("password".into(), c.set_password(&pw)));
            }
        }
        self.busy = true;
    }

    fn sign_in_form(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        let w = ui.available_width().min(380.0);
        let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
        let mut go = false;
        let (title, lead) = match self.panel {
            Panel::SignIn => ("Sign in", "One account for every NativeCompute app. Keep scripts in the cloud, sync them between computers and write with your team.".to_string()),
            Panel::SignUp => ("Create your account", "Free during the beta. Nothing on this computer is uploaded until you choose it.".to_string()),
            Panel::Code(_) => ("Check your email", format!("We sent a 6-digit code to {}. It works for an hour.", self.email.trim())),
            Panel::Forgot => ("Reset your password", "We'll email you a code to set a new one.".to_string()),
            Panel::NewPassword => ("Choose a new password", "You're signed in. Pick a password for next time.".to_string()),
        };
        ui::section(ui, "NativeCloud");
        ui.label(egui::RichText::new(title).font(theme::font_semi(theme::T_H)).color(p.text));
        ui.add_space(2.0);
        ui::caption(ui, &lead);
        ui.add_space(10.0);
        let mut fired = false;
        match self.panel {
            Panel::SignIn => {
                let e = field(ui, &mut self.email, "Email", w);
                ui.add_space(6.0);
                let r = secret(ui, &mut self.password, "Password", w, &mut self.show_password);
                fired |= (e.lost_focus() || r.lost_focus()) && enter;
                ui.add_space(10.0);
                go |= ui::button_sized(ui, "Sign in", Some(Icon::ArrowRight), true, Some(w));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui::link(ui, "Create an account") {
                        self.show(Panel::SignUp);
                    }
                    ui.add_space(12.0);
                    if ui::link(ui, "Forgot password?") {
                        self.show(Panel::Forgot);
                    }
                });
            }
            Panel::SignUp => {
                field(ui, &mut self.name, "Your name, as teammates see it", w);
                ui.add_space(6.0);
                field(ui, &mut self.email, "Email", w);
                ui.add_space(6.0);
                let r = secret(ui, &mut self.password, "Password", w, &mut self.show_password);
                fired |= r.lost_focus() && enter;
                ui.add_space(4.0);
                rules(ui, &self.password);
                ui.add_space(10.0);
                go |= ui::button_sized(ui, "Create account", Some(Icon::ArrowRight), true, Some(w));
                ui.add_space(6.0);
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui::caption(ui, "By creating an account you agree to the");
                    if ui::link(ui, "terms") {
                        self.urls.push(TERMS_URL.into());
                    }
                    ui::caption(ui, "and");
                    if ui::link(ui, "privacy policy") {
                        self.urls.push(PRIVACY_URL.into());
                    }
                });
                ui.add_space(4.0);
                if ui::link(ui, "I already have an account") {
                    self.show(Panel::SignIn);
                }
            }
            Panel::Code(kind) => {
                let r = ui.add(
                    egui::TextEdit::singleline(&mut self.code)
                        .hint_text("000000")
                        .char_limit(7)
                        .font(theme::font_mono(theme::T_H + 4.0))
                        .desired_width(170.0)
                        .margin(egui::Margin::symmetric(14.0, 10.0)),
                );
                fired |= r.lost_focus() && enter;
                let digits = self.code.chars().filter(char::is_ascii_digit).count();
                fired |= digits == 6 && r.changed();
                ui.add_space(10.0);
                go |= ui::button_sized(ui, "Confirm", Some(Icon::Check), true, Some(w));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui::link(ui, "Send a new code") {
                        let email = self.email.trim().to_lowercase();
                        self.note = Some("A new code is on its way.".into());
                        self.engine.run(move |c| {
                            let r = if kind == Code::Recovery { c.recover(&email) } else { c.resend(kind, &email) };
                            Msg::Sent(Code::SignUp, r)
                        });
                    }
                    ui.add_space(12.0);
                    if ui::link(ui, "Use a different email") {
                        self.show(if kind == Code::Recovery { Panel::Forgot } else { Panel::SignUp });
                    }
                });
            }
            Panel::Forgot => {
                let r = field(ui, &mut self.email, "Email", w);
                fired |= r.lost_focus() && enter;
                ui.add_space(10.0);
                go |= ui::button_sized(ui, "Send code", Some(Icon::ArrowRight), true, Some(w));
                ui.add_space(8.0);
                if ui::link(ui, "Back to sign in") {
                    self.show(Panel::SignIn);
                }
            }
            Panel::NewPassword => {
                let r = secret(ui, &mut self.new_password, "New password", w, &mut self.show_password);
                fired |= r.lost_focus() && enter;
                ui.add_space(4.0);
                rules(ui, &self.new_password);
                ui.add_space(10.0);
                go |= ui::button_sized(ui, "Set password", Some(Icon::Check), true, Some(w));
            }
        }
        message(ui, &self.error, &self.note);
        if self.busy {
            ui.add_space(4.0);
            ui::hud(ui, "Working…", Some(p.signal_hi));
        }
        if (go || fired) && !self.busy {
            self.submit();
        }
    }

    // signed in here---

    pub fn account_ui(&mut self, ui: &mut egui::Ui) {
        if !self.signed_in() {
            if self.me.is_some() {
                self.signed_out();
            }
            self.sign_in_form(ui);
            return;
        }
        if self.panel == Panel::NewPassword {
            self.sign_in_form(ui);
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
        let w = ui.available_width().min(460.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 12.0;
            avatar(ui, &me.shown(), self.profile_hue, 44.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(egui::RichText::new(me.shown()).font(theme::font_semi(theme::T_H)).color(p.text));
                ui.label(egui::RichText::new(&me.email).font(theme::font(theme::T_SM)).color(p.text_2));
                ui::hud(ui, &format!("{} · {}", me.plan.name, self.device().name), None);
            });
        });
        ui.add_space(4.0);

        if !self.invites.is_empty() {
            ui::separator(ui);
            ui::section(ui, "Invites for you");
            for inv in self.invites.clone() {
                let who = if inv.invited_by.trim().is_empty() { "Someone".to_string() } else { inv.invited_by.clone() };
                ui::body(ui, &format!("{who} invited you to {} as {}.", inv.workspace, inv.role.label().to_lowercase()));
                ui.horizontal(|ui| {
                    if ui::button(ui, "Join", Some(Icon::Check), true) && !self.busy {
                        let id = inv.id.clone();
                        self.act("joined", move |c| c.accept_invite(&id).map(|_| ()));
                    }
                    if ui::quiet_button(ui, "Decline", None) && !self.busy {
                        let id = inv.id.clone();
                        self.act("declined", move |c| c.decline_invite(&id));
                    }
                });
                ui.add_space(4.0);
            }
        }

        ui::separator(ui);
        ui::section(ui, "Profile");
        field(ui, &mut self.profile_name, "Your name", w);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for k in 0..12 {
                let hue = k * 30;
                if ui::swatch(ui, hue_colour(hue), "Colour", (self.profile_hue / 30) == k, None) {
                    self.profile_hue = hue;
                }
            }
        });
        if self.profile_name.trim() != me.display_name.trim() || self.profile_hue != me.avatar_hue {
            ui.add_space(6.0);
            if ui::button(ui, "Save profile", Some(Icon::Check), true) && !self.busy {
                let (name, hue) = (self.profile_name.clone(), self.profile_hue);
                self.act("profile", move |c| c.update_profile(&name, Some(hue)));
            }
        }

        ui::separator(ui);
        ui::section(ui, "Plan");
        let (plan, used) = (&me.plan, &me.usage);
        meter(ui, "Cloud scripts", used.projects, plan.max_projects, &format!("{} OF {}", used.projects, plan.max_projects));
        meter(ui, "Storage", used.bytes, plan.max_bytes, &format!("{} OF {}", megabytes(used.bytes), megabytes(plan.max_bytes)));
        meter(ui, "Teams you own", used.owned_teams, plan.max_owned_teams, &format!("{} OF {}", used.owned_teams, plan.max_owned_teams));
        ui::caption(ui, "NativeCloud is free during the beta. Paid plans with more room come later.");

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
        ui::section(ui, "Computers");
        let mut devices = self.devices.clone();
        devices.sort_by_key(|d| d.id != self.device().id);
        for d in devices {
            ui.horizontal(|ui| {
                let this = d.id == self.device().id;
                ui.label(egui::RichText::new(&d.name).font(theme::font_med(theme::T_SM)).color(p.text));
                if this {
                    ui::hud(ui, "This computer", Some(p.signal_hi));
                } else {
                    let seen = parse_time(&d.last_seen).map(|t| unix_now() - t).unwrap_or(0);
                    ui::hud(ui, &if seen < 60 { "Seen just now".to_string() } else { format!("Seen {}", time::ago(seen)) }, None);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui::quiet_button(ui, "Forget", None) {
                            self.ask(
                                &format!("Forget {}?", d.name),
                                "It disappears from this list. To stop it syncing, sign out everywhere too.",
                                "Forget",
                                false,
                                format!("ncf-cloud-forget:{}", d.id),
                            );
                        }
                    });
                }
            });
        }

        ui::separator(ui);
        ui::section(ui, "Email and password");
        match self.email_pending.clone() {
            None => {
                ui.horizontal(|ui| {
                    field(ui, &mut self.new_email, "New email", (w - 110.0).max(160.0));
                    if ui::button(ui, "Change", None, false) && !self.busy {
                        let e = self.new_email.trim().to_lowercase();
                        if email_ok(&e) {
                            self.busy = true;
                            self.engine.run(move |c| Msg::Sent(Code::EmailChange, c.change_email(&e)));
                        } else {
                            self.say("Check that", &CloudError::Invalid("email".into()).message(), Tone::Warn);
                        }
                    }
                });
            }
            Some(pending) => {
                ui::caption(ui, &format!("Enter the code we sent to {pending}."));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.email_code).hint_text("000000").char_limit(7).desired_width(120.0).margin(egui::Margin::symmetric(12.0, 8.0)));
                    if ui::button(ui, "Confirm", Some(Icon::Check), true) && !self.busy {
                        let code = self.email_code.clone();
                        self.busy = true;
                        self.engine.run(move |c| Msg::Verified(Code::EmailChange, c.verify(Code::EmailChange, &pending, &code).map(|_| ())));
                    }
                    if ui::quiet_button(ui, "Cancel", None) {
                        self.email_pending = None;
                    }
                });
            }
        }
        ui.add_space(6.0);
        let mut shown = self.show_password;
        secret(ui, &mut self.new_password, "New password", w, &mut shown);
        self.show_password = shown;
        if !self.new_password.is_empty() {
            ui.add_space(4.0);
            rules(ui, &self.new_password);
            ui.add_space(4.0);
            let ok = password_ok(&self.new_password);
            if ui::button(ui, "Change password", None, ok) && ok && !self.busy {
                let pw = self.new_password.clone();
                self.act("password", move |c| c.set_password(&pw));
            }
        }

        ui::separator(ui);
        ui::section(ui, "Sign out");
        ui.horizontal(|ui| {
            if ui::button(ui, "Sign out", Some(Icon::SignOut), false) && !self.busy {
                self.busy = true;
                self.engine.run(|c| Msg::SignedOut(c.sign_out(false)));
            }
            if ui::quiet_button(ui, "Sign out everywhere", None) {
                self.ask(
                    "Sign out on every computer?",
                    "Each one stops syncing until someone signs in again. Their local copies stay.",
                    "Sign out everywhere",
                    false,
                    "ncf-cloud-signout-all".into(),
                );
            }
        });

        ui::separator(ui);
        ui::section(ui, "Your data");
        ui.horizontal(|ui| {
            if ui::button(ui, "Export my data", Some(Icon::Download), false) && !self.busy {
                self.busy = true;
                let dir = export_dir();
                self.engine.run(move |c| {
                    let run = || -> Result<(PathBuf, usize), String> {
                        let spaces = c.workspaces().map_err(|e| e.message())?;
                        let projects = c.export().map_err(|e| e.message())?;
                        let n = write_export(&dir, &spaces, &projects)?;
                        Ok((dir.clone(), n))
                    };
                    Msg::Exported(run())
                });
            }
            if ui::quiet_button(ui, "Delete account…", Some(Icon::Trash)) {
                self.ask(
                    "Delete your NativeCloud account?",
                    "Your cloud scripts, snapshots and settings are removed now. Teams you own pass to their next member. Scripts on this computer stay.",
                    "Delete account",
                    true,
                    "ncf-cloud-delete".into(),
                );
            }
        });
        if let Some(mut pw) = self.reauth.take() {
            ui.add_space(6.0);
            ui::caption(ui, "Enter your password to confirm.");
            let mut shown = false;
            secret(ui, &mut pw, "Password", w, &mut shown);
            let mut keep = true;
            ui.horizontal(|ui| {
                if ui::button(ui, "Delete account", Some(Icon::Trash), true) && !self.busy {
                    self.busy = true;
                    let typed = pw.clone();
                    self.engine.run(move |c| Msg::Deleted(c.reauthenticate(&typed).and_then(|_| c.delete_account())));
                }
                if ui::quiet_button(ui, "Cancel", None) {
                    keep = false;
                    self.error = None;
                }
            });
            if keep {
                self.reauth = Some(pw);
            }
        }
        message(ui, &self.error, &None);
    }

    // teams here---

    pub fn teams_ui(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        if !self.signed_in() {
            ui::section(ui, "Teams");
            ui::caption(ui, "Sign in to NativeCloud to write with a team. Everyone sees the same scripts, one person writes at a time and the rest watch it change.");
            ui.add_space(8.0);
            if ui::button(ui, "Sign in", Some(Icon::ArrowRight), true) {
                self.urls.push("ncf:settings:Account".into());
            }
            return;
        }
        let w = ui.available_width().min(460.0);
        let teams: Vec<Team> = self.teams.iter().filter(|t| !t.workspace.is_personal()).cloned().collect();
        ui::section(ui, "Your teams");
        if teams.is_empty() {
            ui::caption(ui, "No teams yet. Create one and invite people by email.");
        }
        for t in &teams {
            let selected = self.team.as_deref() == Some(t.workspace.id.as_str());
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 36.0), Sense::click());
            let hot = crate::anim::ease(ui.ctx(), resp.id, resp.hovered() || selected, crate::anim::HOVER);
            theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, selected);
            icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 16.0, rect.center().y), Vec2::splat(14.0)), Icon::Users, p.text_2);
            ui.painter().text(Pos2::new(rect.left() + 34.0, rect.center().y), egui::Align2::LEFT_CENTER, &t.workspace.name, theme::font_med(theme::T_SM), p.text);
            let n = t.members.len();
            let tag = format!("{} · {n} {}", t.my_role.label(), if n == 1 { "person" } else { "people" }).to_uppercase();
            let tw = theme::tracked_width(ui.ctx(), &tag, theme::font_hud(theme::T_MICRO), 0.8);
            theme::tracked_text(ui.painter(), Pos2::new(rect.right() - 12.0 - tw, rect.center().y), &tag, theme::font_hud(theme::T_MICRO), p.text_3, 0.8);
            if resp.clicked() {
                self.team = if selected { None } else { Some(t.workspace.id.clone()) };
                self.rename = t.workspace.name.clone();
                self.role_menu = None;
            }
        }
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let r = field(ui, &mut self.new_team, "New team name", (w - 130.0).max(160.0));
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (ui::button(ui, "Create team", Some(Icon::Plus), false) || enter) && !self.new_team.trim().is_empty() && !self.busy {
                self.busy = true;
                let name = self.new_team.clone();
                self.engine.run(move |c| Msg::Created(c.create_workspace(&name)));
            }
        });

        let Some(t) = teams.into_iter().find(|t| Some(&t.workspace.id) == self.team.as_ref()) else {
            return;
        };
        let ws = t.workspace.id.clone();
        let me = self.engine.client().user_id();
        let manage = t.my_role.can_manage();
        ui::separator(ui);
        ui.label(egui::RichText::new(t.workspace.name.to_uppercase()).font(theme::font_display(theme::T_H + 6.0)).color(p.text));
        ui.add_space(4.0);
        if manage {
            ui.horizontal(|ui| {
                field(ui, &mut self.rename, "Team name", (w - 110.0).max(160.0));
                if self.rename.trim() != t.workspace.name && !self.rename.trim().is_empty() && ui::button(ui, "Rename", None, false) && !self.busy {
                    let (ws, name) = (ws.clone(), self.rename.clone());
                    self.act("renamed", move |c| c.rename_workspace(&ws, &name));
                }
            });
        }

        ui::section(ui, "People");
        for (m, person) in &t.members {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                avatar(ui, &person.shown(), person.hue, 26.0);
                let you = if m.user_id == me { " (you)" } else { "" };
                ui.label(egui::RichText::new(format!("{}{you}", person.shown())).font(theme::font_med(theme::T_SM)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let editable = manage && m.user_id != me && m.role != Role::Owner && !(t.my_role == Role::Admin && m.role == Role::Admin);
                    if !editable {
                        ui::hud(ui, m.role.label(), None);
                        return;
                    }
                    if ui::icon_button(ui, Icon::Close, "Remove from the team", 24.0) {
                        self.ask(
                            &format!("Remove {} from {}?", person.shown(), t.workspace.name),
                            "They lose access to the team's scripts straight away. Any script they're editing is freed.",
                            "Remove",
                            true,
                            format!("ncf-cloud-remove:{ws}:{}", m.user_id),
                        );
                    }
                    let open = self.role_menu.as_deref() == Some(m.user_id.as_str());
                    let resp = ui::dropdown(ui, role_icon(m.role), m.role.label(), open, 116.0);
                    if resp.clicked() {
                        self.role_menu = if open { None } else { Some(m.user_id.clone()) };
                    }
                    if open {
                        let mut picked = None;
                        let mut rects = vec![resp.rect];
                        let area = egui::Area::new(egui::Id::new(("ncf-cloud-role", &m.user_id)))
                            .order(egui::Order::Foreground)
                            .fixed_pos(resp.rect.left_bottom() + Vec2::new(0.0, 4.0))
                            .show(ui.ctx(), |ui| {
                                ui::popover_frame().show(ui, |ui| {
                                    ui.set_width(116.0);
                                    for r in Role::INVITABLE {
                                        if r == Role::Admin && t.my_role != Role::Owner {
                                            continue;
                                        }
                                        let icon = if r == m.role { Icon::Check } else { role_icon(r) };
                                        if ui::menu_item(ui, &mut rects, r.label(), icon, false) {
                                            picked = Some(r);
                                        }
                                    }
                                });
                            });
                        rects.push(area.response.rect);
                        if let Some(r) = picked {
                            self.role_menu = None;
                            if r != m.role {
                                let (ws, user) = (ws.clone(), m.user_id.clone());
                                self.act("role", move |c| c.set_role(&ws, &user, r));
                            }
                        } else if ui.input(|i| i.pointer.any_click()) && !ui.ctx().pointer_interact_pos().map(|pt| rects.iter().any(|r| r.contains(pt))).unwrap_or(false) {
                            self.role_menu = None;
                        }
                    }
                });
            });
            ui.add_space(2.0);
        }

        if manage {
            if !t.invites.is_empty() {
                ui.add_space(4.0);
                ui::section(ui, "Waiting to join");
                for inv in &t.invites {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&inv.email).font(theme::font(theme::T_SM)).color(p.text_2));
                        ui::hud(ui, inv.role.label(), None);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui::quiet_button(ui, "Revoke", None) && !self.busy {
                                let id = inv.id.clone();
                                self.act("revoked", move |c| c.revoke_invite(&id));
                            }
                        });
                    });
                }
            }
            ui.add_space(6.0);
            ui::section(ui, "Invite by email");
            field(ui, &mut self.invite_email, "name@example.com", w);
            ui.add_space(6.0);
            let roles: Vec<Role> = Role::INVITABLE.iter().copied().filter(|r| *r != Role::Admin || t.my_role == Role::Owner).collect();
            let labels: Vec<&str> = roles.iter().map(|r| r.label()).collect();
            if !roles.contains(&self.invite_role) {
                self.invite_role = Role::Editor;
            }
            let current = roles.iter().position(|r| *r == self.invite_role).unwrap_or(0);
            if let Some(k) = ui::tabs(ui, &labels, current) {
                self.invite_role = roles[k];
            }
            ui::caption(ui, self.invite_role.hint());
            ui.add_space(6.0);
            if ui::button(ui, "Send invite", Some(Icon::ArrowRight), true) && !self.busy {
                let email = self.invite_email.trim().to_lowercase();
                if !email_ok(&email) {
                    self.say("Check that", &CloudError::Invalid("email".into()).message(), Tone::Warn);
                } else {
                    self.busy = true;
                    let (ws, role, app) = (ws.clone(), self.invite_role, self.info.id.to_string());
                    let shown = email.clone();
                    self.engine.run(move |c| Msg::Invited(shown, c.invite_member(&ws, &email, role).map(|id| c.send_invite(&id, &app).unwrap_or(false))));
                }
            }
        }

        ui::separator(ui);
        ui::section(ui, "Leave or hand over");
        if t.my_role == Role::Owner {
            let others: Vec<(Member, Person)> = t.members.iter().filter(|(m, _)| m.user_id != me).cloned().collect();
            if others.is_empty() {
                ui::caption(ui, "Invite someone before you can hand the team over.");
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui::caption(ui, "Hand over to");
                    for (m, person) in others {
                        if ui::chip_button(ui, &person.shown()) {
                            self.ask(
                                &format!("Make {} the owner?", person.shown()),
                                "You become an admin. Only the owner can delete the team or hand it over again.",
                                "Hand over",
                                false,
                                format!("ncf-cloud-transfer:{ws}:{}", m.user_id),
                            );
                        }
                    }
                });
            }
            ui.add_space(6.0);
            if ui::quiet_button(ui, "Delete team…", Some(Icon::Trash)) {
                self.ask(
                    &format!("Delete {}?", t.workspace.name),
                    "Its scripts and snapshots are deleted for everyone. Copies on people's computers stay.",
                    "Delete team",
                    true,
                    format!("ncf-cloud-delete-team:{ws}"),
                );
            }
        } else if ui::quiet_button(ui, "Leave team…", Some(Icon::SignOut)) {
            self.ask(
                &format!("Leave {}?", t.workspace.name),
                "Its scripts leave your library. An admin can invite you back.",
                "Leave",
                true,
                format!("ncf-cloud-leave:{ws}"),
            );
        }
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
