use web_time::Instant;

use eframe::egui::{self, Pos2, Rect, Sense, Vec2};

use crate::anim;
use crate::theme::{self, pal};
use crate::update::Status;

const PAD_TOP: f32 = 28.0;
const BROW: f32 = 22.0;
const TITLE_GAP: f32 = 14.0;
const TITLE_BAND: f32 = 54.0;
const BAR_GAP: f32 = 10.0;
const RULE_GAP: f32 = 22.0;
const FOOT_GAP: f32 = 22.0;
const PAD_BOTTOM: f32 = 30.0;
pub const PROMPT_H: f32 = 84.0;

pub const CARD: Vec2 = Vec2::new(540.0, PAD_TOP + BROW + TITLE_GAP + TITLE_BAND + BAR_GAP + 2.0 + RULE_GAP + FOOT_GAP + PAD_BOTTOM);

const HOLD: f32 = 1.55;
const FADE: f32 = 0.40;
const LINGER: f32 = 0.35;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Offer {
    pub version: String,
    pub can_update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    UpdateNow(String),
    Download(String),
    NotNow,
    Skip(String),
}

pub struct Splash {
    app: crate::AppInfo,
    born: Instant,
    skip_at: Option<f32>,
    settled_at: Option<f32>,
    status: Status,
    inline: bool,
    bar: f32,
    offer: Option<Offer>,
    choice: Option<Choice>,
    grown: bool,
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

pub fn card_size(prompt: bool) -> Vec2 {
    CARD + Vec2::new(0.0, if prompt { PROMPT_H } else { 0.0 })
}

pub fn swallow_keys(ctx: &egui::Context) {
    ctx.input_mut(|i| {
        i.events.retain(|e| !matches!(e, egui::Event::Key { .. } | egui::Event::Text(_) | egui::Event::Paste(_) | egui::Event::Copy | egui::Event::Cut));
    });
}

fn title_size(painter: &egui::Painter, word: &str, width: f32, band: f32) -> f32 {
    let probe = painter.layout_no_wrap(word.to_string(), theme::font_display(100.0), egui::Color32::WHITE);
    (100.0 * width / probe.rect.width().max(1.0)).min(100.0 * band / probe.rect.height().max(1.0))
}

fn brow(painter: &egui::Painter, app: &crate::AppInfo, left: f32, top: f32, text: &str, ink: egui::Color32) {
    let mark = Rect::from_min_size(Pos2::new(left, top), Vec2::splat(BROW));
    app.mark.paint(painter, mark);
    theme::tracked_text(painter, Pos2::new(left + 32.0, mark.center().y), text, theme::font_hud(theme::T_MICRO + 0.5), ink, 0.8);
}

pub fn prompt_section(ui: &mut egui::Ui, app: &crate::AppInfo, offer: &Offer, area: Rect, alpha: f32) -> Option<Choice> {
    let p = pal();
    let painter = ui.painter().clone();
    let ink = |c: egui::Color32| theme::wash(c, (c.a() as f32 * alpha) as u8);
    let line = format!("Version {} is out. You have {}. Your files stay where they are.", offer.version, app.version.trim_start_matches(['v', 'V']));
    let galley = painter.layout(line, theme::font(theme::T_CAP + 1.0), ink(p.text_2), area.width());
    painter.galley(Pos2::new(area.left(), area.top() + 12.0), galley, ink(p.text_2));
    let row = Rect::from_min_size(Pos2::new(area.left(), area.bottom() - 34.0), Vec2::new(area.width(), 30.0));
    let mut buttons = ui.new_child(egui::UiBuilder::new().max_rect(row).layout(egui::Layout::right_to_left(egui::Align::Center)));
    buttons.spacing_mut().item_spacing.x = 8.0;
    let mut choice = None;
    let primary = if offer.can_update { "Update now" } else { "Download" };
    let icon = if offer.can_update { crate::icons::Icon::Download } else { crate::icons::Icon::External };
    if crate::ui::button(&mut buttons, primary, Some(icon), true) {
        choice = Some(if offer.can_update { Choice::UpdateNow(offer.version.clone()) } else { Choice::Download(offer.version.clone()) });
    }
    if crate::ui::button(&mut buttons, "Not now", None, false) {
        choice = Some(Choice::NotNow);
    }
    if crate::ui::quiet_button(&mut buttons, "Skip this version", None) {
        choice = Some(Choice::Skip(offer.version.clone()));
    }
    if choice.is_none() {
        ui.input(|i| {
            if i.key_pressed(egui::Key::Enter) {
                choice = Some(if offer.can_update { Choice::UpdateNow(offer.version.clone()) } else { Choice::Download(offer.version.clone()) });
            } else if i.key_pressed(egui::Key::Escape) {
                choice = Some(Choice::NotNow);
            }
        });
    }
    choice
}

impl Splash {
    pub fn new(app: crate::AppInfo) -> Splash {
        Splash {
            app,
            born: Instant::now(),
            skip_at: None,
            settled_at: None,
            status: Status::Off,
            inline: false,
            bar: 0.0,
            offer: None,
            choice: None,
            grown: false,
        }
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

    pub fn wants_offer(&self) -> bool {
        self.skip_at.is_none() && self.offer.is_none() && anim::enabled()
    }

    pub fn set_offer(&mut self, offer: Offer) {
        self.offer = Some(offer);
        self.choice = None;
    }

    pub fn has_offer(&self) -> bool {
        self.offer.is_some() && self.choice.is_none()
    }

    pub fn take_choice(&mut self) -> Option<Choice> {
        let choice = self.choice.take()?;
        self.offer = None;
        Some(choice)
    }

    fn waiting(&self) -> bool {
        self.skip_at.is_none() && (self.status == Status::Checking || self.has_offer())
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        let p = pal();
        let t = self.born.elapsed().as_secs_f32();
        if !anim::enabled() {
            return false;
        }
        if !self.waiting() && self.settled_at.is_none() {
            self.settled_at = Some(t);
        }
        let starts = match (self.skip_at, self.settled_at) {
            (Some(s), _) => s,
            _ if self.waiting() => f32::INFINITY,
            (None, Some(at)) => HOLD.max(at + if at > 0.05 { LINGER } else { 0.0 }),
            (None, None) => HOLD,
        };
        let leaving = ((t - starts) / FADE).clamp(0.0, 1.0);
        if leaving >= 1.0 {
            return false;
        }
        ctx.request_repaint();
        let alpha = 1.0 - anim::smootherstep(leaving);

        let target = if self.status == Status::Checking && self.skip_at.is_none() {
            0.9 * anim::ease_out_expo((t / HOLD).clamp(0.0, 1.0))
        } else {
            anim::ease_out_expo((t / HOLD).clamp(0.0, 1.0)).max(if self.settled_at.is_some() { 1.0 } else { 0.0 })
        };
        let dt = ctx.input(|i| i.stable_dt).clamp(0.0, 0.1);
        self.bar += (target - self.bar) * (dt * 9.0).min(1.0);

        let prompting = self.has_offer() && self.skip_at.is_none();
        if prompting && !self.inline && !self.grown {
            self.grown = true;
            let size = card_size(true);
            ctx.send_viewport_cmd(egui::ViewportCommand::MinInnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            centre(ctx, size);
        }
        let grow = if prompting { anim::ease(ctx, "ncf-splash-grow", true, 0.24) } else { anim::ease(ctx, "ncf-splash-grow", false, 0.24) };

        let screen = ctx.screen_rect();
        let mut choice = None;
        egui::Area::new(egui::Id::new("ncf-splash"))
            .order(egui::Order::Tooltip)
            .fixed_pos(screen.min)
            .interactable(true)
            .show(ctx, |ui| {
                let (whole, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
                let painter = ui.painter().clone();
                let round = egui::Rounding::same(if self.inline { 0.0 } else { theme::RADIUS });
                let want = CARD + Vec2::new(0.0, PROMPT_H * grow);
                let rect = if self.inline { Rect::from_center_size(whole.center(), want.min(whole.size())) } else { whole };

                if self.inline {
                    painter.rect_filled(whole, egui::Rounding::ZERO, theme::wash(p.void, (255.0 * alpha) as u8));
                    if alpha > 0.99 {
                        crate::chrome::column_guides(&painter, whole);
                    }
                }
                painter.rect_filled(rect, round, theme::wash(p.void, (255.0 * alpha) as u8));
                if alpha > 0.99 && !self.inline {
                    crate::chrome::column_guides(&painter, rect);
                }
                painter.rect_stroke(
                    rect.shrink(0.5),
                    round,
                    egui::Stroke::new(1.0_f32, theme::wash(p.line_strong, (p.line_strong.a() as f32 * alpha) as u8)),
                );

                let ink = |c: egui::Color32| theme::wash(c, (c.a() as f32 * alpha) as u8);
                let arrive = anim::ease_out_expo((t / 0.55).clamp(0.0, 1.0));
                let drift = (1.0 - arrive) * 10.0;
                let left = rect.left() + 40.0;
                let width = rect.width() - 80.0;

                let mut y = rect.top() + PAD_TOP;
                let eyebrow = if prompting {
                    "UPDATE AVAILABLE".to_string()
                } else if self.app.tagline.is_empty() {
                    format!("V{}", self.app.version)
                } else {
                    format!("V{} / {}", self.app.version, self.app.tagline.trim_end_matches('.').to_uppercase())
                };
                brow(&painter, &self.app, left, y, &eyebrow, ink(if prompting { p.signal_hi } else { p.text_2 }));
                y += BROW + TITLE_GAP;

                let word = self.app.name.to_uppercase();
                let size = title_size(&painter, &word, width, TITLE_BAND);
                let name = painter.layout_no_wrap(word, theme::font_display(size), ink(p.text));
                let name_h = name.rect.height();
                painter.galley(Pos2::new(left + drift, y + (TITLE_BAND - name_h) * 0.5), name, ink(p.text));
                y += TITLE_BAND + BAR_GAP;

                let track = Rect::from_min_size(Pos2::new(left, y), Vec2::new(width, 2.0));
                painter.rect_filled(track, egui::Rounding::ZERO, ink(p.line_strong));
                painter.rect_filled(
                    Rect::from_min_size(track.min, Vec2::new(track.width() * self.bar.clamp(0.0, 1.0), 2.0)),
                    egui::Rounding::ZERO,
                    ink(p.signal_hi),
                );
                y += 2.0;

                if grow > 0.01 {
                    if let Some(offer) = self.offer.clone().filter(|_| prompting) {
                        let area = Rect::from_min_size(Pos2::new(left, y), Vec2::new(width, PROMPT_H * grow));
                        let mut clipped = ui.new_child(egui::UiBuilder::new().max_rect(area));
                        clipped.set_clip_rect(area.expand2(Vec2::new(4.0, 0.0)));
                        if grow > 0.98 {
                            choice = prompt_section(&mut clipped, &self.app, &offer, Rect::from_min_size(area.min, Vec2::new(width, PROMPT_H)), alpha);
                        }
                    }
                    y += PROMPT_H * grow;
                }

                y += RULE_GAP;
                painter.line_segment([Pos2::new(left, y), Pos2::new(left + width, y)], egui::Stroke::new(1.0_f32, ink(p.line)));
                let foot_y = y + FOOT_GAP;
                let foot_font = theme::font_hud(theme::T_MICRO + 0.5);
                let by = theme::tracked_text(&painter, Pos2::new(left, foot_y), "BY", foot_font.clone(), ink(p.text_3), 0.8);
                let logo_h = ((theme::T_MICRO + 0.5) * 1.35).round();
                let logo_at = Rect::from_min_size(Pos2::new(left + by + 8.0, foot_y - logo_h * 0.5), Vec2::new(crate::mark::logo_width(logo_h), logo_h));
                let logo_hit = ui.interact(logo_at.expand(4.0), egui::Id::new("ncf-splash-logo"), Sense::click());
                let logo_hot = anim::ease(ui.ctx(), logo_hit.id, logo_hit.hovered(), anim::HOVER);
                crate::mark::logo(&painter, logo_at.left_center(), logo_h, ink(theme::mix(p.text_2, p.text, logo_hot)));
                if logo_hit.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if logo_hit.clicked() {
                    crate::ui::open_site(ui.ctx());
                }
                if let Some((line, news)) = status_line(&self.status) {
                    let w = theme::tracked_width(ui.ctx(), &line, foot_font.clone(), 0.8);
                    let colour = if news { p.signal_hi } else { p.text_3 };
                    theme::tracked_text(&painter, Pos2::new(left + width - w, foot_y), &line, foot_font, ink(colour), 0.8);
                }

                if resp.clicked() && self.skip_at.is_none() && !prompting {
                    self.skip_at = Some(t);
                }
            });

        if let Some(c) = choice {
            self.choice = Some(c);
            self.skip_at = Some(t);
        } else if self.skip_at.is_none()
            && !prompting
            && ctx.input(|i| i.key_pressed(egui::Key::Escape) || i.key_pressed(egui::Key::Enter))
        {
            self.skip_at = Some(t);
        }
        swallow_keys(ctx);
        true
    }
}

pub fn prompt(ctx: &egui::Context, app: &crate::AppInfo, offer: &Offer) -> Option<Choice> {
    let p = pal();
    let screen = ctx.screen_rect();
    let veil = anim::ease(ctx, "ncf-update-veil", true, 0.24);
    let size = Vec2::new(CARD.x.min(screen.width() - 32.0), PAD_TOP + BROW + TITLE_GAP + 40.0 + BAR_GAP + PROMPT_H + 20.0);
    let rect = Rect::from_center_size(screen.center(), size);
    let mut choice = None;
    egui::Area::new(egui::Id::new("ncf-update-prompt"))
        .order(egui::Order::Tooltip)
        .fixed_pos(screen.min)
        .interactable(true)
        .show(ctx, |ui| {
            let _ = ui.allocate_exact_size(screen.size(), Sense::click());
            let painter = ui.painter().clone();
            painter.rect_filled(screen, egui::Rounding::ZERO, egui::Color32::from_black_alpha((178.0 * veil) as u8));
            painter.rect_filled(rect, egui::Rounding::same(theme::RADIUS), p.void);
            painter.rect_stroke(rect.shrink(0.5), egui::Rounding::same(theme::RADIUS), egui::Stroke::new(1.0_f32, p.line_strong));
            let left = rect.left() + 40.0;
            let width = rect.width() - 80.0;
            let mut y = rect.top() + PAD_TOP;
            brow(&painter, app, left, y, &format!("UPDATE AVAILABLE / V{}", offer.version), p.signal_hi);
            y += BROW + TITLE_GAP;
            let word = app.name.to_uppercase();
            let size = title_size(&painter, &word, width, 40.0);
            let name = painter.layout_no_wrap(word, theme::font_display(size), p.text);
            let h = name.rect.height();
            painter.galley(Pos2::new(left, y + (40.0 - h) * 0.5), name, p.text);
            y += 40.0 + BAR_GAP;
            painter.rect_filled(Rect::from_min_size(Pos2::new(left, y), Vec2::new(width, 2.0)), egui::Rounding::ZERO, p.signal_hi);
            y += 2.0;
            choice = prompt_section(ui, app, offer, Rect::from_min_size(Pos2::new(left, y), Vec2::new(width, PROMPT_H)), 1.0);
        });
    swallow_keys(ctx);
    choice
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
