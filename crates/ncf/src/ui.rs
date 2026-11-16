#![allow(dead_code)]

use eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, Response, Sense, Stroke, Vec2};

use crate::anim;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};

// pieces here---

pub fn ghost_slot(ui: &egui::Ui, rect: Rect, hot: bool, on: bool) {
    let p = pal();
    let t = anim::ease(ui.ctx(), rect.min.x as i32 * 7919 + rect.min.y as i32, hot, anim::HOVER);
    theme::hover_surface(ui.painter(), rect, rect.height().min(rect.width()) * 0.5, p.signal, t, on);
}

pub fn rule(ui: &mut egui::Ui, height: f32) {
    let p = pal();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), Sense::hover());
    ui.painter()
        .line_segment([rect.center_top(), rect.center_bottom()], Stroke::new(1.0_f32, p.line));
}

pub fn wipe(painter: &egui::Painter, rect: Rect, t: f32, color: Color32) {
    let t = anim::ease_out_expo(t.clamp(0.0, 1.0));
    if t <= 0.002 {
        return;
    }
    let filled = Rect::from_min_max(rect.min, Pos2::new(rect.left() + rect.width() * t, rect.max.y));
    painter.rect_filled(filled, egui::Rounding::same(rect.height() * 0.5), color);
}

pub fn icon_button(ui: &mut egui::Ui, icon: Icon, tip: &str, size: f32) -> bool {
    icon_button_full(ui, icon, tip, size, None, None)
}

pub fn icon_button_tinted(
    ui: &mut egui::Ui,
    icon: Icon,
    tip: &str,
    size: f32,
    tint: Option<Color32>,
) -> bool {
    icon_button_full(ui, icon, tip, size, tint, None)
}

pub fn icon_button_full(
    ui: &mut egui::Ui,
    icon: Icon,
    tip: &str,
    size: f32,
    tint: Option<Color32>,
    id: Option<egui::Id>,
) -> bool {
    let p = pal();
    let (rect, resp) = match id {
        Some(id) => {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
            let resp = ui.interact(rect, id, Sense::click());
            (rect, resp)
        }
        None => ui.allocate_exact_size(Vec2::splat(size), Sense::click()),
    };
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let glyph = rect.shrink(size * 0.28);

    theme::hover_surface(ui.painter(), rect, size * 0.5, p.signal, hot, false);
    let ink = theme::mix(tint.unwrap_or(p.text_2), tint.unwrap_or(p.text), hot);
    icons::draw(ui.painter(), glyph, icon, ink);

    if tip.is_empty() {
        resp.clicked()
    } else {
        resp.on_hover_text(tip).clicked()
    }
}

pub fn button(ui: &mut egui::Ui, label: &str, icon: Option<Icon>, accent: bool) -> bool {
    button_sized(ui, label, icon, accent, None)
}

pub fn button_sized(
    ui: &mut egui::Ui,
    label: &str,
    icon: Option<Icon>,
    accent: bool,
    width: Option<f32>,
) -> bool {
    button_response(ui, label, icon, accent, width).clicked()
}

pub fn button_colors(accent: bool, hot: f32) -> (Color32, Color32, Color32) {
    let p = pal();
    if accent {
        (theme::mix(p.signal, p.signal_deep, hot), Color32::TRANSPARENT, p.ink)
    } else {
        (theme::mix(p.deep, p.hover_fill(), hot), theme::mix(p.line_strong, p.text_3, hot), p.text)
    }
}

pub const BUTTON_H: f32 = 32.0;

pub fn button_response(
    ui: &mut egui::Ui,
    label: &str,
    icon: Option<Icon>,
    accent: bool,
    width: Option<f32>,
) -> Response {
    let font = theme::font_bold(theme::T_SM);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font.clone(), Color32::PLACEHOLDER);
    let icon_w = if icon.is_some() { 20.0 } else { 0.0 };
    let w = width.unwrap_or(galley.rect.width() + icon_w + 32.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, BUTTON_H), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), crate::tokens::MOTION_QUICK);
    let painter = ui.painter();
    let (fill, edge, ink) = button_colors(accent, hot);
    let r = egui::Rounding::same(BUTTON_H * 0.5);
    painter.rect_filled(rect, r, fill);
    if edge.a() > 0 {
        painter.rect_stroke(rect.shrink(0.5), r, Stroke::new(1.0_f32, edge));
    }
    let mut x = rect.center().x - (galley.rect.width() + icon_w) * 0.5;
    if let Some(i) = icon {
        icons::draw(painter, Rect::from_center_size(Pos2::new(x + 7.0, rect.center().y), Vec2::splat(13.0)), i, ink);
        x += icon_w;
    }
    painter.galley(Pos2::new(x, rect.center().y - galley.rect.height() * 0.5), galley, ink);
    resp
}

pub fn quiet_button(ui: &mut egui::Ui, label: &str, icon: Option<Icon>) -> bool {
    let p = pal();
    let font = theme::font_med(theme::T_SM);
    let galley = ui.painter().layout_no_wrap(label.to_string(), font, p.text);
    let icon_w = if icon.is_some() { 20.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(galley.rect.width() + icon_w + 20.0, 28.0),
        Sense::click(),
    );
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), crate::tokens::MOTION_QUICK);
    if hot > 0.005 {
        ui.painter().rect_filled(rect, egui::Rounding::same(rect.height() * 0.5), theme::wash(p.hover_fill(), (255.0 * hot) as u8));
    }
    let mut x = rect.left() + 10.0;
    if let Some(i) = icon {
        icons::draw(
            ui.painter(),
            Rect::from_center_size(Pos2::new(x + 7.0, rect.center().y), Vec2::splat(13.0)),
            i,
            p.text,
        );
        x += icon_w;
    }
    ui.painter().galley(
        Pos2::new(x, rect.center().y - galley.rect.height() * 0.5),
        galley,
        p.text,
    );
    resp.clicked()
}

