use web_time::Instant;

use eframe::egui::{self, Pos2, Rect, Sense, Vec2};

use crate::anim;
use crate::theme::{self, pal};
use crate::update::Status;

pub const CARD: Vec2 = Vec2::new(540.0, 296.0);

const HOLD: f32 = 1.55;
const FADE: f32 = 0.40;

pub struct Splash {
    app: crate::AppInfo,
    born: Instant,
    skip_at: Option<f32>,
    status: Status,
    inline: bool,
}

pub fn own_window(os: crate::Os) -> bool {
    !crate::chrome::native_frame(os)
}

pub fn status_line(status: &Status) -> Option<(String, bool)> {
    match status {
        Status::Off => None,
        Status::Checking => Some(("CHECKING FOR UPDATES…".to_string(), false)),
        Status::Latest => Some(("UP TO DATE".to_string(), false)),
        Status::Newer(r) => Some((format!("V{} IS OUT", r.version), true)),
        Status::Failed => Some(("COULD NOT CHECK FOR UPDATES".to_string(), false)),
    }
}

impl Splash {
    pub fn new(app: crate::AppInfo) -> Splash {
        Splash { app, born: Instant::now(), skip_at: None, status: Status::Off, inline: false }
    }

    pub fn inline(app: crate::AppInfo) -> Splash {
        Splash { inline: true, ..Splash::new(app) }
    }

    pub fn for_os(app: crate::AppInfo, os: crate::Os) -> Splash {
        if own_window(os) {
            Splash::new(app)
        } else {
            Splash::inline(app)
        }
    }

