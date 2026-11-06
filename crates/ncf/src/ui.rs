#![allow(dead_code)]

use eframe::egui::{self, Align, Color32, Layout, Pos2, Rect, Response, Sense, Stroke, Vec2};

use crate::anim;
use crate::icons::{self, Icon};
use crate::theme::{self, pal};

// pieces here---

pub fn ghost_slot(ui: &egui::Ui, rect: Rect, hot: bool, on: bool) {
    let p = pal();
    let t = anim::ease(ui.ctx(), rect.min.x as i32 * 7919 + rect.min.y as i32, hot, anim::HOVER);
    theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, t, on);
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
    painter.rect_filled(filled, egui::Rounding::same(theme::RADIUS), color);
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

    theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, false);
    let ink = theme::mix(tint.unwrap_or(p.text_2), tint.map(|t| theme::lighten(t, 0.2)).unwrap_or(p.text), hot);
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

pub fn button_response(
    ui: &mut egui::Ui,
    label: &str,
    icon: Option<Icon>,
    accent: bool,
    width: Option<f32>,
) -> Response {
    let p = pal();
    let font = theme::font_semi(theme::T_SM);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font.clone(), p.text);
    let icon_w = if icon.is_some() { 20.0 } else { 0.0 };
    let w = width.unwrap_or(galley.rect.width() + icon_w + 28.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), crate::tokens::MOTION_SLOW);
    let painter = ui.painter();

    let (rest_ink, fill_col, lit_ink) = if accent {
        (Color32::WHITE, Color32::WHITE, Color32::BLACK)
    } else {
        (p.text, p.text, p.void)
    };
    if accent {
        painter.rect_filled(rect, egui::Rounding::same(theme::RADIUS), p.signal);
    } else {
        painter.rect_stroke(
            rect.shrink(0.5),
            egui::Rounding::same(theme::RADIUS),
            Stroke::new(1.0_f32, theme::mix(p.line_strong, p.text, hot)),
        );
    }
    wipe(painter, rect, hot, fill_col);

    let reach = rect.left() + rect.width() * anim::ease_out_expo(hot.clamp(0.0, 1.0));
    let start = rect.center().x - (galley.rect.width() + icon_w) * 0.5;
    let draw = |painter: &egui::Painter, ink: Color32| {
        let mut x = start;
        if let Some(i) = icon {
            icons::draw(
                painter,
                Rect::from_center_size(Pos2::new(x + 7.0, rect.center().y), Vec2::splat(13.0)),
                i,
                ink,
            );
            x += icon_w;
        }
        painter.text(
            Pos2::new(x, rect.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            font.clone(),
            ink,
        );
    };
    let clip = painter.clip_rect();
    draw(
        &painter.with_clip_rect(clip.intersect(Rect::from_min_max(Pos2::new(reach, rect.top()), rect.max))),
        rest_ink,
    );
    if reach > rect.left() + 0.5 {
        draw(
            &painter.with_clip_rect(clip.intersect(Rect::from_min_max(rect.min, Pos2::new(reach, rect.bottom())))),
            lit_ink,
        );
    }
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
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), crate::tokens::MOTION_SLOW);
    wipe(ui.painter(), rect, hot, if p.dark { p.panel_2 } else { p.sunken });
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
    let w = theme::tracked_width(ui.ctx(), &text.to_uppercase(), font.clone(), 0.8);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 16.0), Sense::hover());
    theme::tracked_text(
        ui.painter(),
        Pos2::new(rect.left(), rect.center().y),
        &text.to_uppercase(),
        font,
        col,
        0.8,
    );
    resp
}

pub fn chip(ui: &mut egui::Ui, text: &str, tint: Option<Color32>) -> Response {
    hud(ui, text, tint)
}

pub fn chip_button(ui: &mut egui::Ui, text: &str) -> bool {
    let p = pal();
    let font = theme::font_hud(theme::T_MICRO + 0.5);
    let label = text.to_uppercase();
    let w = theme::tracked_width(ui.ctx(), &label, font.clone(), 0.8);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w + 16.0, 20.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        egui::Rounding::same(theme::RADIUS),
        Stroke::new(1.0_f32, theme::mix(p.line_strong, p.text, hot)),
    );
    theme::tracked_text(
        ui.painter(),
        Pos2::new(rect.left() + 8.0, rect.center().y),
        &label,
        font,
        theme::mix(p.text_2, p.text, hot),
        0.8,
    );
    resp.clicked()
}