pub fn hud(ui: &mut egui::Ui, text: &str, color: Option<Color32>) -> Response {
    let p = pal();
    let col = color.unwrap_or(p.text_3);
    let font = theme::font_hud(theme::T_MICRO + 0.5);
    let w = theme::tracked_width(ui.ctx(), text, font.clone(), 0.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 16.0), Sense::hover());
    theme::tracked_text(ui.painter(), Pos2::new(rect.left(), rect.center().y), text, font, col, 0.0);
    resp
}

pub fn chip(ui: &mut egui::Ui, text: &str, tint: Option<Color32>) -> Response {
    hud(ui, text, tint)
}

pub fn chip_button(ui: &mut egui::Ui, text: &str) -> bool {
    let p = pal();
    let font = theme::font_hud(theme::T_MICRO + 0.5);
    let label = text.to_string();
    let w = theme::tracked_width(ui.ctx(), &label, font.clone(), 0.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w + 20.0, 22.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    ui.painter().rect_filled(rect, egui::Rounding::same(11.0), theme::mix(p.panel_2, p.line_strong, hot * 0.6));
    theme::tracked_text(
        ui.painter(),
        Pos2::new(rect.left() + 10.0, rect.center().y),
        &label,
        font,
        theme::mix(p.text_2, p.text, hot),
        0.8,
    );
    resp.clicked()
}

pub fn toggle(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(38.0, 22.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    let t = anim::ease(ui.ctx(), resp.id.with("on"), *on, 0.15);
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let r = egui::Rounding::same(rect.height() * 0.5);
    let off = theme::mix(p.line_strong, p.text_3, hot * 0.5);
    ui.painter().rect_filled(rect, r, theme::mix(off, p.signal, t));
    let side = rect.height() - 6.0;
    let c = Pos2::new(rect.left() + 3.0 + side * 0.5 + t * (rect.width() - 6.0 - side), rect.center().y);
    let knob = Rect::from_center_size(c, Vec2::splat(side));
    theme::shadow(ui.painter(), knob, side * 0.5, theme::Lift::Rest);
    ui.painter().circle_filled(c, side * 0.5, Color32::WHITE);
    resp.clicked()
}

pub fn toggle_row(ui: &mut egui::Ui, label: &str, hint: &str, on: &mut bool) -> bool {
    let p = pal();
    let mut flipped = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 48.0).max(40.0));
            ui.label(
                egui::RichText::new(label)
                    .font(theme::font_med(theme::T_SM))
                    .color(p.text),
            );
            if !hint.is_empty() {
                ui.label(
                    egui::RichText::new(hint)
                        .font(theme::font(theme::T_CAP))
                        .color(p.text_3),
                );
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            flipped = toggle(ui, on);
        });
    });
    ui.add_space(3.0);
    flipped
}

pub fn slider(ui: &mut egui::Ui, value: &mut f32, range: std::ops::RangeInclusive<f32>) -> bool {
    let p = pal();
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 22.0), Sense::click_and_drag());
    let (lo, hi) = (*range.start(), *range.end());
    let span = (hi - lo).max(0.0001);

    const KNOB: f32 = 8.0;
    let x0 = rect.left() + KNOB;
    let x1 = rect.right() - KNOB;

    let mut changed = false;
    if resp.dragged() || resp.clicked() {
        if let Some(q) = ui.ctx().pointer_interact_pos() {
            let t = ((q.x - x0) / (x1 - x0)).clamp(0.0, 1.0);
            let next = lo + t * span;
            if (next - *value).abs() > f32::EPSILON {
                *value = next;
                changed = true;
            }
        }
    }
    let t = ((*value - lo) / span).clamp(0.0, 1.0);
    let knob = Pos2::new(x0 + t * (x1 - x0), rect.center().y);

    let track = Rect::from_min_max(
        Pos2::new(rect.left(), rect.center().y - 2.0),
        Pos2::new(rect.right(), rect.center().y + 2.0),
    );
    ui.painter().rect_filled(track, egui::Rounding::same(2.0), p.line_strong);
    let filled = Rect::from_min_max(track.min, Pos2::new(knob.x, track.max.y));
    ui.painter().rect_filled(filled, egui::Rounding::same(2.0), p.signal);

    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered() || resp.dragged(), anim::HOVER);
    let side = KNOB * 2.0 + 2.0 * hot;
    let dot = Rect::from_center_size(knob, Vec2::splat(side));
    theme::shadow(ui.painter(), dot, side * 0.5, theme::Lift::Rest);
    ui.painter().circle_filled(knob, side * 0.5, Color32::WHITE);
    ui.painter().circle_stroke(knob, side * 0.5 - 0.5, Stroke::new(1.0_f32, p.line_strong));
    changed
}