    pub fn is_inline(&self) -> bool {
        self.inline
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let p = pal();
        let t = self.born.elapsed().as_secs_f32();
        if !anim::enabled() {
            return false;
        }
        let starts = self.skip_at.unwrap_or(HOLD);
        let leaving = ((t - starts) / FADE).clamp(0.0, 1.0);
        if leaving >= 1.0 {
            return false;
        }
        ctx.request_repaint();
        let alpha = 1.0 - anim::smootherstep(leaving);

        let screen = ctx.screen_rect();
        egui::Area::new(egui::Id::new("ncf-splash"))
            .order(egui::Order::Tooltip)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ctx, |ui| {
                let (whole, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                let painter = ui.painter();
                let round = egui::Rounding::same(if self.inline { 0.0 } else { theme::RADIUS });
                let rect = if self.inline { Rect::from_center_size(whole.center(), CARD.min(whole.size())) } else { whole };

                if self.inline {
                    painter.rect_filled(whole, egui::Rounding::ZERO, theme::wash(p.void, (255.0 * alpha) as u8));
                    if alpha > 0.99 {
                        crate::chrome::column_guides(painter, whole);
                    }
                }
                painter.rect_filled(rect, round, theme::wash(p.void, (255.0 * alpha) as u8));
                if alpha > 0.99 && !self.inline {
                    crate::chrome::column_guides(painter, rect);
                }
                painter.rect_stroke(
                    rect.shrink(0.5),
                    round,
                    egui::Stroke::new(1.0_f32, theme::wash(p.line_strong, (p.line_strong.a() as f32 * alpha) as u8)),
                );

                let ink = |c: egui::Color32, k: f32| theme::wash(c, (c.a() as f32 * alpha * k) as u8);
                let arrive = anim::ease_out_expo((t / 0.55).clamp(0.0, 1.0));
                let drift = (1.0 - arrive) * 10.0;
                let left = rect.left() + 40.0;
                let width = rect.width() - 80.0;

                let mark = Rect::from_min_size(Pos2::new(left, rect.top() + 36.0), Vec2::splat(22.0));
                self.app.mark.paint(painter, mark);
                let eyebrow = if self.app.tagline.is_empty() {
                    format!("V{}", self.app.version)
                } else {
                    format!("V{} / {}", self.app.version, self.app.tagline.trim_end_matches('.').to_uppercase())
                };
                theme::tracked_text(
                    painter,
                    Pos2::new(left + 32.0, mark.center().y),
                    &eyebrow,
                    theme::font_hud(theme::T_MICRO + 0.5),
                    ink(p.text_2, 1.0),
                    0.8,
                );

                let word = self.app.name.to_uppercase();
                let probe = painter.layout_no_wrap(word.clone(), theme::font_display(100.0), p.text);
                let size = (100.0 * width / probe.rect.width().max(1.0)).min(rect.height() * 0.42);
                let name = painter.layout_no_wrap(word, theme::font_display(size), ink(p.text, 1.0));
                painter.galley(Pos2::new(left + drift, rect.top() + 84.0), name, ink(p.text, 1.0));

                let foot_y = rect.bottom() - 40.0;
                painter.line_segment(
                    [Pos2::new(left, foot_y - 26.0), Pos2::new(left + width, foot_y - 26.0)],
                    egui::Stroke::new(1.0_f32, ink(p.line, 1.0)),
                );
                let foot_font = theme::font_hud(theme::T_MICRO + 0.5);
                let by = theme::tracked_text(painter, Pos2::new(left, foot_y), "BY", foot_font.clone(), ink(p.text_3, 1.0), 0.8);
                let logo = crate::mark::logo(painter, Pos2::new(left + by + 8.0, foot_y), ((theme::T_MICRO + 0.5) * 1.35).round(), ink(p.text_2, 1.0));
                theme::tracked_text(painter, Pos2::new(logo.right() + 8.0, foot_y), "/ NATIVECOMPUTE.NET", foot_font, ink(p.text_3, 1.0), 0.8);
                if let Some((line, news)) = status_line(&self.status) {
                    let font = theme::font_hud(theme::T_MICRO + 0.5);
                    let w = theme::tracked_width(ui.ctx(), &line, font.clone(), 0.8);
                    let colour = if news { p.signal_hi } else { p.text_3 };
                    theme::tracked_text(painter, Pos2::new(left + width - w, foot_y - 13.0), &line, font, ink(colour, 1.0), 0.8);
                }
                let track = Rect::from_min_size(Pos2::new(left + width - 140.0, foot_y - 1.0), Vec2::new(140.0, 2.0));
                painter.rect_filled(track, egui::Rounding::ZERO, ink(p.line_strong, 1.0));
                let done = anim::ease_out_expo((t / HOLD).clamp(0.0, 1.0));
                painter.rect_filled(
                    Rect::from_min_size(track.min, Vec2::new(track.width() * done, 2.0)),
                    egui::Rounding::ZERO,
                    ink(p.signal_hi, 1.0),
                );

                if resp.clicked() && self.skip_at.is_none() {
                    self.skip_at = Some(t);
                }
            });

        if self.skip_at.is_none()
            && ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Enter))
        {
            self.skip_at = Some(t);
        }
        true
    }
}

pub fn centre(ctx: &egui::Context, size: Vec2) {
    if let Some(monitor) = ctx.input(|i| i.viewport().monitor_size) {
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(centred(monitor, size, crate::OS)));
    }
}

pub fn centred(monitor: Vec2, size: Vec2, os: crate::Os) -> Pos2 {
    let top = if os.is_mac() { 38.0 } else { 0.0 };
    let at = ((monitor - size) * 0.5).max(Vec2::ZERO);
    Pos2::new(at.x, at.y.max(top))
}

pub fn settle(ctx: &egui::Context, want: Vec2) -> bool {
    let (monitor, inner) = ctx.input(|i| (i.viewport().monitor_size, i.viewport().inner_rect.map(|r| r.size())));
    let (Some(monitor), Some(inner)) = (monitor, inner) else {
        return false;
    };
    let fit = crate::chrome::fit(want, Some(monitor), crate::OS);
    if inner.x > fit.x + 1.0 || inner.y > fit.y + 1.0 {
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(fit));
        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(centred(monitor, fit, crate::OS)));
    }
    true
}
