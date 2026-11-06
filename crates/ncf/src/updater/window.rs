use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use eframe::egui::{self, Align, Layout, Pos2, Rect, Sense, Stroke, Vec2};
use web_time::Instant;

use super::work::{self, Event, State, Step, STEPS};
use super::Plan;
use crate::settings::Look;
use crate::theme::{self, pal};
use crate::{anim, chrome, ui, AppInfo};

pub const SIZE: Vec2 = Vec2::new(540.0, 308.0);

pub fn size(os: crate::Os) -> Vec2 {
    SIZE + Vec2::new(0.0, if chrome::native_frame(os) { 14.0 } else { 0.0 })
}
const HOLD_DONE: f32 = 1.5;

pub struct Updater {
    info: AppInfo,
    plan: Plan,
    rx: Option<Receiver<Event>>,
    pub state: State,
    cancel: Arc<AtomicBool>,
    done_at: Option<Instant>,
    closing: bool,
    shot: Option<PathBuf>,
    shot_asked: bool,
    frames: u32,
    height: f32,
    reopened: Arc<AtomicBool>,
}

impl Updater {
    pub fn new(info: AppInfo, plan: Plan, rx: Option<Receiver<Event>>, cancel: Arc<AtomicBool>) -> Updater {
        Updater {
            info,
            plan,
            rx,
            state: State::default(),
            cancel,
            done_at: None,
            closing: false,
            shot: std::env::var_os("NCF_UPDATE_SHOT").map(PathBuf::from),
            shot_asked: false,
            frames: 0,
            height: size(crate::OS).y,
            reopened: Arc::new(AtomicBool::new(false)),
        }
    }