pub fn link(ui: &mut egui::Ui, text: &str) -> bool {
    let p = pal();
    let font = theme::font_med(theme::T_SM);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, Color32::PLACEHOLDER);
    let (rect, resp) = ui.allocate_exact_size(galley.rect.size(), Sense::click());
    let hot = resp.hovered();
    let ink = if hot { p.text } else { p.signal_hi };
    ui.painter().galley(rect.min, galley, ink);
    if hot {
        let y = rect.bottom() - 0.5;
        ui.painter().line_segment([Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)], Stroke::new(1.0_f32, ink));
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn maker_line(ui: &mut egui::Ui, rect: Rect, centre: bool, app_name: &str) -> bool {
    let p = pal();
    let text = &format!("{app_name} by");
    let font = theme::font_hud(theme::T_MICRO);
    let logo_h = (theme::T_MICRO * 1.3).round();
    let text_w = theme::tracked_width(ui.ctx(), text, font.clone(), 0.8);
    let gap = 7.0;
    let width = text_w + gap + crate::mark::logo_width(logo_h);
    let left = if centre { rect.center().x - width * 0.5 } else { rect.left() + 3.0 };
    let hit = Rect::from_min_size(Pos2::new(left, rect.center().y - 8.0), Vec2::new(width, 16.0));
    let resp = ui.interact(hit, ui.id().with(("ncf-maker", rect.min.x as i32, rect.min.y as i32)), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let ink = theme::mix(p.text_3, p.text, hot);
    theme::tracked_text(ui.painter(), Pos2::new(left, rect.center().y), text, font, ink, 0.8);
    crate::mark::logo(ui.painter(), Pos2::new(left + text_w + gap, rect.center().y), logo_h, ink);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn logo_link(ui: &mut egui::Ui, height: f32, tint: Color32) -> bool {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(crate::mark::logo_width(height), height), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    crate::mark::logo(ui.painter(), rect.left_center(), height, theme::mix(tint, p.signal_hi, hot));
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.clicked()
}

pub fn open_site(ctx: &egui::Context) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = ctx;
        crate::platform::open(crate::tokens::BRAND_URL);
    }
    #[cfg(target_arch = "wasm32")]
    ctx.open_url(egui::OpenUrl::new_tab(crate::tokens::BRAND_URL));
}

pub fn logo(ui: &mut egui::Ui, height: f32, tint: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(crate::mark::logo_width(height), height), Sense::hover());
    crate::mark::logo(ui.painter(), rect.left_center(), height, tint);
    resp
}

pub fn vertical_tabs(ui: &mut egui::Ui, options: &[(&str, Icon)], current: usize) -> Option<usize> {
    let p = pal();
    ui.spacing_mut().item_spacing.y = 2.0;
    let mut picked = None;
    let (sel_fill, sel_ink) = p.selected();
    for (i, (label, icon)) in options.iter().enumerate() {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::click());
        let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
        let lit = anim::ease(ui.ctx(), resp.id.with("on"), i == current, crate::tokens::MOTION_QUICK);
        let r = egui::Rounding::same(theme::R_SM);
        if lit > 0.005 {
            ui.painter().rect_filled(rect, r, theme::wash(sel_fill, (255.0 * lit) as u8));
        } else if hot > 0.01 {
            ui.painter().rect_filled(rect, r, theme::wash(p.hover_fill(), (255.0 * hot) as u8));
        }
        let ink = theme::mix(theme::mix(p.text_2, p.text, hot), sel_ink, lit);
        icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 18.0, rect.center().y), Vec2::splat(14.0)), *icon, ink);
        ui.painter().text(Pos2::new(rect.left() + 34.0, rect.center().y), egui::Align2::LEFT_CENTER, label, theme::font_semi(theme::T_SM), ink);
        if resp.clicked() {
            picked = Some(i);
        }
    }
    picked
}

pub fn scrim(ctx: &egui::Context, id: &str, t: f32) -> bool {
    let screen = ctx.screen_rect();
    let mut clicked = false;
    egui::Area::new(egui::Id::new(id))
        .order(egui::Order::Middle)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click());
            let s = pal().scrim();
            ui.painter().rect_filled(rect, egui::Rounding::ZERO, theme::wash(s, (s.a() as f32 * t.clamp(0.0, 1.0)) as u8));
            clicked = resp.clicked();
        });
    clicked
}

// text roles here---

pub fn display_text(text: &str, size: f32, color: Color32) -> egui::RichText {
    egui::RichText::new(text).font(theme::font_display(size)).extra_letter_spacing(theme::display_tracking(size)).color(color)
}

pub fn giant(ui: &mut egui::Ui, text: &str, size: f32) -> Response {
    let p = pal();
    ui.add(egui::Label::new(display_text(text, size, p.text)).extend())
}

pub fn giant_fit(ui: &mut egui::Ui, text: &str, max: f32, min: f32, room: f32) -> Response {
    let p = pal();
    let full = theme::tracked_width(ui.ctx(), text, theme::font_display(max), theme::display_tracking(max));
    let size = if full <= room { max } else { (max * room / full * 0.97).max(min) };
    let shown = elide_to_width(ui.ctx(), text, theme::font_display(size), theme::display_tracking(size), room);
    ui.add(egui::Label::new(display_text(&shown, size, p.text)).extend())
}

pub fn heading(ui: &mut egui::Ui, text: &str) -> Response {
    let p = pal();
    ui.label(egui::RichText::new(text).font(theme::font_heading(theme::T_H)).extra_letter_spacing(theme::display_tracking(theme::T_H)).color(p.text))
}

pub fn big_number(ui: &mut egui::Ui, text: &str, size: f32, color: Color32) -> Response {
    ui.add(egui::Label::new(display_text(text, size, color)).extend())
}

