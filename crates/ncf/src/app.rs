use std::path::PathBuf;
use std::sync::mpsc::Receiver;

use eframe::egui::{self, Align, Layout, Pos2, Rect, Sense, Stroke, Vec2};

use crate::alerts::{Deck, Tone};
use crate::icons::Icon;
use crate::settings::{self, Look};
use crate::splash::{self, Splash};
use crate::theme::{self, pal};
use crate::update::{self, Status};
use crate::{anim, chrome, platform, ui, AppInfo};

pub const SIZE: Vec2 = Vec2::new(1320.0, 900.0);
pub const MIN_SIZE: Vec2 = Vec2::new(880.0, 580.0);

// app side here---

pub trait NcfApp {
    fn ui(&mut self, ui: &mut egui::Ui, shell: &mut Shell);
    fn title_extras(&mut self, _ui: &mut egui::Ui, _shell: &mut Shell) {}
    fn settings_tabs(&self) -> Vec<(&'static str, Icon)> {
        Vec::new()
    }
    fn settings_ui(&mut self, _tab: &str, _ui: &mut egui::Ui, _shell: &mut Shell) {}
    fn read_setting(&mut self, _key: &str, _value: &str) {}
    fn write_settings(&self) -> String {
        String::new()
    }
    fn answer(&mut self, _action: &str, _text: Option<String>, _shell: &mut Shell) {}
    fn on_exit(&mut self, _shell: &mut Shell) {}
    fn about(&mut self, _ui: &mut egui::Ui, _shell: &mut Shell) {}
    fn debug_ui(&mut self, _ui: &mut egui::Ui, _shell: &mut Shell) {}
}

// shell here---

pub struct Shell {
    pub info: AppInfo,
    pub look: Look,
    pub deck: Deck<String>,
    pub update: Status,
    pub frame_no: u64,
    pub debug: crate::debug::Debug,
    splash_replay: Option<Status>,
    settings_open: bool,
    settings_tab: String,
    look_dirty: bool,
    save_due: bool,
    update_rx: Option<Receiver<Status>>,
    extra_settings: Vec<(String, String)>,
}

impl Shell {
    pub fn new(info: AppInfo) -> Shell {
        let text = std::fs::read_to_string(settings_path(&info)).unwrap_or_default();
        let look = Look::parse(&text);
        let extra_settings = settings::pairs(&text).into_iter().filter(|(k, _)| !Look::default().clone().read(k, "")).collect();
        Shell {
            info,
            debug: crate::debug::Debug::new(look.debug),
            splash_replay: None,
            look,
            deck: Deck::default(),
            update: Status::Off,
            frame_no: 0,
            settings_open: false,
            settings_tab: "Appearance".into(),
            look_dirty: true,
            save_due: false,
            update_rx: None,
            extra_settings,
        }
    }

    pub fn folder(&self) -> PathBuf {
        platform::app_folder(self.info.name)
    }

    pub fn say(&mut self, title: &str, body: &str) {
        self.deck.say(title, body, Tone::Info);
    }

    pub fn ok(&mut self, title: &str, body: &str) {
        self.deck.ok(title, body);
    }

    pub fn warn(&mut self, title: &str, body: &str) {
        self.deck.warn(title, body);
    }

    pub fn ask(&mut self, title: &str, body: &str, yes: &str, danger: bool, action: &str) {
        let tone = if danger { Tone::Danger } else { Tone::Info };
        self.deck.ask(title, body, yes, tone, action.to_string());
    }

    pub fn open_url(&self, url: &str) {
        platform::open(url);
    }

    pub fn open_settings(&mut self, tab: &str) {
        self.settings_open = true;
        self.settings_tab = tab.to_string();
    }

    pub fn settings_open(&self) -> bool {
        self.settings_open
    }

    pub fn set_light(&mut self, light: bool) {
        self.look.light_mode = light;
        self.look_dirty = true;
        self.save_due = true;
    }

    pub fn save_settings(&mut self) {
        self.save_due = true;
    }

    pub fn update_plan(&self) -> Option<crate::updater::Plan> {
        let Status::Newer(r) = &self.update else {
            return None;
        };
        crate::updater::Plan::for_release(&self.info, &r.version, self.folder())
    }

    pub fn update_now(&mut self, ctx: &egui::Context) -> bool {
        let Some(plan) = self.update_plan() else {
            return false;
        };
        match crate::updater::start(&plan) {
            Ok(()) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                true
            }
            Err(e) => {
                self.warn("Couldn't start the update", &e);
                false
            }
        }
    }

    pub fn check_for_updates(&mut self) {
        if !self.info.repo.is_empty() {
            self.update_rx = Some(update::spawn(self.info.repo, self.info.version));
            self.update = Status::Checking;
        }
    }
}