    fn close(&mut self, ctx: &egui::Context, reopen: bool) {
        if self.closing {
            return;
        }
        self.closing = true;
        if reopen {
            self.reopen();
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    fn reopen(&self) {
        if self.plan.reopen && !self.reopened.swap(true, Ordering::SeqCst) {
            if let Err(e) = work::reopen(&self.plan) {
                work::log(&self.plan, &e);
            }
        }
    }

    fn title(&self) -> String {
        match &self.state.outcome {
            Some(Ok(())) => format!("Updated to {}", self.plan.version),
            Some(Err(_)) => "Couldn't update".to_string(),
            None => format!("Updating {}", self.plan.name),
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        self.frames += 1;
        if let Some(rx) = &self.rx {
            while let Ok(event) = rx.try_recv() {
                self.state.apply(event);
            }
        }
        if matches!(self.state.outcome, Some(Ok(()))) {
            let at = *self.done_at.get_or_insert_with(Instant::now);
            if at.elapsed().as_secs_f32() > HOLD_DONE {
                self.close(ctx, true);
            }
        }

        if ctx.input(|i| i.viewport().close_requested()) && !self.closing {
            if self.state.installing() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            } else {
                if !self.state.finished() {
                    self.cancel.store(true, Ordering::SeqCst);
                }
                self.closing = true;
            }
        }

        self.screenshot(ctx);
        self.paint(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
    }

    fn screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.shot.clone() else {
            return;
        };
        let image = ctx.input(|i| {
            i.raw.events.iter().find_map(|e| match e {
                egui::Event::Screenshot { image, .. } => Some(image.clone()),
                _ => None,
            })
        });
        if let Some(image) = image {
            let colours = colours(&image);
            let saved = save_png(&path, &image);
            work::log(&self.plan, &format!("updater window drawn: {colours} colours, {}x{}{}", image.size[0], image.size[1], saved.err().map(|e| format!(", not saved: {e}")).unwrap_or_default()));
            self.shot = None;
            return;
        }
        if !self.shot_asked && self.frames > 6 && self.state.step.map(|s| s.index() >= Step::Download.index()).unwrap_or(false) {
            self.shot_asked = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
        }
    }

    fn paint(&mut self, ctx: &egui::Context) {
        let p = pal();
        let native = chrome::native_frame(crate::OS);
        let mut reopen_old = false;
        let mut page = false;
        let mut cancel = false;
        let mut site = false;
        egui::CentralPanel::default().frame(egui::Frame::none()).show(ctx, |ui| {
            let rect = ui.max_rect();
            let round = egui::Rounding::same(if native { 0.0 } else { theme::RADIUS });
            let drag = ui.interact(rect, egui::Id::new("ncf-updater-drag"), Sense::click_and_drag());
            if drag.drag_started() && !native {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            let painter = ui.painter().clone();
            painter.rect_filled(rect, round, p.void);
            chrome::column_guides(&painter, rect);
            painter.rect_stroke(rect.shrink(0.5), round, Stroke::new(1.0_f32, p.line_strong));

            let left = rect.left() + 40.0;
            let width = rect.width() - 80.0;
            let top = rect.top() + if native { 40.0 } else { 26.0 };

            let mark = Rect::from_min_size(Pos2::new(left, top), Vec2::splat(22.0));
            self.info.mark.paint(&painter, mark);
            let eyebrow = format!("UPDATE / V{} TO V{}", self.info.version, self.plan.version);
            theme::tracked_text(&painter, Pos2::new(left + 32.0, mark.center().y), &eyebrow, theme::font_hud(theme::T_MICRO + 0.5), p.text_2, 0.8);
            let logo_h = 13.0;
            let logo_at = Rect::from_min_size(Pos2::new(left + width - crate::mark::logo_width(logo_h), mark.center().y - logo_h * 0.5), Vec2::new(crate::mark::logo_width(logo_h), logo_h));
            let logo_hit = ui.interact(logo_at, egui::Id::new("ncf-updater-logo"), Sense::click());
            let logo_hot = anim::ease(ctx, logo_hit.id, logo_hit.hovered(), anim::HOVER);
            crate::mark::logo(&painter, logo_at.left_center(), logo_h, theme::mix(p.text_3, p.text, logo_hot));
            if logo_hit.hovered() {
                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if logo_hit.clicked() {
                site = true;
            }

            let title = self.title().to_uppercase();
            let probe = painter.layout_no_wrap(title.clone(), theme::font_display(40.0), p.text);
            let size = (40.0 * width * 0.97 / probe.rect.width().max(1.0)).min(40.0);
            let galley = painter.layout_no_wrap(title, theme::font_display(size), p.text);
            let title_top = top + 22.0 + 12.0;
            painter.galley(Pos2::new(left, title_top + (40.0 - galley.rect.height()) * 0.5), galley, p.text);
            let track_y = title_top + 40.0 + 8.0;

            let failed = matches!(self.state.outcome, Some(Err(_)));
            let done = matches!(self.state.outcome, Some(Ok(())));
            let current = self.state.step.map(|s| s.index());
            let row_top = track_y + 2.0 + 16.0;
            let t = ui.input(|i| i.time) as f32;
            for (k, step) in STEPS.iter().enumerate() {
                let y = row_top + k as f32 * 20.0;
                let marker = Rect::from_center_size(Pos2::new(left + 4.0, y), Vec2::splat(7.0));
                let (state, colour) = match current {
                    _ if done => (2, p.text_3),
                    Some(c) if k < c => (2, p.text_3),
                    Some(c) if k == c && failed => (3, p.danger_hi),
                    Some(c) if k == c => (1, p.text),
                    _ => (0, p.text_3),
                };
                match state {
                    2 => {
                        painter.rect_filled(marker, egui::Rounding::ZERO, p.text_3);
                    }
                    3 => {
                        painter.rect_filled(marker, egui::Rounding::ZERO, p.danger_hi);
                    }
                    1 => {
                        let pulse = if anim::enabled() { 0.55 + 0.45 * (t * 4.0).sin().abs() } else { 1.0 };
                        painter.rect_filled(marker, egui::Rounding::ZERO, theme::wash(p.signal_hi, (255.0 * pulse) as u8));
                    }
                    _ => {
                        painter.rect_stroke(marker, egui::Rounding::ZERO, Stroke::new(1.0_f32, p.line_strong));
                    }
                }
                let label = if *step == Step::Reopen && !self.plan.reopen { "Finishing".to_string() } else { step.label(&self.plan) };
                theme::tracked_text(&painter, Pos2::new(left + 18.0, y), &label.to_uppercase(), theme::font_hud(theme::T_MICRO + 0.5), colour, 0.8);
                if state == 1 && *step == Step::Download && self.state.progress > 0.0 {
                    let pct = format!("{:.0}%", self.state.progress * 100.0);
                    let font = theme::font_hud(theme::T_MICRO + 0.5);
                    let w = theme::tracked_width(ctx, &pct, font.clone(), 0.8);
                    theme::tracked_text(&painter, Pos2::new(left + width - w, y), &pct, font, p.signal_hi, 0.8);
                }
            }

            let note_top = row_top + (STEPS.len() - 1) as f32 * 20.0 + 6.0 + 12.0;
            let (note, note_colour) = match &self.state.outcome {
                Some(Err(why)) => (format!("{} The version you had is still installed.", sentence(why)), p.text),
                Some(Ok(())) => (format!("Opening {}…", self.plan.name), p.text_2),
                None => (self.state.detail.clone(), p.text_3),
            };
            let wrapped = painter.layout(note, theme::font(theme::T_CAP), note_colour, width);
            let note_h = wrapped.rect.height().max(15.0);
            painter.galley(Pos2::new(left, note_top), wrapped, note_colour);
            let rule_y = note_top + note_h + 12.0;
            let row_mid = rule_y + 22.0;
            let needed = (row_mid + 15.0 + 20.0 - rect.top()).ceil();
            if needed > self.height + 0.5 {
                self.height = needed;
                ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(Vec2::new(SIZE.x, needed)));
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(Vec2::new(SIZE.x, needed)));
            }

            painter.line_segment([Pos2::new(left, rule_y), Pos2::new(left + width, rule_y)], Stroke::new(1.0_f32, p.line));
            let overall = match (done, current) {
                (true, _) => 1.0,
                (_, Some(c)) => (c as f32 + if STEPS[c] == Step::Download { self.state.progress } else { 0.5 }) / STEPS.len() as f32,
                _ => 0.0,
            };
            let track = Rect::from_min_size(Pos2::new(left, track_y), Vec2::new(width, 2.0));
            painter.rect_filled(track, egui::Rounding::ZERO, p.line_strong);
            painter.rect_filled(
                Rect::from_min_size(track.min, Vec2::new(track.width() * overall.clamp(0.0, 1.0), 2.0)),
                egui::Rounding::ZERO,
                if failed { p.danger_hi } else { p.signal_hi },
            );

            let bar = Rect::from_center_size(Pos2::new(left + width * 0.5, row_mid), Vec2::new(width, 32.0));
            let mut buttons = ui.new_child(egui::UiBuilder::new().max_rect(bar).layout(Layout::right_to_left(Align::Center)));
            if failed {
                if ui::button(&mut buttons, &format!("Open {}", self.plan.name), None, true) {
                    reopen_old = true;
                }
                buttons.add_space(6.0);
                if !self.plan.repo.is_empty() && ui::quiet_button(&mut buttons, "Download page", Some(crate::icons::Icon::External)) {
                    page = true;
                }
            } else if !done && !self.state.installing() && ui::quiet_button(&mut buttons, "Cancel", None) {
                cancel = true;
            } else if self.state.installing() {
                theme::tracked_text(&painter, Pos2::new(left, row_mid), "PLEASE WAIT · THIS TAKES A MOMENT", theme::font_hud(theme::T_MICRO + 0.5), p.text_3, 0.8);
            }
        });
        if page {
            crate::platform::open(self.plan.page());
        }
        if site {
            crate::platform::open(crate::tokens::BRAND_URL);
        }
        if reopen_old {
            self.close(ctx, true);
        }
        if cancel {
            self.cancel.store(true, Ordering::SeqCst);
            self.close(ctx, true);
        }
    }
}

fn sentence(text: &str) -> String {
    let mut s = text.trim().to_string();
    if let Some(first) = s.get(0..1) {
        s.replace_range(0..1, &first.to_uppercase());
    }
    if !s.ends_with(['.', '!', '?']) {
        s.push('.');
    }
    s
}

pub fn colours(image: &egui::ColorImage) -> usize {
    let mut seen: Vec<[u8; 3]> = Vec::new();
    for c in image.pixels.iter().step_by(53) {
        let rgb = [c.r(), c.g(), c.b()];
        if seen.len() < 256 && !seen.contains(&rgb) {
            seen.push(rgb);
        }
    }
    seen.len()
}

fn save_png(path: &std::path::Path, image: &egui::ColorImage) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), image.size[0] as u32, image.size[1] as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_srgba_unmultiplied()).collect();
    writer.write_image_data(&bytes).map_err(|e| e.to_string())
}

