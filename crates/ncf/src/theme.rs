use std::sync::RwLock;

use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, Margin, Mesh, Pos2, Rect, Rounding, Shape, Stroke, Vec2};

use crate::tokens::{self, Colors};

// palette here---

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pal {
    pub dark: bool,
    pub colors: Colors,
}

impl std::ops::Deref for Pal {
    type Target = Colors;
    fn deref(&self) -> &Colors {
        &self.colors
    }
}

impl Pal {
    pub fn group(&self, hue: usize) -> Color32 {
        self.colors.groups[hue % self.colors.groups.len()]
    }

    pub fn hover_fill(&self) -> Color32 {
        if self.dark {
            self.panel_2
        } else {
            self.sunken
        }
    }
}

pub fn palette(dark: bool) -> Pal {
    Pal { dark, colors: if dark { tokens::DARK } else { tokens::LIGHT } }
}

static CURRENT: RwLock<Option<Pal>> = RwLock::new(None);

pub fn pal() -> Pal {
    if let Ok(g) = CURRENT.read() {
        if let Some(p) = *g {
            return p;
        }
    }
    palette(true)
}

pub fn set_palette(dark: bool) {
    if let Ok(mut g) = CURRENT.write() {
        *g = Some(palette(dark));
    }
}

pub fn set_look(ctx: &egui::Context, dark: bool) {
    set_palette(dark);
    apply(ctx);
}

// metrics here---

pub const RADIUS: f32 = tokens::RADIUS;

pub const T_TITLE: f32 = tokens::SIZE_TITLE;
pub const T_H: f32 = tokens::SIZE_HEADING;
pub const T_BODY: f32 = tokens::SIZE_BODY;
pub const T_SM: f32 = tokens::SIZE_SMALL;
pub const T_LABEL: f32 = tokens::SIZE_LABEL;
pub const T_CAP: f32 = tokens::SIZE_CAPTION;
pub const T_MICRO: f32 = tokens::SIZE_MICRO;

pub const HUD_TRACKING: f32 = 0.8;

// colour math here---

pub fn wash(col: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(col.r(), col.g(), col.b(), alpha)
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round().clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()), f(a.a(), b.a()))
}

pub fn lighten(col: Color32, t: f32) -> Color32 {
    mix(col, Color32::WHITE, t)
}

pub fn darken(col: Color32, t: f32) -> Color32 {
    mix(col, Color32::BLACK, t)
}

pub fn on(bg: Color32) -> Color32 {
    let l = 0.299 * bg.r() as f32 + 0.587 * bg.g() as f32 + 0.114 * bg.b() as f32;
    if l > 150.0 {
        Color32::from_rgb(0x0B, 0x0C, 0x12)
    } else {
        Color32::from_rgb(0xF2, 0xF3, 0xF7)
    }
}

pub fn tint(col: Color32, k: f32) -> Color32 {
    let k = k.clamp(0.0, 1.0);
    let f = |v: u8| (v as f32 * k).round() as u8;
    Color32::from_rgba_premultiplied(f(col.r()), f(col.g()), f(col.b()), (255.0 * k).round() as u8)
}

pub fn hsl(h: f32, s: f32, l: f32) -> Color32 {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = (h.fract() + 1.0).fract() * 6.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let (r, g, b) = match hp as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c * 0.5;
    let to = |v: f32| ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgb(to(r), to(g), to(b))
}

// shapes here---

pub fn rounded_poly(rect: Rect, radius: f32) -> Vec<Pos2> {
    let r = radius.min(rect.width() * 0.5).min(rect.height() * 0.5).max(0.0);
    if r <= 0.5 {
        return vec![rect.left_top(), rect.right_top(), rect.right_bottom(), rect.left_bottom()];
    }
    let steps = ((r * 0.9) as usize).clamp(5, 18);
    let mut pts = Vec::with_capacity(steps * 4 + 4);
    let corners = [
        (rect.right() - r, rect.top() + r, -std::f32::consts::FRAC_PI_2),
        (rect.right() - r, rect.bottom() - r, 0.0),
        (rect.left() + r, rect.bottom() - r, std::f32::consts::FRAC_PI_2),
        (rect.left() + r, rect.top() + r, std::f32::consts::PI),
    ];
    for (cx, cy, a0) in corners {
        for i in 0..=steps {
            let a = a0 + std::f32::consts::FRAC_PI_2 * (i as f32 / steps as f32);
            pts.push(Pos2::new(cx + a.cos() * r, cy + a.sin() * r));
        }
    }
    pts
}