pub fn settings_path(info: &AppInfo) -> PathBuf {
    platform::app_folder(info.name).join("settings.conf")
}

// runner here---

struct Runner<A: NcfApp> {
    app: A,
    shell: Shell,
    splash: Option<Splash>,
    settled: bool,
    replaying: bool,
}

pub fn viewport(info: &AppInfo, size: Vec2, min: Vec2) -> egui::ViewportBuilder {
    let builder = egui::ViewportBuilder::default()
        .with_title(info.name)
        .with_icon(std::sync::Arc::new(info.mark.icon_data(128)))
        .with_app_id(info.id)
        .with_drag_and_drop(true)
        .with_inner_size(size)
        .with_min_inner_size(min);
    chrome::viewport(builder, crate::OS)
}

pub fn first_size(splash: bool, os: crate::Os) -> (Vec2, Vec2) {
    if splash && splash::own_window(os) {
        (splash::CARD, splash::CARD)
    } else {
        (SIZE, MIN_SIZE)
    }
}

pub fn run<A: NcfApp + 'static>(info: AppInfo, make: impl FnOnce(&egui::Context, &mut Shell) -> A + 'static) -> eframe::Result<()> {
    run_with(Shell::new(info), make)
}

pub fn renderers() -> Vec<eframe::Renderer> {
    #[cfg(target_os = "macos")]
    return vec![eframe::Renderer::Wgpu, eframe::Renderer::Glow];
    #[cfg(not(target_os = "macos"))]
    return vec![eframe::Renderer::Glow];
}

pub fn run_with<A: NcfApp + 'static>(shell: Shell, make: impl FnOnce(&egui::Context, &mut Shell) -> A + 'static) -> eframe::Result<()> {
    let info = shell.info;
    let wants_splash = shell.look.splash && shell.look.animations;
    let (size, min) = first_size(wants_splash, crate::OS);
    let waiting = std::rc::Rc::new(std::cell::RefCell::new(Some((shell, make))));
    let mut outcome = Ok(());
    for renderer in renderers() {
        let options = eframe::NativeOptions {
            viewport: viewport(&info, size, min),
            renderer,
            centered: chrome::native_frame(crate::OS),
            ..Default::default()
        };
        let parts = waiting.clone();
        outcome = eframe::run_native(
            info.name,
            options,
            Box::new(move |cc| {
                let (mut shell, make) = parts.borrow_mut().take().ok_or("the app had already started")?;
                let app = start(&cc.egui_ctx, &mut shell, make);
                let splash = if wants_splash { Some(Splash::for_os(info, crate::OS)) } else { None };
                Ok(Box::new(Runner { app, shell, splash, settled: false, replaying: false }))
            }),
        );
        if outcome.is_ok() || waiting.borrow().is_none() {
            break;
        }
        if let Err(e) = &outcome {
            eprintln!("{}: the {renderer} renderer could not start ({e}); trying the next one", info.name);
        }
    }
    outcome
}

pub fn start<A: NcfApp>(ctx: &egui::Context, shell: &mut Shell, make: impl FnOnce(&egui::Context, &mut Shell) -> A) -> A {
    theme::install_fonts(ctx);
    theme::set_palette(!shell.look.light_mode);
    theme::apply(ctx);
    anim::set_enabled(shell.look.animations);
    ctx.set_zoom_factor(shell.look.font_scale);
    let mut app = make(ctx, shell);
    for (k, v) in std::mem::take(&mut shell.extra_settings) {
        app.read_setting(&k, &v);
    }
    if shell.look.check_updates {
        shell.check_for_updates();
    }
    app
}

impl<A: NcfApp> eframe::App for Runner<A> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        frame(ctx, &mut self.app, &mut self.shell);
        if let Some(status) = self.shell.splash_replay.take() {
            let mut s = Splash::inline(self.shell.info);
            s.set_status(status);
            self.splash = Some(s);
            self.replaying = true;
        }
        let own_window = self.splash.as_ref().map(|s| !s.is_inline()).unwrap_or(false);
        if !own_window && !self.settled {
            self.settled = splash::settle(ctx, SIZE);
        }
        if let Some(s) = self.splash.as_mut() {
            if !self.replaying {
                s.set_status(self.shell.update.clone());
            }
            if s.show(ctx) {
                if own_window && self.shell.frame_no < 3 {
                    splash::centre(ctx, splash::CARD);
                }
            } else {
                self.splash = None;
                self.replaying = false;
                if own_window {
                    let size = chrome::fit(SIZE, ctx.input(|i| i.viewport().monitor_size), crate::OS);
                    ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(MIN_SIZE));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                    splash::centre(ctx, size);
                }
            }
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        chrome::clear_color(crate::OS, pal().void)
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.app.on_exit(&mut self.shell);
        let text = format!("{}{}", self.shell.look.write(), self.app.write_settings());
        let _ = platform::write_atomic(&settings_path(&self.shell.info), text.as_bytes());
    }
}

