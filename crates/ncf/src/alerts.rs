#![allow(dead_code)]

use web_time::{Duration, Instant};

use eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, Sense, Vec2};

use crate::anim;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};

const TOAST_TTL: Duration = Duration::from_millis(3400);
const RISE: f32 = 0.42;
const FALL: f32 = 0.22;
const CARD_W: f32 = 388.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tone {
    Ok,
    Info,
    Warn,
    Danger,
}

impl Tone {
    fn accent(self) -> Color32 {
        let p = pal();
        match self {
            Tone::Ok => p.ok,
            Tone::Info => p.signal,
            Tone::Warn => p.warn,
            Tone::Danger => p.danger,
        }
    }
    fn icon(self) -> Icon {
        match self {
            Tone::Ok => Icon::Check,
            Tone::Info => Icon::Info,
            Tone::Warn => Icon::Warning,
            Tone::Danger => Icon::Trash,
        }
    }
}

struct Prompt {
    hint: String,
    text: String,
    focused: bool,
}

enum Kind<A> {
    Toast,
    Ask {
        yes: String,
        action: A,
        prompt: Option<Prompt>,
    },
}

struct Item<A> {
    id: u64,
    title: String,
    body: String,
    tone: Tone,
    kind: Kind<A>,
    born: Instant,
    dying: Option<Instant>,
}

impl<A> Item<A> {
    fn modal(&self) -> bool {
        matches!(self.kind, Kind::Ask { .. })
    }
}

pub struct Answer<A> {
    pub action: A,
    pub text: Option<String>,
}

pub struct Deck<A> {
    items: Vec<Item<A>>,
    next: u64,
}

impl<A> Default for Deck<A> {
    fn default() -> Self {
        Deck {
            items: Vec::new(),
            next: 1,
        }
    }
}

impl<A> Deck<A> {
    fn push(&mut self, item: Kind<A>, title: &str, body: &str, tone: Tone) {
        let id = self.next;
        self.next += 1;
        while self.items.iter().filter(|i| !i.modal()).count() >= 4 {
            if let Some(pos) = self.items.iter().position(|i| !i.modal()) {
                self.items.remove(pos);
            } else {
                break;
            }
        }
        self.items.push(Item {
            id,
            title: title.to_string(),
            body: body.to_string(),
            tone,
            kind: item,
            born: Instant::now(),
            dying: None,
        });
    }

    pub fn say(&mut self, title: &str, body: &str, tone: Tone) {
        self.push(Kind::Toast, title, body, tone);
    }

    pub fn ok(&mut self, title: &str, body: &str) {
        self.say(title, body, Tone::Ok);
    }

    pub fn warn(&mut self, title: &str, body: &str) {
        self.say(title, body, Tone::Warn);
    }

    pub fn ask(&mut self, title: &str, body: &str, yes: &str, tone: Tone, action: A) {
        self.push(
            Kind::Ask {
                yes: yes.to_string(),
                action,
                prompt: None,
            },
            title,
            body,
            tone,
        );
    }

    pub fn prompt(
        &mut self,
        title: &str,
        body: &str,
        hint: &str,
        seed: &str,
        yes: &str,
        action: A,
    ) {
        self.push(
            Kind::Ask {
                yes: yes.to_string(),
                action,
                prompt: Some(Prompt {
                    hint: hint.to_string(),
                    text: seed.to_string(),
                    focused: false,
                }),
            },
            title,
            body,
            Tone::Info,
        );
    }

    pub fn asking(&self) -> bool {
        self.items.iter().any(|i| i.modal() && i.dying.is_none())
    }

    pub fn dismiss_top(&mut self) -> bool {
        if let Some(item) = self
            .items
            .iter_mut()
            .rev()
            .find(|i| i.modal() && i.dying.is_none())
        {
            item.dying = Some(Instant::now());
            return true;
        }
        false
    }