pub const EYEBROW_SIZE: f32 = theme::T_CAP;

pub fn eyebrow_text(painter: &egui::Painter, left_centre: Pos2, text: &str, color: Color32) -> f32 {
    theme::tracked_text(painter, left_centre, &text.to_uppercase(), theme::font_eyebrow(EYEBROW_SIZE), color, theme::eyebrow_tracking(EYEBROW_SIZE))
}

pub fn eyebrow_width(ctx: &egui::Context, text: &str) -> f32 {
    theme::tracked_width(ctx, &text.to_uppercase(), theme::font_eyebrow(EYEBROW_SIZE), theme::eyebrow_tracking(EYEBROW_SIZE))
}

pub fn body(ui: &mut egui::Ui, text: &str) -> Response {
    let p = pal();
    ui.label(egui::RichText::new(text).font(theme::font(theme::T_SM)).color(p.text_2))
}

pub fn caption(ui: &mut egui::Ui, text: &str) -> Response {
    let p = pal();
    ui.label(egui::RichText::new(text).font(theme::font(theme::T_CAP)).color(p.text_3))
}

pub fn eyebrow(ui: &mut egui::Ui, text: &str) -> Response {
    let p = pal();
    let w = eyebrow_width(ui.ctx(), text);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 16.0), Sense::hover());
    eyebrow_text(ui.painter(), Pos2::new(rect.left(), rect.center().y), text, p.text_3);
    resp
}

// fields here---

pub fn field(ui: &mut egui::Ui, text: &mut String, hint: &str, width: f32) -> Response {
    ui.add(
        egui::TextEdit::singleline(text)
            .hint_text(hint)
            .desired_width(width)
            .margin(egui::Margin::symmetric(12.0, 8.0)),
    )
}

pub fn tabs(ui: &mut egui::Ui, labels: &[&str], current: usize) -> Option<usize> {
    let options: Vec<(&str, Icon)> = labels.iter().map(|l| (*l, Icon::Close)).collect();
    segmented(ui, &options, current)
}

pub fn section(ui: &mut egui::Ui, label: &str) {
    let p = pal();
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 14.0), Sense::hover());
    eyebrow_text(ui.painter(), Pos2::new(rect.left(), rect.center().y), label, p.text_3);
    ui.add_space(4.0);
}

pub fn separator(ui: &mut egui::Ui) {
    let p = pal();
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 13.0), Sense::hover());
    let y = rect.center().y.round() + 0.5;
    ui.painter().line_segment(
        [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
        Stroke::new(1.0_f32, p.line),
    );
}

// ribbon here---

pub fn ribbon_button(ui: &mut egui::Ui, icon: Icon, label: &str, tip: &str, on: bool) -> bool {
    let p = pal();
    let galley = if label.is_empty() {
        None
    } else {
        Some(ui.painter().layout_no_wrap(
            label.to_string(),
            theme::font_med(theme::T_LABEL),
            Color32::PLACEHOLDER,
        ))
    };
    let w = 30.0 + galley.as_ref().map(|g| g.rect.width() + 10.0).unwrap_or(0.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let lit = anim::ease(ui.ctx(), resp.id.with("on"), on, crate::tokens::MOTION_QUICK);
    let (sel_fill, sel_ink) = p.selected();
    let r = egui::Rounding::same(rect.height() * 0.5);
    if lit > 0.005 {
        ui.painter().rect_filled(rect, r, theme::wash(sel_fill, (255.0 * lit) as u8));
    } else if hot > 0.005 {
        ui.painter().rect_filled(rect, r, theme::wash(p.hover_fill(), (255.0 * hot) as u8));
    }
    let ink = theme::mix(theme::mix(p.text_2, p.text, hot), sel_ink, lit);
    let glyph = Rect::from_center_size(Pos2::new(rect.left() + 15.0, rect.center().y), Vec2::splat(14.0));
    icons::draw(ui.painter(), glyph, icon, ink);
    if let Some(g) = galley {
        ui.painter().galley(
            Pos2::new(rect.left() + 28.0, rect.center().y - g.rect.height() * 0.5),
            g,
            ink,
        );
    }
    if tip.is_empty() {
        resp.clicked()
    } else {
        resp.on_hover_text(tip).clicked()
    }
}

pub fn segmented(ui: &mut egui::Ui, options: &[(&str, Icon)], current: usize) -> Option<usize> {
    let p = pal();
    let font = theme::font_semi(theme::T_SM);
    let widths: Vec<f32> = options
        .iter()
        .map(|(l, _)| ui.painter().layout_no_wrap((*l).to_string(), font.clone(), p.text).rect.width() + 30.0)
        .collect();
    let seg_w = widths.iter().cloned().fold(64.0_f32, f32::max);
    let h = 34.0;
    let pad = 3.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(seg_w * options.len() as f32 + pad * 2.0, h), Sense::hover());
    ui.painter().rect_filled(rect, egui::Rounding::same(h * 0.5), p.panel_2);
    let (sel_fill, sel_ink) = p.selected();
    let mut picked = None;
    for (i, (label, _)) in options.iter().enumerate() {
        let cell = Rect::from_min_size(Pos2::new(rect.left() + pad + i as f32 * seg_w, rect.top() + pad), Vec2::new(seg_w, h - pad * 2.0));
        let resp = ui.interact(cell, ui.id().with(("seg", i)), Sense::click());
        let lit = anim::ease(ui.ctx(), resp.id.with("on"), i == current, 0.15);
        if lit > 0.005 {
            ui.painter().rect_filled(cell, egui::Rounding::same(cell.height() * 0.5), theme::wash(sel_fill, (255.0 * lit) as u8));
        }
        let rest = if resp.hovered() { p.text } else { p.text_2 };
        let ink = theme::mix(rest, sel_ink, lit);
        ui.painter().text(cell.center(), egui::Align2::CENTER_CENTER, label, font.clone(), ink);
        if resp.clicked() {
            picked = Some(i);
        }
    }
    picked
}