pub fn frame<A: NcfApp>(ctx: &egui::Context, app: &mut A, shell: &mut Shell) {
    shell.frame_no = shell.frame_no.wrapping_add(1);
    if shell.look_dirty {
        theme::set_palette(!shell.look.light_mode);
        theme::apply(ctx);
        anim::set_enabled(shell.look.animations);
        ctx.set_zoom_factor(shell.look.font_scale);
        shell.look_dirty = false;
    }
    if let Some(rx) = &shell.update_rx {
        if let Ok(status) = rx.try_recv() {
            shell.update_rx = None;
            if let Status::Newer(r) = &status {
                let (title, body) = (format!("{} {} is out", shell.info.name, r.version), "Download it from Settings → About.".to_string());
                shell.say(&title, &body);
            }
            shell.update = status;
        }
    }

    chrome::backdrop(ctx, pal().void);
    chrome::resize_handles(ctx);

    egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
        let info = shell.info;
        let mut open_settings = false;
        chrome::title_bar(ui, ctx, &info, |ui| {
            if ui::icon_button(ui, Icon::Settings, &format!("Settings · {}+,", crate::OS.command_key()), 28.0) {
                open_settings = true;
            }
            app.title_extras(ui, shell);
            shell.debug.tag(ui);
        });
        if open_settings {
            shell.settings_open = !shell.settings_open;
        }
        let rect = ui.available_rect_before_wrap().shrink2(Vec2::new(14.0, 0.0));
        let mut content = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_max(rect.min, rect.max - Vec2::new(0.0, 14.0))));
        app.ui(&mut content, shell);
    });

    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::Comma)) {
        shell.settings_open = !shell.settings_open;
    }
    if shell.settings_open {
        settings_window(ctx, app, shell);
    }
    let mut debug = std::mem::take(&mut shell.debug);
    let requests = debug.panel(ctx, SIZE, |ui, _| app.debug_ui(ui, shell));
    shell.debug = debug;
    for request in requests {
        apply_debug(ctx, shell, &request);
    }
    if let Some(answer) = shell.deck.show(ctx) {
        if answer.action == "ncf-debug-update" {
            shell.update_now(ctx);
        } else {
            app.answer(&answer.action, answer.text, shell);
        }
    }

    if shell.save_due {
        shell.save_due = false;
        let text = format!("{}{}", shell.look.write(), app.write_settings());
        if let Err(e) = platform::write_atomic(&settings_path(&shell.info), text.as_bytes()) {
            shell.warn("Could not save settings", &e.to_string());
        }
    }
}

pub fn apply_debug(ctx: &egui::Context, shell: &mut Shell, request: &crate::debug::Request) {
    use crate::debug::Request;
    if crate::debug::apply(ctx, &mut shell.debug, request) {
        if *request == Request::Off {
            shell.look.debug = false;
            shell.save_due = true;
            shell.say("Debug mode off", "Turn it on again the same way.");
        }
        return;
    }
    match request {
        Request::FakeUpdate => {
            let version = crate::debug::next_version(shell.info.version);
            shell.update = Status::Newer(update::Release { version: version.clone(), url: update::releases_page(shell.info.repo) });
            shell.say(&format!("{} {} is out", shell.info.name, version), "A pretend release, from debug mode.");
        }
        Request::UpdaterDemo(demo) => match crate::updater::start_demo(*demo) {
            Ok(()) => shell.say("Updater demo", "It opens in its own window. Nothing is installed."),
            Err(e) => shell.warn("Couldn't open the updater demo", &e),
        },
        Request::UpdateFrom { dir, version } => {
            std::env::set_var("NCF_UPDATE_SOURCE", format!("file://{}", dir.trim_end_matches(['/', '\\'])));
            shell.update = Status::Newer(update::Release { version: version.clone(), url: String::new() });
            let body = format!("{} closes and installs {version} from {dir}.", shell.info.name);
            shell.ask(&format!("Update to {version} from this folder?"), &body, "Update now", false, "ncf-debug-update");
        }
        Request::Splash(status) => shell.splash_replay = Some(status.clone()),
        Request::Notice(tone) => {
            let (title, body) = crate::debug::notice(*tone);
            shell.deck.say(title, body, *tone);
        }
        Request::Confirm => shell.ask("A confirm card", "This is how a question looks. Nothing happens either way.", "Fine", false, "ncf-debug-confirm"),
        Request::Crash => {
            let text = format!("{} hit a test error from debug mode. Nothing is wrong.", shell.info.name);
            let log = crate::debug::crash_report(&shell.info, &text);
            shell.say("Crash report", &format!("Written to {}", log.display()));
        }
        Request::TextScale(step) => {
            shell.look.font_scale = (shell.look.font_scale + step).clamp(0.8, 1.4);
            shell.look_dirty = true;
            shell.save_due = true;
        }
        Request::Light(light) => shell.set_light(*light),
        Request::Animations(on) => {
            shell.look.animations = *on;
            shell.look_dirty = true;
            shell.save_due = true;
        }
        _ => {}
    }
}

