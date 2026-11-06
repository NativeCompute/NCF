use eframe::egui::{self, Id, Rect, Sense, Stroke, Vec2};

use crate::alerts::Tone;
use crate::icons::Icon;
use crate::theme::{self, pal};
use crate::update::Status;
use crate::{anim, ui};

pub const KNOCKS: usize = 5;
pub const KNOCK_WINDOW: f64 = 3.0;
pub const LAPTOP: Vec2 = Vec2::new(1280.0, 800.0);
pub const SMALL: Vec2 = Vec2::new(1024.0, 640.0);
pub const OUTLINES: bool = cfg!(debug_assertions);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demo {
    Works,
    Fails,
    Slow,
}

impl Demo {
    pub fn arg(&self) -> &'static str {
        match self {
            Demo::Works => "ok",
            Demo::Fails => "fail",
            Demo::Slow => "slow",
        }
    }

    pub fn parse(s: &str) -> Option<Demo> {
        match s {
            "ok" => Some(Demo::Works),
            "fail" => Some(Demo::Fails),
            "slow" => Some(Demo::Slow),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    FakeUpdate,
    UpdaterDemo(Demo),
    UpdateFrom { dir: String, version: String },
    Splash(Status),
    Notice(Tone),
    Confirm,
    Crash,
    WindowSize(Vec2),
    TextScale(f32),
    Light(bool),
    Animations(bool),
    Inspector(bool),
    Off,
}

#[derive(Clone, Debug, Default)]
pub struct Debug {
    pub on: bool,
    pub open: bool,
    pub inspector: bool,
    knocks: Vec<f64>,
    folder: String,
    version: String,
}

impl Debug {
    pub fn new(on: bool) -> Debug {
        Debug { on, ..Debug::default() }
    }

    pub fn knock(&mut self, ui: &egui::Ui, rect: Rect) -> bool {
        let resp = ui.interact(rect, ui.id().with("ncf-knock"), Sense::click());
        if !resp.clicked() {
            return false;
        }
        let (m, t) = ui.input(|i| (i.modifiers, i.time));
        self.count(m.command && m.shift, t)
    }

    pub fn count(&mut self, held: bool, t: f64) -> bool {
        if !held {
            self.knocks.clear();
            return false;
        }
        self.knocks.retain(|k| t - *k <= KNOCK_WINDOW);
        self.knocks.push(t);
        if self.knocks.len() < KNOCKS {
            return false;
        }
        self.knocks.clear();
        self.on = !self.on;
        self.open = self.on;
        if !self.on {
            self.inspector = false;
        }
        true
    }

    pub fn tag(&mut self, ui: &mut egui::Ui) {
        if self.on && ui::chip_button(ui, "DEBUG") {
            self.open = !self.open;
        }
    }

    pub fn panel(&mut self, ctx: &egui::Context, default_size: Vec2, app: impl FnOnce(&mut egui::Ui, &mut Vec<Request>)) -> Vec<Request> {
        let mut out = Vec::new();
        if !self.on || !self.open {
            return out;
        }
        let p = pal();
        let screen = ctx.screen_rect();
        let mut close = false;
        egui::Area::new(Id::new("ncf-debug-panel"))
            .order(egui::Order::Foreground)
            .default_pos(egui::pos2(screen.right() - 290.0, screen.top() + 52.0))
            .default_size(Vec2::new(274.0, (screen.height() - 80.0).max(240.0)))
            .movable(true)
            .constrain(true)
            .show(ctx, |ui| {
                egui::Frame::none()
                    .fill(p.panel)
                    .stroke(Stroke::new(1.0_f32, p.line_strong))
                    .rounding(egui::Rounding::same(theme::RADIUS))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(250.0);
                        ui.horizontal(|ui| {
                            let ms = ui.input(|i| i.stable_dt) * 1000.0;
                            ui::hud(ui, &format!("DEBUG · {ms:.1} MS"), Some(p.signal_hi));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui::icon_button(ui, Icon::Close, "Hide", 22.0) {
                                    close = true;
                                }
                            });
                        });
                        egui::ScrollArea::vertical().max_height((screen.height() - 140.0).max(200.0)).show(ui, |ui| {
                            self.sections(ui, &mut out, default_size, app);
                        });
                    });
            });
        if close {
            self.open = false;
        }
        out
    }

    fn sections(&mut self, ui: &mut egui::Ui, out: &mut Vec<Request>, default_size: Vec2, app: impl FnOnce(&mut egui::Ui, &mut Vec<Request>)) {
        let row = |ui: &mut egui::Ui, items: &[(&str, Request)], out: &mut Vec<Request>| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
                for (label, req) in items {
                    if ui::quiet_button(ui, label, None) {
                        out.push(req.clone());
                    }
                }
            });
        };

        ui::section(ui, "Updater");
        row(ui, &[("Fake update", Request::FakeUpdate)], out);
        row(
            ui,
            &[
                ("Demo · works", Request::UpdaterDemo(Demo::Works)),
                ("Demo · fails", Request::UpdaterDemo(Demo::Fails)),
                ("Demo · slow", Request::UpdaterDemo(Demo::Slow)),
            ],
            out,
        );
        ui::field(ui, &mut self.folder, "Folder of release files", ui.available_width() - 24.0);
        ui.horizontal(|ui| {
            ui::field(ui, &mut self.version, "Version", 56.0);
            if ui::quiet_button(ui, "Update from folder", None) && !self.folder.trim().is_empty() && !self.version.trim().is_empty() {
                out.push(Request::UpdateFrom { dir: self.folder.trim().to_string(), version: self.version.trim().to_string() });
            }
        });

        ui::section(ui, "Launch & errors");
        let newer = Status::Newer(crate::update::Release { version: "9.9.9".into(), url: String::new() });
        row(
            ui,
            &[
                ("Splash · checking", Request::Splash(Status::Checking)),
                ("· up to date", Request::Splash(Status::Latest)),
                ("· new version", Request::Splash(newer)),
                ("· failed", Request::Splash(Status::Failed)),
            ],
            out,
        );
        row(
            ui,
            &[
                ("Info", Request::Notice(Tone::Info)),
                ("Ok", Request::Notice(Tone::Ok)),
                ("Warn", Request::Notice(Tone::Warn)),
                ("Danger", Request::Notice(Tone::Danger)),
                ("Confirm", Request::Confirm),
                ("Crash report", Request::Crash),
            ],
            out,
        );

        app(ui, out);

        ui::section(ui, "Display");
        row(
            ui,
            &[
                ("1280×800", Request::WindowSize(LAPTOP)),
                ("1024×640", Request::WindowSize(SMALL)),
                ("Default size", Request::WindowSize(default_size)),
            ],
            out,
        );
        let dark = pal().dark;
        let animations = anim::enabled();
        let inspector = self.inspector;
        let mut display = vec![
            ("Text −", Request::TextScale(-0.1)),
            ("Text +", Request::TextScale(0.1)),
            (if dark { "Light" } else { "Dark" }, Request::Light(dark)),
            (if animations { "Animations off" } else { "Animations on" }, Request::Animations(!animations)),
        ];
        if OUTLINES {
            display.push((if inspector { "Outlines off" } else { "Outlines on" }, Request::Inspector(!inspector)));
        }
        row(ui, &display, out);
        ui.add_space(6.0);
        row(ui, &[("Turn debug mode off", Request::Off)], out);
    }
}