pub fn toggle(ui: &mut egui::Ui, on: &mut bool) -> bool {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(36.0, 20.0), Sense::click());
    if resp.clicked() {
        *on = !*on;
    }
    let t = anim::ease(ui.ctx(), resp.id.with("on"), *on, 0.15);
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let r = egui::Rounding::same(theme::RADIUS);
    ui.painter()
        .rect_filled(rect, r, theme::wash(p.signal, (255.0 * t) as u8));
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        r,
        Stroke::new(
            1.0_f32,
            theme::mix(theme::mix(p.line_strong, p.text_3, hot), p.signal, t),
        ),
    );
    let side = rect.height() - 8.0;
    let knob = Rect::from_min_size(
        Pos2::new(rect.left() + 4.0 + t * (rect.width() - 8.0 - side), rect.top() + 4.0),
        Vec2::splat(side),
    );
    ui.painter()
        .rect_filled(knob, r, theme::mix(p.text_2, Color32::WHITE, t));
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

    const KNOB: f32 = 7.0;
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
        Pos2::new(rect.left(), rect.center().y - 1.0),
        Pos2::new(rect.right(), rect.center().y + 1.0),
    );
    ui.painter().rect_filled(track, egui::Rounding::ZERO, p.line_strong);
    let filled = Rect::from_min_max(track.min, Pos2::new(knob.x, track.max.y));
    ui.painter().rect_filled(filled, egui::Rounding::ZERO, p.signal_hi);

    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered() || resp.dragged(), anim::HOVER);
    let side = KNOB * 2.0 + 2.0 * hot;
    ui.painter().rect_filled(
        Rect::from_center_size(knob, Vec2::splat(side)),
        egui::Rounding::ZERO,
        p.text,
    );
    changed
}

pub fn link(ui: &mut egui::Ui, text: &str) -> bool {
    let p = pal();
    let font = theme::font_med(theme::T_SM);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, p.signal_hi);
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
    let text = &format!("{} BY", app_name.to_uppercase());
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
    resp.on_hover_text(crate::tokens::BRAND_URL).clicked()
}

pub fn logo(ui: &mut egui::Ui, height: f32, tint: Color32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(crate::mark::logo_width(height), height), Sense::hover());
    crate::mark::logo(ui.painter(), rect.left_center(), height, tint);
    resp
}

pub fn vertical_tabs(ui: &mut egui::Ui, options: &[(&str, Icon)], current: usize) -> Option<usize> {
    let p = pal();
    ui.spacing_mut().item_spacing.y = 0.0;
    let mut picked = None;
    let mut rows = Vec::new();
    for (i, (label, icon)) in options.iter().enumerate() {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 36.0), Sense::click());
        let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
        let lit = anim::ease(ui.ctx(), resp.id.with("on"), i == current, crate::tokens::MOTION_QUICK);
        if lit > 0.005 {
            ui.painter().rect_filled(rect, egui::Rounding::ZERO, theme::wash(p.text, (255.0 * lit) as u8));
        } else if hot > 0.01 {
            ui.painter().rect_filled(rect, egui::Rounding::ZERO, theme::wash(p.hover_fill(), (255.0 * hot) as u8));
        }
        let ink = theme::mix(theme::mix(p.text_2, p.text, hot), p.void, lit);
        icons::draw(ui.painter(), Rect::from_center_size(Pos2::new(rect.left() + 18.0, rect.center().y), Vec2::splat(13.0)), *icon, ink);
        theme::tracked_text(ui.painter(), Pos2::new(rect.left() + 34.0, rect.center().y), &label.to_uppercase(), theme::font_hud(theme::T_CAP), ink, 0.9);
        if resp.clicked() {
            picked = Some(i);
        }
        rows.push(rect);
    }
    if let (Some(first), Some(last)) = (rows.first(), rows.last()) {
        for r in &rows[1..] {
            let y = r.top().round() + 0.5;
            ui.painter().line_segment([Pos2::new(r.left(), y), Pos2::new(r.right(), y)], Stroke::new(1.0_f32, p.line));
        }
        ui.painter().rect_stroke(Rect::from_min_max(first.min, last.max).shrink(0.5), egui::Rounding::ZERO, Stroke::new(1.0_f32, p.line_strong));
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
            ui.painter().rect_filled(rect, egui::Rounding::ZERO, Color32::from_black_alpha((178.0 * t) as u8));
            clicked = resp.clicked();
        });
    clicked
}

// text roles here---

pub fn giant(ui: &mut egui::Ui, text: &str, size: f32) -> Response {
    let p = pal();
    ui.add(egui::Label::new(egui::RichText::new(text.to_uppercase()).font(theme::font_display(size)).color(p.text)).extend())
}

pub fn giant_fit(ui: &mut egui::Ui, text: &str, max: f32, min: f32, room: f32) -> Response {
    let p = pal();
    let shout = text.to_uppercase();
    let full = theme::tracked_width(ui.ctx(), &shout, theme::font_display(max), 0.0);
    let size = if full <= room { max } else { (max * room / full * 0.97).max(min) };
    let shown = elide_to_width(ui.ctx(), &shout, theme::font_display(size), 0.0, room);
    ui.add(egui::Label::new(egui::RichText::new(shown).font(theme::font_display(size)).color(p.text)).extend())
}