pub fn pill_bar<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> egui::InnerResponse<R> {
    theme::glass_frame(theme::R_PILL)
        .inner_margin(egui::Margin::symmetric(6.0, 4.0))
        .show(ui, |ui| ui.horizontal(|ui| add(ui)).inner)
}

// popups here---

pub fn popover_frame() -> egui::Frame {
    let p = pal();
    egui::Frame::none()
        .fill(p.deep)
        .stroke(if p.dark { Stroke::new(1.0_f32, p.line) } else { Stroke::NONE })
        .rounding(egui::Rounding::same(theme::R_MD * 0.8))
        .shadow(theme::shadow_of(theme::Lift::Float))
        .inner_margin(egui::Margin::symmetric(6.0, 6.0))
}

pub fn menu_width(ui: &egui::Ui, labels: &[&str]) -> f32 {
    let widest = labels
        .iter()
        .map(|l| {
            ui.painter()
                .layout_no_wrap((*l).to_string(), theme::font(theme::T_SM), Color32::WHITE)
                .rect
                .width()
        })
        .fold(0.0_f32, f32::max);
    (widest + 44.0).clamp(112.0, 236.0)
}

pub fn menu_item(
    ui: &mut egui::Ui,
    rects: &mut Vec<Rect>,
    label: &str,
    icon: Icon,
    critical: bool,
) -> bool {
    menu_item_enabled(ui, rects, label, icon, critical, true)
}

pub fn menu_item_enabled(
    ui: &mut egui::Ui,
    rects: &mut Vec<Rect>,
    label: &str,
    icon: Icon,
    critical: bool,
    enabled: bool,
) -> bool {
    let p = pal();
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(
        Vec2::new(w, 28.0),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    rects.push(rect);
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered() && enabled, anim::HOVER);
    if hot > 0.01 {
        let fill = if critical { theme::wash(p.danger, (36.0 * hot) as u8) } else { theme::wash(p.hover_fill(), (255.0 * hot) as u8) };
        ui.painter().rect_filled(rect, egui::Rounding::same(8.0), fill);
    }
    let base = if !enabled {
        theme::wash(p.text_3, 110)
    } else if critical {
        p.danger
    } else {
        p.text_2
    };
    let color = theme::mix(base, if critical { p.danger } else { p.text }, hot);
    icons::draw(
        ui.painter(),
        Rect::from_center_size(Pos2::new(rect.left() + 15.0, rect.center().y), Vec2::splat(13.0)),
        icon,
        color,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 29.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        theme::font(theme::T_SM),
        color,
    );
    enabled && resp.clicked()
}

pub fn menu_logo_item(ui: &mut egui::Ui, rects: &mut Vec<Rect>) -> bool {
    let p = pal();
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 28.0), Sense::click());
    rects.push(rect);
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    if hot > 0.01 {
        ui.painter().rect_filled(rect, egui::Rounding::same(8.0), theme::wash(p.hover_fill(), (255.0 * hot) as u8));
    }
    let color = theme::mix(p.text_2, p.text, hot);
    icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 15.0, rect.center().y), Vec2::splat(13.0)), Icon::External, color);
    crate::mark::logo(ui.painter(), Pos2::new(rect.left() + 29.0, rect.center().y), 13.0, color);
    resp.clicked()
}

pub fn dropdown(ui: &mut egui::Ui, icon: Icon, label: &str, open: bool, width: f32) -> Response {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered() || open, anim::HOVER);
    theme::hover_surface(ui.painter(), rect, rect.height() * 0.5, p.signal, hot, false);
    let ink = theme::mix(p.text_2, p.text, hot);
    icons::draw(
        ui.painter(),
        Rect::from_center_size(Pos2::new(rect.left() + 15.0, rect.center().y), Vec2::splat(13.0)),
        icon,
        ink,
    );
    ui.painter().text(
        Pos2::new(rect.left() + 29.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        elide(label, ((width - 54.0) / 6.4) as usize),
        theme::font_med(theme::T_LABEL),
        ink,
    );
    let turn = anim::ease(ui.ctx(), resp.id.with("turn"), open, 0.18);
    icons::draw(
        ui.painter(),
        Rect::from_center_size(Pos2::new(rect.right() - 13.0, rect.center().y), Vec2::splat(11.0)),
        if turn > 0.5 { Icon::ChevronUp } else { Icon::ChevronDown },
        p.text_3,
    );
    resp
}

pub struct Dismisser {
    pub opened_on: u64,
}

impl Dismisser {
    pub fn should_close(&self, ctx: &egui::Context, frame_no: u64, areas: &[Rect]) -> bool {
        if frame_no <= self.opened_on + 1 {
            return false;
        }
        if !ctx.input(|i| i.pointer.any_click()) {
            return false;
        }
        ctx.pointer_interact_pos()
            .map(|p| !areas.iter().any(|r| r.expand(4.0).contains(p)))
            .unwrap_or(false)
    }
}