// settings window here---

fn settings_window<A: NcfApp>(ctx: &egui::Context, app: &mut A, shell: &mut Shell) {
    let p = pal();
    let t = anim::ease(ctx, "ncf-settings-rise", true, crate::tokens::MOTION_SLOW);
    if ui::scrim(ctx, "ncf-settings-scrim", t) || ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !shell.deck.asking() {
        shell.settings_open = false;
        return;
    }
    let screen = ctx.screen_rect();
    let size = Vec2::new((screen.width() - 80.0).clamp(560.0, 780.0), (screen.height() - 80.0).clamp(420.0, 620.0));
    let rect = Rect::from_center_size(screen.center(), size * (0.97 + 0.03 * t));

    let mut tabs: Vec<(&str, Icon)> = vec![("Appearance", Icon::Palette)];
    let extra = app.settings_tabs();
    tabs.extend(extra.iter().copied());
    tabs.push(("About", Icon::Info));
    let current = tabs.iter().position(|(n, _)| *n == shell.settings_tab).unwrap_or(0);

    egui::Area::new(egui::Id::new("ncf-settings"))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .show(ctx, |ui| {
            ui.set_opacity(0.2 + 0.8 * t);
            let (win, _) = ui.allocate_exact_size(rect.size(), Sense::click());
            ui.painter().rect_filled(win, egui::Rounding::same(theme::RADIUS), p.deep);
            ui.painter().rect_stroke(win.shrink(0.5), egui::Rounding::same(theme::RADIUS), Stroke::new(1.0_f32, p.line_strong));

            let head = Rect::from_min_size(win.min, Vec2::new(win.width(), 56.0));
            let mut h = ui.new_child(egui::UiBuilder::new().max_rect(head.shrink2(Vec2::new(20.0, 0.0))).layout(Layout::left_to_right(Align::Center)));
            h.spacing_mut().item_spacing.x = 10.0;
            h.label(egui::RichText::new("Settings").font(theme::font_bold(theme::T_H + 1.0)).color(p.text));
            ui::hud(&mut h, &format!("{} / {}", shell.info.name, if p.dark { "dark" } else { "light" }), None);
            h.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui::icon_button(ui, Icon::Close, "Close · Esc", 28.0) {
                    shell.settings_open = false;
                }
            });

            let nav = Rect::from_min_max(Pos2::new(win.left() + 14.0, head.bottom()), Pos2::new(win.left() + 198.0, win.bottom() - 14.0));
            let mut n = ui.new_child(egui::UiBuilder::new().max_rect(nav).layout(Layout::top_down(Align::Min)));
            if let Some(k) = ui::vertical_tabs(&mut n, &tabs, current) {
                shell.settings_tab = tabs[k].0.to_string();
            }
            theme::tracked_text(
                ui.painter(),
                Pos2::new(nav.left() + 10.0, nav.bottom() - 16.0),
                &format!("{} V{}", shell.info.name.to_uppercase(), shell.info.version),
                theme::font_hud(theme::T_MICRO),
                p.text_3,
                0.8,
            );

            let pane = Rect::from_min_max(Pos2::new(nav.right() + 14.0, head.bottom()), Pos2::new(win.right() - 10.0, win.bottom() - 10.0));
            ui.painter().rect_filled(pane, egui::Rounding::same(theme::RADIUS), p.panel);
            ui.painter().rect_stroke(pane.shrink(0.5), egui::Rounding::same(theme::RADIUS), Stroke::new(1.0_f32, p.line));
            let mut pane_ui = ui.new_child(egui::UiBuilder::new().max_rect(pane.shrink2(Vec2::new(18.0, 14.0))).layout(Layout::top_down(Align::Min)));
            let name = tabs[current].0;
            pane_ui.label(egui::RichText::new(name.to_uppercase()).font(theme::font_display(theme::T_H + 1.0)).color(p.text));
            pane_ui.add_space(4.0);
            egui::ScrollArea::vertical().id_salt(("ncf-settings-pane", current)).auto_shrink([false, false]).show(&mut pane_ui, |ui| {
                ui.set_max_width(ui.available_width() - 14.0);
                match name {
                    "Appearance" => appearance(ui, shell),
                    "About" => about(ui, app, shell),
                    other => app.settings_ui(other, ui, shell),
                }
                ui.add_space(8.0);
            });
        });
}