struct Host {
    updater: Updater,
}

impl eframe::App for Host {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.updater.ui(ctx);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        chrome::clear_color(crate::OS, pal().void)
    }
}

pub fn look(info: &AppInfo) -> Look {
    Look::parse(&std::fs::read_to_string(crate::app::settings_path(info)).unwrap_or_default())
}

pub fn run(info: AppInfo, plan: Plan) -> i32 {
    let cancel = Arc::new(AtomicBool::new(false));
    let (rx, worker) = work::spawn(plan.clone(), cancel.clone());
    run_with(info, plan, rx, worker, cancel)
}

pub fn run_with(info: AppInfo, plan: Plan, rx: Receiver<Event>, worker: work::Worker, cancel: Arc<AtomicBool>) -> i32 {
    let look = look(&info);
    let started = Arc::new(AtomicBool::new(false));
    let reopened = Arc::new(AtomicBool::new(false));
    let rx = Arc::new(std::sync::Mutex::new(Some(rx)));
    for renderer in crate::app::renderers() {
        let builder = egui::ViewportBuilder::default()
            .with_title(format!("Updating {}", info.name))
            .with_icon(Arc::new(info.mark.icon_data(128)))
            .with_app_id(info.id)
            .with_inner_size(size(crate::OS))
            .with_min_inner_size(size(crate::OS));
        let viewport = chrome::viewport(builder, crate::OS).with_resizable(false).with_maximize_button(false);
        let options = eframe::NativeOptions { viewport, renderer, centered: true, ..Default::default() };
        let (flag, plan2, rx2, cancel2, reopened2) = (started.clone(), plan.clone(), rx.clone(), cancel.clone(), reopened.clone());
        let light = look.light_mode;
        let animations = look.animations;
        let outcome = eframe::run_native(
            &format!("{} updater", info.name),
            options,
            Box::new(move |cc| {
                flag.store(true, Ordering::SeqCst);
                theme::install_fonts(&cc.egui_ctx);
                theme::set_palette(!light);
                theme::apply(&cc.egui_ctx);
                anim::set_enabled(animations);
                let rx = rx2.lock().ok().and_then(|mut r| r.take());
                let mut updater = Updater::new(info, plan2, rx, cancel2);
                updater.reopened = reopened2;
                Ok(Box::new(Host { updater }))
            }),
        );
        if outcome.is_ok() || started.load(Ordering::SeqCst) {
            break;
        }
        if let Err(e) = outcome {
            work::log(&plan, &format!("the {renderer} renderer could not open the updater window ({e})"));
        }
    }
    if !started.load(Ordering::SeqCst) {
        work::log(&plan, "no window could be opened; updating without one");
    }
    if let Some(rx) = rx.lock().ok().and_then(|mut r| r.take()) {
        work::print(&plan, rx);
    }
    work::finish(&plan, worker, reopened.load(Ordering::SeqCst))
}