pub fn elide(s: &str, max: usize) -> String {
    let max = max.max(4);
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let head: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{head}\u{2026}")
    }
}

pub fn elide_to_width(ctx: &egui::Context, text: &str, font: egui::FontId, tracking: f32, max_w: f32) -> String {
    let fits = |s: &str| theme::tracked_width(ctx, s, font.clone(), tracking) <= max_w;
    if fits(text) {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let cut = |n: usize| -> String {
        let head: String = chars[..n].iter().collect();
        format!("{}\u{2026}", head.trim_end())
    };
    let (mut lo, mut hi) = (0usize, chars.len());
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        if fits(&cut(mid)) {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    if lo == 0 && !fits("\u{2026}") {
        return String::new();
    }
    cut(lo)
}

fn text_width(ui: &egui::Ui, s: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(s.to_string(), theme::font_mono(theme::T_CAP), Color32::WHITE)
        .rect
        .width()
}

pub fn fit_text(ui: &egui::Ui, s: &str, room: f32) -> String {
    if room <= 20.0 {
        return String::new();
    }
    if text_width(ui, s) <= room {
        return s.to_string();
    }
    let mut words: Vec<&str> = s.split(' ').collect();
    while words.len() > 1 {
        words.pop();
        let candidate = format!("{}\u{2026}", words.join(" "));
        if text_width(ui, &candidate) <= room {
            return candidate;
        }
    }
    String::new()
}

pub fn pick_that_fits(ui: &egui::Ui, options: &[&str], room: f32) -> String {
    for o in options {
        if text_width(ui, o) <= room {
            return (*o).to_string();
        }
    }
    String::new()
}

pub fn swatch(
    ui: &mut egui::Ui,
    color: Color32,
    tip: &str,
    selected: bool,
    id: Option<egui::Id>,
) -> bool {
    let (rect, resp) = match id {
        Some(id) => {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(19.0), Sense::hover());
            let resp = ui.interact(rect, id, Sense::click());
            (rect, resp)
        }
        None => ui.allocate_exact_size(Vec2::splat(19.0), Sense::click()),
    };
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    ui.painter().circle_filled(rect.center(), 6.0 + hot, color);
    if selected {
        ui.painter().circle_stroke(rect.center(), 8.5, Stroke::new(1.5_f32, pal().focus));
    }
    resp.on_hover_text(tip).clicked()
}

// concept pieces here---

pub fn tag_size(ctx: &egui::Context, text: &str, dot: bool) -> Vec2 {
    let w = theme::tracked_width(ctx, text, theme::font_hud(theme::T_MICRO), 0.0);
    Vec2::new(w + 16.0 + if dot { 10.0 } else { 0.0 }, 22.0)
}

pub fn tag_at(painter: &egui::Painter, rect: Rect, text: &str, tone: Option<usize>, dot: bool) {
    let p = pal();
    let (ink, fill) = p.tag(tone);
    painter.rect_filled(rect, egui::Rounding::same(rect.height() * 0.5), fill);
    let mut x = rect.left() + 8.0;
    if dot {
        painter.circle_filled(Pos2::new(x + 3.0, rect.center().y), 3.0, ink);
        x += 10.0;
    }
    theme::tracked_text(painter, Pos2::new(x, rect.center().y), text, theme::font_hud(theme::T_MICRO), ink, 0.0);
}

pub fn tag(ui: &mut egui::Ui, text: &str, tone: Option<usize>) -> Response {
    let (rect, resp) = ui.allocate_exact_size(tag_size(ui.ctx(), text, false), Sense::hover());
    tag_at(ui.painter(), rect, text, tone, false);
    resp
}

pub fn live_tag(ui: &mut egui::Ui, text: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(tag_size(ui.ctx(), text, true), Sense::hover());
    tag_at(ui.painter(), rect, text, Some(4), true);
    resp
}

pub const SIDE_ROW_H: f32 = 38.0;

pub fn side_link(ui: &mut egui::Ui, icon: Icon, label: &str, trailing: Option<&str>, on: bool) -> Response {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), SIDE_ROW_H), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let lit = anim::ease(ui.ctx(), resp.id.with("on"), on, crate::tokens::MOTION_QUICK);
    let round = egui::Rounding::same(theme::R_SM);
    let (sel_fill, sel_ink) = p.selected();
    if lit > 0.005 {
        ui.painter().rect_filled(rect, round, theme::wash(sel_fill, (255.0 * lit) as u8));
    } else if hot > 0.01 {
        ui.painter().rect_filled(rect, round, theme::wash(p.pill(), (p.pill().a() as f32 * hot * 1.6).min(255.0) as u8));
    }
    let ink = theme::mix(theme::mix(p.text_2, p.text, hot), sel_ink, lit);
    icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 20.0, rect.center().y), Vec2::splat(16.0)), icon, ink);
    let font = theme::font_med(theme::T_SM + 0.5);
    let room = rect.width() - 46.0 - if trailing.is_some() { 44.0 } else { 8.0 };
    ui.painter().text(Pos2::new(rect.left() + 40.0, rect.center().y), egui::Align2::LEFT_CENTER, elide_to_width(ui.ctx(), label, font.clone(), 0.0, room), font, ink);
    if let Some(t) = trailing {
        let soft = theme::mix(p.text_3, theme::wash(sel_ink, 170), lit);
        ui.painter().text(Pos2::new(rect.right() - 12.0, rect.center().y), egui::Align2::RIGHT_CENTER, t, theme::font_hud(theme::T_MICRO), soft);
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn side_section(ui: &mut egui::Ui, label: &str) {
    ui.add_space(10.0);
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        eyebrow(ui, label);
    });
    ui.add_space(4.0);
}