pub fn fill_grad_poly(painter: &egui::Painter, pts: &[Pos2], a: Color32, b: Color32, dir: Vec2) {
    painter.add(grad_poly_shape(pts, a, b, dir));
}

pub fn grad_poly_shape(pts: &[Pos2], a: Color32, b: Color32, dir: Vec2) -> Shape {
    if pts.len() < 3 {
        return Shape::Noop;
    }
    let dir = if dir.length_sq() < 1e-6 { Vec2::new(0.0, 1.0) } else { dir.normalized() };
    let (mut lo, mut hi) = (f32::MAX, f32::MIN);
    for p in pts {
        let t = p.to_vec2().dot(dir);
        lo = lo.min(t);
        hi = hi.max(t);
    }
    let span = (hi - lo).max(0.0001);
    let at = |p: Pos2| -> Color32 { mix(a, b, (p.to_vec2().dot(dir) - lo) / span) };
    let mut centre = Vec2::ZERO;
    for p in pts {
        centre += p.to_vec2();
    }
    let centre = (centre / pts.len() as f32).to_pos2();
    let mut mesh = Mesh::default();
    mesh.colored_vertex(centre, at(centre));
    for p in pts {
        mesh.colored_vertex(*p, at(*p));
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    const FEATHER: f32 = 0.9;
    let base = mesh.vertices.len() as u32;
    for p in pts {
        let out = (*p - centre).normalized() * FEATHER;
        mesh.colored_vertex(*p + out, wash(at(*p), 0));
    }
    for i in 0..n {
        let j = (i + 1) % n;
        mesh.add_triangle(1 + i, base + i, base + j);
        mesh.add_triangle(1 + i, base + j, 1 + j);
    }
    Shape::mesh(mesh)
}

// surfaces here---

pub fn hover_surface(painter: &egui::Painter, rect: Rect, radius: f32, accent: Color32, t: f32, on: bool) {
    let p = pal();
    let t = t.clamp(0.0, 1.0);
    if t <= 0.005 && !on {
        return;
    }
    let lit = t.max(if on { 1.0 } else { 0.0 });
    painter.rect_filled(rect, Rounding::same(radius), wash(p.hover_fill(), (255.0 * lit) as u8));
    let edge = if on { accent } else { p.line_strong };
    painter.rect_stroke(
        rect.shrink(0.5),
        Rounding::same(radius),
        Stroke::new(1.0_f32, wash(edge, ((if on { 255.0 } else { 140.0 }) * lit) as u8)),
    );
}

pub fn panel(painter: &egui::Painter, rect: Rect) {
    let p = pal();
    painter.rect_filled(rect, Rounding::same(RADIUS), p.panel);
    painter.rect_stroke(rect.shrink(0.5), Rounding::same(RADIUS), Stroke::new(1.0_f32, p.line_strong));
    top_edge(painter, rect);
}

pub fn top_edge(painter: &egui::Painter, rect: Rect) {
    let p = pal();
    if p.edge.a() > 0 {
        painter.line_segment(
            [rect.left_top() + Vec2::new(1.0, 1.5), rect.right_top() + Vec2::new(-1.0, 1.5)],
            Stroke::new(1.0_f32, p.edge),
        );
    }
}

pub fn panel_frame() -> egui::Frame {
    let p = pal();
    egui::Frame::none().fill(p.deep).rounding(Rounding::same(RADIUS)).stroke(Stroke::new(1.0_f32, p.line))
}

pub fn panel_box() -> egui::Frame {
    let p = pal();
    egui::Frame::none()
        .fill(p.panel)
        .rounding(Rounding::same(RADIUS))
        .stroke(Stroke::new(1.0_f32, p.line_strong))
        .inner_margin(Margin::symmetric(tokens::SPACE_6, tokens::SPACE_4 + 2.0))
}

pub fn focus_ring(painter: &egui::Painter, rect: Rect) {
    let p = pal();
    painter.rect_stroke(
        rect.expand(tokens::FOCUS_OFFSET + tokens::FOCUS_RING * 0.5),
        Rounding::same(RADIUS),
        Stroke::new(tokens::FOCUS_RING, p.signal_hi),
    );
}

// text here---

pub fn tracked_text(painter: &egui::Painter, left_centre: Pos2, text: &str, font: egui::FontId, color: Color32, tracking: f32) -> f32 {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.0,
        egui::TextFormat { font_id: font, color, extra_letter_spacing: tracking, ..Default::default() },
    );
    let galley = painter.ctx().fonts(|f| f.layout_job(job));
    let size = galley.rect.size();
    painter.galley(Pos2::new(left_centre.x, left_centre.y - size.y * 0.5), galley, color);
    (size.x - tracking).max(0.0)
}