pub fn apply(ctx: &egui::Context, debug: &mut Debug, request: &Request) -> bool {
    match request {
        Request::WindowSize(want) => {
            let monitor = ctx.input(|i| i.viewport().monitor_size);
            let size = crate::chrome::fit(*want, monitor, crate::OS);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            if let Some(monitor) = monitor {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(crate::splash::centred(monitor, size, crate::OS)));
            }
            true
        }
        Request::Inspector(on) => {
            debug.inspector = *on && OUTLINES;
            #[cfg(debug_assertions)]
            ctx.style_mut(|s| {
                s.debug.show_expand_width = *on;
                s.debug.show_expand_height = *on;
                s.debug.show_interactive_widgets = *on;
                s.debug.show_unaligned = *on;
            });
            true
        }
        Request::Off => {
            debug.on = false;
            debug.open = false;
            if debug.inspector {
                apply(ctx, debug, &Request::Inspector(false));
            }
            true
        }
        _ => false,
    }
}

pub fn notice(tone: Tone) -> (&'static str, &'static str) {
    match tone {
        Tone::Info => ("Info notice", "This is how a plain notice looks."),
        Tone::Ok => ("Ok notice", "This is how a success looks."),
        Tone::Warn => ("Warning notice", "This is how a warning looks."),
        Tone::Danger => ("Danger notice", "This is how an error looks."),
    }
}

pub fn next_version(version: &str) -> String {
    let mut parts: Vec<u64> = version.trim_start_matches(['v', 'V']).split('.').map(|p| p.parse().unwrap_or(0)).collect();
    parts.resize(3, 0);
    parts[2] += 1;
    format!("{}.{}.{}", parts[0], parts[1], parts[2])
}

#[cfg(not(target_arch = "wasm32"))]
pub fn crash_report(info: &crate::AppInfo, text: &str) -> std::path::PathBuf {
    let folder = crate::platform::app_folder(info.name);
    let log = folder.join("error.log");
    let _ = std::fs::create_dir_all(&folder);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&log) {
        use std::io::Write;
        let _ = writeln!(f, "{text}");
    }
    let (title, text) = (info.name.to_string(), text.to_string());
    std::thread::spawn(move || crate::platform::alert(&title, &text));
    log
}