    #[allow(dead_code)]
    pub fn take_top(&mut self, text: Option<String>) -> Option<Answer<A>> {
        let pos = self
            .items
            .iter()
            .rposition(|i| i.modal() && i.dying.is_none())?;
        let item = self.items.remove(pos);
        match item.kind {
            Kind::Ask { action, prompt, .. } => Some(Answer {
                action,
                text: text.or_else(|| prompt.map(|q| q.text)),
            }),
            Kind::Toast => None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<Answer<A>> {
        let fall = Duration::from_secs_f32(FALL);
        self.items.retain(|i| match i.dying {
            Some(t) => t.elapsed() < fall,
            None => true,
        });
        for i in self.items.iter_mut() {
            if !i.modal() && i.dying.is_none() && i.born.elapsed() > TOAST_TTL {
                i.dying = Some(Instant::now());
            }
        }
        if self.items.is_empty() {
            return None;
        }
        anim::keep_going(ctx);

        let screen = ctx.screen_rect();
        let modal = self.items.iter().any(|i| i.modal() && i.dying.is_none());

        if modal {
            let veil = anim::ease(ctx, "ncf-veil", true, 0.24);
            egui::Area::new(egui::Id::new("ncf-scrim"))
                .order(egui::Order::Foreground)
                .fixed_pos(screen.min)
                .interactable(true)
                .show(ctx, |ui| {
                    let (rect, _) = ui.allocate_exact_size(screen.size(), Sense::click());
                    ui.painter().rect_filled(
                        rect,
                        egui::Rounding::ZERO,
                        Color32::from_black_alpha((155.0 * veil) as u8),
                    );
                });
        } else {
            ctx.animate_bool_with_time(egui::Id::new("ncf-veil"), false, 0.0);
        }

        let mut answer = None;
        let mut dismiss: Option<u64> = None;
        let mut confirm: Option<u64> = None;

        let mut toast_y = screen.bottom() - 96.0;

        for idx in (0..self.items.len()).rev() {
            let (id, is_modal) = (self.items[idx].id, self.items[idx].modal());
            let raw = if let Some(d) = self.items[idx].dying {
                1.0 - (d.elapsed().as_secs_f32() / FALL).clamp(0.0, 1.0)
            } else {
                (self.items[idx].born.elapsed().as_secs_f32() / RISE).clamp(0.0, 1.0)
            };
            let leaving = self.items[idx].dying.is_some();
            let eased = if !anim::enabled() {
                1.0
            } else if leaving {
                anim::smootherstep(raw)
            } else if is_modal {
                anim::ease_out_back(raw)
            } else {
                anim::pop(raw)
            };

            let size = self.card_size(ctx, idx);
            let rect = if is_modal {
                let scale = 0.90 + 0.10 * eased;
                Rect::from_center_size(
                    Pos2::new(screen.center().x, screen.center().y - 10.0),
                    size * scale,
                )
            } else {
                let rest = toast_y - size.y;
                let lift = (1.0 - eased) * (size.y + 26.0);
                toast_y = rest - 8.0;
                Rect::from_min_size(
                    Pos2::new(screen.center().x - size.x * 0.5, rest + lift),
                    size,
                )
            };

            match self.card(ctx, idx, rect) {
                CardOut::None => {}
                CardOut::Dismiss => dismiss = Some(id),
                CardOut::Confirm => confirm = Some(id),
            }

            if !is_modal && toast_y < screen.top() + 120.0 {
                break;
            }
        }

        if let Some(id) = confirm {
            if let Some(pos) = self.items.iter().position(|i| i.id == id) {
                let item = self.items.remove(pos);
                if let Kind::Ask { action, prompt, .. } = item.kind {
                    answer = Some(Answer {
                        action,
                        text: prompt.map(|p| p.text),
                    });
                }
            }
        } else if let Some(id) = dismiss {
            if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
                item.dying = Some(Instant::now());
            }
        }
        answer
    }

    fn card_size(&self, ctx: &egui::Context, idx: usize) -> Vec2 {
        let item = &self.items[idx];
        let width_of = |text: &str, font: egui::FontId| -> f32 {
            if text.is_empty() {
                0.0
            } else {
                ctx.fonts(|f| {
                    f.layout_no_wrap(text.to_string(), font, Color32::WHITE)
                        .rect
                        .width()
                })
            }
        };
        if !item.modal() {
            let title = width_of(&item.title, theme::font_semi(theme::T_SM));
            let body = width_of(&item.body, theme::font(theme::T_CAP));
            let w = (title + body + if body > 0.0 { 12.0 } else { 0.0 } + 60.0).clamp(180.0, 470.0);
            return Vec2::new(w, 36.0);
        }

        let wrap = CARD_W - 56.0;
        let body_h = if item.body.is_empty() {
            0.0
        } else {
            ctx.fonts(|f| {
                f.layout(
                    item.body.clone(),
                    theme::font(theme::T_CAP + 1.0),
                    Color32::WHITE,
                    wrap,
                )
                .rect
                .height()
            }) + 10.0
        };
        let mut h = 40.0 + body_h + 42.0;
        if let Kind::Ask { prompt, .. } = &item.kind {
            if prompt.is_some() {
                h += 40.0;
            }
        }
        Vec2::new(CARD_W, h)
    }

    fn card(&mut self, ctx: &egui::Context, idx: usize, rect: Rect) -> CardOut {
        let p = pal();
        let mut out = CardOut::None;
        let tone = self.items[idx].tone;
        let accent = tone.accent();
        let is_modal = self.items[idx].modal();
        let radius = if is_modal {
            theme::RADIUS
        } else {
            rect.height() * 0.5
        };

        egui::Area::new(egui::Id::new(("ncf-alert", self.items[idx].id)))
            .order(egui::Order::Tooltip)
            .fixed_pos(rect.min)
            .movable(false)
            .show(ctx, |ui| {
                ui.set_width(rect.width());
                let painter = ui.painter();

                theme::panel_rounded(painter, rect, radius);
                theme::fill_grad_poly(
                    painter,
                    &theme::rounded_poly(rect, radius),
                    theme::wash(accent, if p.dark { 40 } else { 30 }),
                    theme::wash(accent, 0),
                    Vec2::new(0.35, 1.0),
                );

                let icon_c = Pos2::new(
                    rect.left() + 20.0,
                    rect.top() + if is_modal { 23.0 } else { rect.height() * 0.5 },
                );
                icons::draw(
                    painter,
                    Rect::from_center_size(icon_c, Vec2::splat(14.0)),
                    tone.icon(),
                    accent,
                );

                if !is_modal {
                    let tw = painter
                        .layout_no_wrap(
                            self.items[idx].title.clone(),
                            theme::font_semi(theme::T_SM),
                            p.text,
                        )
                        .rect
                        .width();
                    painter.text(
                        Pos2::new(rect.left() + 35.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &self.items[idx].title,
                        theme::font_semi(theme::T_SM),
                        p.text,
                    );
                    if !self.items[idx].body.is_empty() {
                        painter.text(
                            Pos2::new(rect.left() + 35.0 + tw + 10.0, rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            crate::ui::elide(&self.items[idx].body, 46),
                            theme::font(theme::T_CAP),
                            p.text_3,
                        );
                    }
                    let (_, resp) = ui.allocate_exact_size(rect.size(), Sense::click());
                    if resp.clicked() {
                        out = CardOut::Dismiss;
                    }
                    return;
                }

                painter.text(
                    Pos2::new(rect.left() + 37.0, rect.top() + 23.0),
                    egui::Align2::LEFT_CENTER,
                    &self.items[idx].title,
                    theme::font_semi(theme::T_H - 1.0),
                    p.text,
                );

                let mut y = 41.0;
                if !self.items[idx].body.is_empty() {
                    let galley = painter.layout(
                        self.items[idx].body.clone(),
                        theme::font(theme::T_CAP + 1.0),
                        p.text_2,
                        rect.width() - 56.0,
                    );
                    let h = galley.rect.height();
                    painter.galley(rect.left_top() + Vec2::new(28.0, y), galley, p.text_2);
                    y += h + 12.0;
                }

                let has_prompt =
                    matches!(&self.items[idx].kind, Kind::Ask { prompt: Some(_), .. });
                if has_prompt {
                    let field = Rect::from_min_size(
                        rect.left_top() + Vec2::new(28.0, y),
                        Vec2::new(rect.width() - 56.0, 30.0),
                    );
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(field)
                            .layout(Layout::left_to_right(Align::Center)),
                    );
                    let field_id = egui::Id::new(("ncf-alert-field", self.items[idx].id));
                    if let Kind::Ask {
                        prompt: Some(pr), ..
                    } = &mut self.items[idx].kind
                    {
                        let resp = child.add(
                            egui::TextEdit::singleline(&mut pr.text)
                                .id(field_id)
                                .desired_width(f32::INFINITY)
                                .margin(egui::Margin::symmetric(10.0, 6.0))
                                .hint_text(pr.hint.clone()),
                        );
                        if !pr.focused {
                            resp.request_focus();
                            pr.focused = true;
                        }
                        if resp.lost_focus() && child.input(|i| i.key_pressed(egui::Key::Enter)) {
                            out = CardOut::Confirm;
                        }
                    }
                    y += 40.0;
                }

                let yes = match &self.items[idx].kind {
                    Kind::Ask { yes, .. } => yes.clone(),
                    _ => String::new(),
                };
                let row = Rect::from_min_size(
                    rect.left_top() + Vec2::new(28.0, y),
                    Vec2::new(rect.width() - 56.0, 30.0),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(row)
                        .layout(Layout::right_to_left(Align::Center)),
                );
                if pill_button(&mut child, &yes, accent, true) {
                    out = CardOut::Confirm;
                }
                if pill_button(&mut child, "Cancel", p.text_2, false) {
                    out = CardOut::Dismiss;
                }
                ui.allocate_space(Vec2::new(rect.width(), 0.0));
            });

        if is_modal && self.items[idx].dying.is_none() {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
                out = CardOut::Dismiss;
            }
            let has_prompt = matches!(&self.items[idx].kind, Kind::Ask { prompt: Some(_), .. });
            if !has_prompt
                && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter))
            {
                out = CardOut::Confirm;
            }
        }
        out
    }
}

enum CardOut {
    None,
    Dismiss,
    Confirm,
}

fn pill_button(ui: &mut egui::Ui, label: &str, tint: Color32, filled: bool) -> bool {
    let p = pal();
    let galley = ui.painter().layout_no_wrap(
        label.to_string(),
        theme::font_semi(theme::T_SM),
        if filled { theme::on(tint) } else { tint },
    );
    let size = Vec2::new(galley.rect.width() + 28.0, 28.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER) > 0.45;
    if filled {
        theme::grad_rect(
            ui.painter(),
            rect,
            theme::RADIUS,
            if hot { theme::lighten(tint, 0.16) } else { tint },
            theme::darken(tint, 0.18),
            Vec2::new(0.0, 1.0),
        );
    } else {
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(theme::RADIUS),
            if hot { p.panel } else { Color32::TRANSPARENT },
        );
        ui.painter().rect_filled(
            rect,
            egui::Rounding::same(theme::RADIUS),
            theme::wash(p.text, if hot { 30 } else { 14 }),
        );
    }
    ui.painter().galley(
        rect.center() - galley.rect.size() * 0.5,
        galley,
        if filled { theme::on(tint) } else { tint },
    );
    resp.clicked()
}