pub fn tracked_width(ctx: &egui::Context, text: &str, font: egui::FontId, tracking: f32) -> f32 {
    let mut job = egui::text::LayoutJob::default();
    job.append(text, 0.0, egui::TextFormat { font_id: font, extra_letter_spacing: tracking, ..Default::default() });
    let w = ctx.fonts(|f| f.layout_job(job)).rect.width();
    (w - tracking).max(0.0)
}

// visuals here---

pub fn apply(ctx: &egui::Context) {
    let p = pal();
    let mut v = if p.dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    let r = Rounding::same(RADIUS);
    v.override_text_color = Some(p.text);
    v.panel_fill = Color32::TRANSPARENT;
    v.window_fill = p.deep;
    v.extreme_bg_color = if p.dark { Color32::BLACK } else { Color32::WHITE };
    v.faint_bg_color = p.panel_2;
    v.window_stroke = Stroke::new(1.0_f32, p.line_strong);
    v.window_rounding = r;
    v.menu_rounding = r;
    v.selection.bg_fill = wash(p.signal, if p.dark { 150 } else { 90 });
    v.selection.stroke = Stroke::new(1.0_f32, p.signal);
    v.hyperlink_color = p.signal_hi;
    v.popup_shadow = egui::epaint::Shadow::NONE;
    v.window_shadow = egui::epaint::Shadow::NONE;
    v.text_cursor.stroke = Stroke::new(2.0_f32, p.signal_hi);

    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.deep;
    w.noninteractive.weak_bg_fill = p.deep;
    w.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.line);
    w.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text_2);
    w.noninteractive.rounding = r;
    w.inactive.bg_fill = p.sunken;
    w.inactive.weak_bg_fill = p.sunken;
    w.inactive.bg_stroke = Stroke::new(1.0_f32, p.line_strong);
    w.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    w.inactive.rounding = r;
    w.hovered.bg_fill = p.panel;
    w.hovered.weak_bg_fill = p.panel;
    w.hovered.bg_stroke = Stroke::new(1.0_f32, p.line_strong);
    w.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
    w.hovered.rounding = r;
    w.hovered.expansion = 0.0;
    w.active.bg_fill = p.signal_soft_hi;
    w.active.weak_bg_fill = p.signal_soft_hi;
    w.active.bg_stroke = Stroke::new(1.0_f32, p.signal);
    w.active.fg_stroke = Stroke::new(1.0_f32, p.signal_hi);
    w.active.rounding = r;
    w.open.bg_fill = p.panel_2;
    w.open.weak_bg_fill = p.panel_2;
    w.open.bg_stroke = Stroke::new(1.0_f32, p.line);
    w.open.fg_stroke = Stroke::new(1.0_f32, p.text);
    w.open.rounding = r;

    ctx.all_styles_mut(|s| {
        s.visuals = v.clone();
        s.spacing.item_spacing = Vec2::new(8.0, 8.0);
        s.spacing.button_padding = Vec2::new(11.0, 6.0);
        s.spacing.window_margin = Margin::same(18.0);
        s.spacing.menu_margin = Margin::same(6.0);
        s.spacing.scroll.bar_width = 8.0;
        s.spacing.scroll.floating = true;
        s.visuals.clip_rect_margin = 2.0;
    });
}