fn appearance(ui: &mut egui::Ui, shell: &mut Shell) {
    let before = shell.look.clone();
    ui::section(ui, "Mode");
    let dark = !shell.look.light_mode;
    if let Some(k) = ui::tabs(ui, &["Dark", "Light"], if dark { 0 } else { 1 }) {
        shell.look.light_mode = k == 1;
    }
    ui.add_space(4.0);
    ui::caption(ui, "Dark is NativeCompute's own: void black and one signal blue. Light is the same language on paper white.");
    ui::separator(ui);
    ui::section(ui, "Motion");
    let mut v = shell.look.animations;
    if ui::toggle_row(ui, "Animations", "Every transition in the app, at once.", &mut v) {
        shell.look.animations = v;
    }
    let mut s = shell.look.splash;
    if ui::toggle_row(ui, "Splash screen", "The card the app opens on.", &mut s) {
        shell.look.splash = s;
    }
    ui.add_space(6.0);
    ui.label(egui::RichText::new(format!("Interface size · {:.0}%", shell.look.font_scale * 100.0)).font(theme::font_med(theme::T_SM)).color(pal().text));
    let mut fs = shell.look.font_scale;
    if ui::slider(ui, &mut fs, 0.8..=1.4) {
        shell.look.font_scale = (fs * 20.0).round() / 20.0;
    }
    if shell.look != before {
        shell.look_dirty = true;
        shell.save_due = true;
    }
}

fn about<A: NcfApp>(ui: &mut egui::Ui, app: &mut A, shell: &mut Shell) {
    let p = pal();
    ui::section(ui, shell.info.name);
    let version = ui.label(egui::RichText::new(format!("Version {}", shell.info.version)).font(theme::font(theme::T_CAP)).color(p.text_3));
    if shell.debug.knock(ui, version.rect) {
        shell.look.debug = shell.debug.on;
        shell.save_due = true;
        if shell.debug.on {
            shell.say("Debug mode on", "The DEBUG tag in the title bar opens its panel.");
        } else {
            shell.say("Debug mode off", "Turn it on again the same way.");
        }
    }
    if !shell.info.tagline.is_empty() {
        ui::body(ui, shell.info.tagline);
    }
    app.about(ui, shell);
    if !shell.info.repo.is_empty() {
        ui::separator(ui);
        ui::section(ui, "Updates");
        let mut v = shell.look.check_updates;
        if ui::toggle_row(
            ui,
            "Check for updates on launch",
            "Asks GitHub once whether a newer version is out. Nothing is downloaded or installed by itself.",
            &mut v,
        ) {
            shell.look.check_updates = v;
            shell.save_due = true;
        }
        let line = match &shell.update {
            Status::Off | Status::Newer(_) => None,
            Status::Checking => Some("Checking…".to_string()),
            Status::Latest => Some(format!("{} {} is the latest version.", shell.info.name, shell.info.version)),
            Status::Failed => Some("Could not reach GitHub to check.".to_string()),
        };
        if let Some(line) = line {
            ui::caption(ui, &line);
        }
        if let Status::Newer(r) = shell.update.clone() {
            if shell.update_plan().is_some() {
                if ui::link(ui, &format!("{} {} is out — update now", shell.info.name, r.version)) {
                    shell.update_now(ui.ctx());
                }
            } else if ui::link(ui, &format!("{} {} is out — download it", shell.info.name, r.version)) {
                shell.open_url(&r.url);
            }
        }
        if ui::link(ui, "All releases") {
            shell.open_url(&update::releases_page(shell.info.repo));
        }
    }
    ui::separator(ui);
    ui::section(ui, "Made by");
    ui.add_space(2.0);
    ui::logo(ui, (theme::T_H * 1.25).round(), p.text).on_hover_text("NativeCompute");
    ui.add_space(4.0);
    ui::body(ui, &format!("{} is made by NativeCompute.", shell.info.name));
    ui.add_space(4.0);
    if ui::link(ui, "nativecompute.net") {
        shell.open_url(crate::tokens::BRAND_URL);
    }
}