pub fn search_trigger(ui: &mut egui::Ui, width: f32, hint: &str, chord: &str) -> Response {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 32.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let r = rect.height() * 0.5;
    ui.painter().rect_filled(rect, egui::Rounding::same(r), theme::mix(p.glass, p.pill(), hot));
    ui.painter().rect_stroke(rect.shrink(0.5), egui::Rounding::same(r), Stroke::new(1.0_f32, p.glass_line));
    theme::glass_highlight(ui.painter(), rect, r);
    icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 20.0, rect.center().y), Vec2::splat(14.0)), Icon::Search, p.text_2);
    ui.painter().text(Pos2::new(rect.left() + 36.0, rect.center().y), egui::Align2::LEFT_CENTER, hint, theme::font(theme::T_SM), p.text_3);
    if !chord.is_empty() {
        let kw = theme::tracked_width(ui.ctx(), chord, theme::font_hud(theme::T_MICRO - 0.5), 0.0) + 12.0;
        let k = Rect::from_center_size(Pos2::new(rect.right() - 8.0 - kw * 0.5, rect.center().y), Vec2::new(kw, 20.0));
        ui.painter().rect_filled(k, egui::Rounding::same(6.0), p.pill());
        ui.painter().rect_stroke(k.shrink(0.5), egui::Rounding::same(6.0), Stroke::new(1.0_f32, p.line));
        ui.painter().text(k.center(), egui::Align2::CENTER_CENTER, chord, theme::font_hud(theme::T_MICRO - 0.5), p.text_2);
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn stat_chip(ui: &mut egui::Ui, value: &str, unit: &str) -> Response {
    let p = pal();
    let bold = theme::font_hud(theme::T_SM - 1.5);
    let soft = theme::font(theme::T_SM);
    let vw = theme::tracked_width(ui.ctx(), value, bold.clone(), 0.0);
    let uw = theme::tracked_width(ui.ctx(), unit, soft.clone(), 0.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(vw + uw + 26.0, 28.0), Sense::hover());
    ui.painter().rect_filled(rect, egui::Rounding::same(14.0), p.pill());
    ui.painter().text(Pos2::new(rect.left() + 11.0, rect.center().y), egui::Align2::LEFT_CENTER, value, bold, p.text);
    ui.painter().text(Pos2::new(rect.left() + 15.0 + vw, rect.center().y), egui::Align2::LEFT_CENTER, unit, soft, p.text_2);
    resp
}

pub struct SegTab<'a> {
    pub label: &'a str,
    pub icon: Option<Icon>,
    pub count: Option<usize>,
    pub dot: bool,
}

pub fn seg_tabs(ui: &mut egui::Ui, tabs: &[SegTab], current: usize, width: Option<f32>) -> Option<usize> {
    let p = pal();
    let font = theme::font_semi(theme::T_SM);
    let extra = |t: &SegTab| -> f32 {
        (if t.icon.is_some() { 22.0 } else { 0.0 }) + if t.count.is_some() { 26.0 } else if t.dot { 12.0 } else { 0.0 }
    };
    let content: Vec<f32> = tabs.iter().map(|t| theme::tracked_width(ui.ctx(), t.label, font.clone(), 0.0) + extra(t)).collect();
    let pad = 4.0;
    let h = 38.0;
    let snug: f32 = content.iter().sum::<f32>() + 14.0 * tabs.len() as f32 + pad * 2.0;
    let natural_total: f32 = content.iter().sum::<f32>() + 30.0 * tabs.len() as f32 + pad * 2.0;
    let total = width.unwrap_or(natural_total).max(snug);
    let spare = (total - pad * 2.0 - content.iter().sum::<f32>()) / tabs.len().max(1) as f32;
    let widths: Vec<f32> = content.iter().map(|c| c + spare).collect();
    let (rect, _) = ui.allocate_exact_size(Vec2::new(total, h), Sense::hover());
    let r = h * 0.5;
    ui.painter().rect_filled(rect, egui::Rounding::same(r), p.glass);
    ui.painter().rect_stroke(rect.shrink(0.5), egui::Rounding::same(r), Stroke::new(1.0_f32, p.glass_line));
    theme::glass_highlight(ui.painter(), rect, r);
    let (sel_fill, sel_ink) = p.selected();
    let mut x = rect.left() + pad;
    let mut picked = None;
    for (i, t) in tabs.iter().enumerate() {
        let w = widths[i];
        let cell = Rect::from_min_size(Pos2::new(x, rect.top() + pad), Vec2::new(w, h - pad * 2.0));
        x += w;
        let resp = ui.interact(cell, ui.id().with(("ncf-segtab", i)), Sense::click());
        let lit = anim::ease(ui.ctx(), resp.id.with("on"), i == current, 0.15);
        if lit > 0.005 {
            ui.painter().rect_filled(cell, egui::Rounding::same(cell.height() * 0.5), theme::wash(sel_fill, (255.0 * lit) as u8));
        }
        let rest = if resp.hovered() { p.text } else { p.text_2 };
        let ink = theme::mix(rest, sel_ink, lit);
        let mut cx = cell.center().x - content[i] * 0.5;
        if let Some(icon) = t.icon {
            icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(cx + 7.0, cell.center().y), Vec2::splat(14.0)), icon, ink);
            cx += 22.0;
        }
        let lw = theme::tracked_text(ui.painter(), Pos2::new(cx, cell.center().y), t.label, font.clone(), ink, 0.0);
        cx += lw + 6.0;
        if let Some(n) = t.count {
            let b = Rect::from_center_size(Pos2::new(cx + 9.0, cell.center().y), Vec2::new(18.0, 18.0));
            ui.painter().circle_filled(b.center(), 9.0, p.signal);
            ui.painter().text(b.center(), egui::Align2::CENTER_CENTER, n.min(99).to_string(), theme::font_hud(theme::T_MICRO - 1.5), p.ink);
        } else if t.dot {
            ui.painter().circle_filled(Pos2::new(cx + 3.0, cell.center().y), 3.5, p.signal);
        }
        if resp.clicked() {
            picked = Some(i);
        }
    }
    picked
}