// fonts here---

const ANYBODY_REGULAR: &[u8] = include_bytes!("../fonts/Anybody-Regular.ttf");
const ANYBODY_MEDIUM: &[u8] = include_bytes!("../fonts/Anybody-Medium.ttf");
const ANYBODY_SEMIBOLD: &[u8] = include_bytes!("../fonts/Anybody-SemiBold.ttf");
const ANYBODY_BOLD: &[u8] = include_bytes!("../fonts/Anybody-Bold.ttf");
const ANYBODY_DISPLAY: &[u8] = include_bytes!("../fonts/Anybody-Display.ttf");
const MONO_REGULAR: &[u8] = include_bytes!("../fonts/JetBrainsMono-Regular.ttf");
const MONO_MEDIUM: &[u8] = include_bytes!("../fonts/JetBrainsMono-Medium.ttf");

pub fn medium() -> FontFamily {
    FontFamily::Name("anybody-medium".into())
}
pub fn semibold() -> FontFamily {
    FontFamily::Name("anybody-semibold".into())
}
pub fn bold() -> FontFamily {
    FontFamily::Name("anybody-bold".into())
}
pub fn display() -> FontFamily {
    FontFamily::Name("anybody-display".into())
}
pub fn hud() -> FontFamily {
    FontFamily::Name("mono-medium".into())
}

pub fn font_definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    for (key, bytes) in [
        ("Anybody", ANYBODY_REGULAR),
        ("AnybodyMedium", ANYBODY_MEDIUM),
        ("AnybodySemiBold", ANYBODY_SEMIBOLD),
        ("AnybodyBold", ANYBODY_BOLD),
        ("AnybodyDisplay", ANYBODY_DISPLAY),
        ("JetBrainsMono", MONO_REGULAR),
        ("JetBrainsMonoMedium", MONO_MEDIUM),
    ] {
        fonts.font_data.insert(key.to_string(), FontData::from_static(bytes));
    }
    fonts.families.entry(FontFamily::Proportional).or_default().insert(0, "Anybody".to_string());
    fonts.families.entry(FontFamily::Monospace).or_default().insert(0, "JetBrainsMono".to_string());
    let proportional = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mono = fonts.families.get(&FontFamily::Monospace).cloned().unwrap_or_default();
    for (family, key, rest) in [
        (medium(), "AnybodyMedium", &proportional),
        (semibold(), "AnybodySemiBold", &proportional),
        (bold(), "AnybodyBold", &proportional),
        (display(), "AnybodyDisplay", &proportional),
        (hud(), "JetBrainsMonoMedium", &mono),
    ] {
        let mut stack = vec![key.to_string()];
        stack.extend(rest.iter().cloned());
        fonts.families.insert(family, stack);
    }
    fonts
}

pub fn add_font(fonts: &mut FontDefinitions, family: &str, key: &str, bytes: &'static [u8], mono_fallback: bool) {
    fonts.font_data.insert(key.to_string(), FontData::from_static(bytes));
    let base = if mono_fallback { FontFamily::Monospace } else { FontFamily::Proportional };
    let mut stack = vec![key.to_string()];
    stack.extend(fonts.families.get(&base).cloned().unwrap_or_default());
    fonts.families.insert(FontFamily::Name(family.into()), stack);
}

pub fn install_fonts(ctx: &egui::Context) {
    ctx.set_fonts(font_definitions());
}

pub fn font(size: f32) -> egui::FontId {
    egui::FontId::proportional(size)
}
pub fn font_med(size: f32) -> egui::FontId {
    egui::FontId::new(size, medium())
}
pub fn font_semi(size: f32) -> egui::FontId {
    egui::FontId::new(size, semibold())
}
pub fn font_bold(size: f32) -> egui::FontId {
    egui::FontId::new(size, bold())
}
pub fn font_mono(size: f32) -> egui::FontId {
    egui::FontId::monospace(size)
}
pub fn font_display(size: f32) -> egui::FontId {
    egui::FontId::new(size, display())
}
pub fn font_hud(size: f32) -> egui::FontId {
    egui::FontId::new(size, hud())
}