pub fn heading(ui: &mut egui::Ui, text: &str) -> Response {
    let p = pal();
    ui.label(egui::RichText::new(text).font(theme::font_bold(theme::T_H)).color(p.text))
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
    hud(ui, text, None)
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
    theme::tracked_text(
        ui.painter(),
        Pos2::new(rect.left(), rect.center().y),
        &label.to_uppercase(),
        theme::font_hud(theme::T_MICRO),
        p.text_3,
        0.8,
    );
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
            p.text_2,
        ))
    };
    let w = 30.0 + galley.as_ref().map(|g| g.rect.width() + 6.0).unwrap_or(0.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 30.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, on);
    let ink = if on {
        p.signal_hi
    } else {
        theme::mix(p.text_2, p.text, hot)
    };
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
    let font = theme::font_hud(theme::T_CAP);
    let labels: Vec<String> = options.iter().map(|(l, _)| l.to_uppercase()).collect();
    let widths: Vec<f32> = labels
        .iter()
        .map(|l| theme::tracked_width(ui.ctx(), l, font.clone(), 0.9) + 32.0)
        .collect();
    let seg_w = widths.iter().cloned().fold(72.0_f32, f32::max);
    let h = 32.0;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(seg_w * options.len() as f32, h),
        Sense::hover(),
    );

    let mut picked = None;
    for (i, label) in labels.iter().enumerate() {
        let cell = Rect::from_min_size(
            Pos2::new(rect.left() + i as f32 * seg_w, rect.top()),
            Vec2::new(seg_w, h),
        );
        let resp = ui.interact(cell, ui.id().with(("seg", i)), Sense::click());
        let on = i == current;
        let lit = anim::ease(ui.ctx(), resp.id.with("on"), on, 0.15);
        if lit > 0.005 {
            ui.painter().rect_filled(cell, egui::Rounding::ZERO, theme::wash(p.text, (255.0 * lit) as u8));
        }
        if i > 0 {
            let x = cell.left().round() + 0.5;
            ui.painter().line_segment(
                [Pos2::new(x, cell.top()), Pos2::new(x, cell.bottom())],
                Stroke::new(1.0_f32, p.line),
            );
        }
        let rest = if resp.hovered() { p.text } else { p.text_2 };
        let ink = theme::mix(rest, p.void, lit);
        let w = widths[i] - 32.0;
        theme::tracked_text(
            ui.painter(),
            Pos2::new(cell.center().x - w * 0.5, cell.center().y),
            label,
            font.clone(),
            ink,
            0.9,
        );
        if resp.clicked() {
            picked = Some(i);
        }
    }
    ui.painter().rect_stroke(
        rect.shrink(0.5),
        egui::Rounding::ZERO,
        Stroke::new(1.0_f32, p.line_strong),
    );
    picked
}

// popups here---

pub fn popover_frame() -> egui::Frame {
    let p = pal();
    egui::Frame::none()
        .fill(p.panel)
        .stroke(Stroke::new(1.0_f32, p.line_strong))
        .rounding(egui::Rounding::same(theme::RADIUS))
        .inner_margin(egui::Margin::symmetric(4.0, 4.0))
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
        Vec2::new(w, 25.0),
        if enabled { Sense::click() } else { Sense::hover() },
    );
    rects.push(rect);
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered() && enabled, anim::HOVER);
    if hot > 0.01 {
        let fill = if critical {
            theme::wash(p.danger, (40.0 * hot) as u8)
        } else {
            theme::wash(if p.dark { p.panel_2 } else { p.sunken }, (255.0 * hot) as u8)
        };
        ui.painter().rect_filled(rect, egui::Rounding::ZERO, fill);
        ui.painter().rect_filled(
            Rect::from_min_size(rect.left_top(), Vec2::new(2.0, rect.height())),
            egui::Rounding::ZERO,
            theme::wash(if critical { p.danger } else { p.signal_hi }, (255.0 * hot) as u8),
        );
    }
    let base = if !enabled {
        theme::wash(p.text_3, 110)
    } else if critical {
        p.danger_hi
    } else {
        p.text_2
    };
    let color = theme::mix(base, if critical { theme::lighten(p.danger_hi, 0.2) } else { p.text }, hot);
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

pub fn dropdown(ui: &mut egui::Ui, icon: Icon, label: &str, open: bool, width: f32) -> Response {
    let p = pal();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 30.0), Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered() || open, anim::HOVER);
    theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, open);
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
    ui.painter().rect_filled(
        Rect::from_center_size(rect.center(), Vec2::splat(11.0 + 2.0 * hot)),
        egui::Rounding::ZERO,
        color,
    );
    if selected {
        ui.painter().rect_stroke(
            Rect::from_center_size(rect.center(), Vec2::splat(17.0)),
            egui::Rounding::ZERO,
            Stroke::new(1.0_f32, pal().text),
        );
    }
    resp.on_hover_text(tip).clicked()
}