pub fn select_button(ui: &mut egui::Ui, label: &str, value: &str) -> Response {
    let p = pal();
    let soft = theme::font(theme::T_SM);
    let strong = theme::font_semi(theme::T_SM);
    let lw = theme::tracked_width(ui.ctx(), label, soft.clone(), 0.0);
    let vw = theme::tracked_width(ui.ctx(), value, strong.clone(), 0.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(lw + vw + 54.0, BUTTON_H), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let r = egui::Rounding::same(BUTTON_H * 0.5);
    ui.painter().rect_filled(rect, r, theme::mix(p.pill(), p.hover_fill(), hot * 0.5));
    ui.painter().rect_stroke(rect.shrink(0.5), r, Stroke::new(1.0_f32, p.glass_line));
    ui.painter().text(Pos2::new(rect.left() + 14.0, rect.center().y), egui::Align2::LEFT_CENTER, label, soft, p.text_3);
    ui.painter().text(Pos2::new(rect.left() + 20.0 + lw, rect.center().y), egui::Align2::LEFT_CENTER, value, strong, p.text);
    icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.right() - 18.0, rect.center().y), Vec2::splat(13.0)), Icon::ChevronDown, p.text_2);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn empty_state(ui: &mut egui::Ui, icon: Icon, title: &str, body: &str, action: Option<&str>) -> bool {
    let p = pal();
    let w = ui.available_width();
    let title_g = ui.painter().layout(title.to_string(), theme::font_bold(theme::T_BODY), p.text, w - 48.0);
    let body_g = ui.painter().layout(body.to_string(), theme::font(theme::T_SM), p.text_2, (w - 48.0).min(440.0));
    let h = 34.0 + 44.0 + 12.0 + title_g.rect.height() + 6.0 + body_g.rect.height() + if action.is_some() { 48.0 } else { 0.0 } + 30.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(w, h), Sense::hover());
    let shape = egui::Shape::dashed_line(&dashed_round(rect.shrink(0.5), theme::R_MD), Stroke::new(1.0_f32, p.line_strong), 6.0, 5.0);
    ui.painter().extend(shape);
    let mut y = rect.top() + 34.0;
    let chip = Rect::from_center_size(Pos2::new(rect.center().x, y + 20.0), Vec2::splat(40.0));
    ui.painter().rect_filled(chip, egui::Rounding::same(12.0), p.signal_soft_hi);
    icons::draw(ui.painter(), chip.shrink(11.0), icon, p.signal_hi);
    y += 44.0 + 12.0;
    let th = title_g.rect.height();
    ui.painter().galley(Pos2::new(rect.center().x - title_g.rect.width() * 0.5, y), title_g, p.text);
    y += th + 6.0;
    let bh = body_g.rect.height();
    ui.painter().galley(Pos2::new(rect.center().x - body_g.rect.width() * 0.5, y), body_g, p.text_2);
    y += bh + 14.0;
    let mut clicked = false;
    if let Some(a) = action {
        let font = theme::font_bold(theme::T_SM);
        let aw = theme::tracked_width(ui.ctx(), a, font.clone(), 0.0) + 28.0;
        let b = Rect::from_min_size(Pos2::new(rect.center().x - aw * 0.5, y), Vec2::new(aw, 30.0));
        let resp = ui.interact(b, ui.id().with(("ncf-empty", title)), Sense::click());
        let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
        let (fill, _, ink) = button_colors(true, hot);
        ui.painter().rect_filled(b, egui::Rounding::same(15.0), fill);
        ui.painter().text(b.center(), egui::Align2::CENTER_CENTER, a, font, ink);
        clicked = resp.clicked();
    }
    clicked
}

fn dashed_round(rect: Rect, radius: f32) -> Vec<Pos2> {
    let r = radius.min(rect.width() * 0.5).min(rect.height() * 0.5);
    let mut pts = Vec::new();
    let corners = [
        (Pos2::new(rect.right() - r, rect.top() + r), -90.0_f32),
        (Pos2::new(rect.right() - r, rect.bottom() - r), 0.0),
        (Pos2::new(rect.left() + r, rect.bottom() - r), 90.0),
        (Pos2::new(rect.left() + r, rect.top() + r), 180.0),
    ];
    for (c, start) in corners {
        for k in 0..=8 {
            let a = (start + 90.0 * k as f32 / 8.0).to_radians();
            pts.push(Pos2::new(c.x + a.cos() * r, c.y + a.sin() * r));
        }
    }
    pts.push(pts[0]);
    pts
}

