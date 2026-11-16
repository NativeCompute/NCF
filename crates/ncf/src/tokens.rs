// generated from tokens/ncf.toml by ncf tokens here---

use eframe::egui::Color32;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colors {
    pub void: Color32,
    pub deep: Color32,
    pub panel: Color32,
    pub panel_2: Color32,
    pub sunken: Color32,
    pub line: Color32,
    pub line_strong: Color32,
    pub text: Color32,
    pub text_2: Color32,
    pub text_3: Color32,
    pub ink: Color32,
    pub signal: Color32,
    pub signal_hi: Color32,
    pub signal_deep: Color32,
    pub signal_soft: Color32,
    pub signal_soft_hi: Color32,
    pub chrome: Color32,
    pub chrome_hi: Color32,
    pub chrome_lo: Color32,
    pub chrome_shade: Color32,
    pub danger: Color32,
    pub danger_hi: Color32,
    pub warn: Color32,
    pub ok: Color32,
    pub edge: Color32,
    pub guide: Color32,
    pub focus: Color32,
    pub on_danger: Color32,
    pub on_tile: Color32,
    pub glass: Color32,
    pub glass_line: Color32,
    pub shadow: Color32,
    pub shadow_soft: Color32,
    pub wall_a: Color32,
    pub wall_b: Color32,
    pub wall_c: Color32,
    pub glow: Color32,
    pub wall_from: Color32,
    pub wall_to: Color32,
    pub groups: [Color32; 6],
    pub brights: [Color32; 6],
}

impl Colors {
    pub fn named(&self) -> [(&'static str, Color32); 39] {
        [
            ("void", self.void),
            ("deep", self.deep),
            ("panel", self.panel),
            ("panel-2", self.panel_2),
            ("sunken", self.sunken),
            ("line", self.line),
            ("line-strong", self.line_strong),
            ("text", self.text),
            ("text-2", self.text_2),
            ("text-3", self.text_3),
            ("ink", self.ink),
            ("signal", self.signal),
            ("signal-hi", self.signal_hi),
            ("signal-deep", self.signal_deep),
            ("signal-soft", self.signal_soft),
            ("signal-soft-hi", self.signal_soft_hi),
            ("chrome", self.chrome),
            ("chrome-hi", self.chrome_hi),
            ("chrome-lo", self.chrome_lo),
            ("chrome-shade", self.chrome_shade),
            ("danger", self.danger),
            ("danger-hi", self.danger_hi),
            ("warn", self.warn),
            ("ok", self.ok),
            ("edge", self.edge),
            ("guide", self.guide),
            ("focus", self.focus),
            ("on-danger", self.on_danger),
            ("on-tile", self.on_tile),
            ("glass", self.glass),
            ("glass-line", self.glass_line),
            ("shadow", self.shadow),
            ("shadow-soft", self.shadow_soft),
            ("wall-a", self.wall_a),
            ("wall-b", self.wall_b),
            ("wall-c", self.wall_c),
            ("glow", self.glow),
            ("wall-from", self.wall_from),
            ("wall-to", self.wall_to),
        ]
    }
}

pub const DARK: Colors = Colors {
    void: Color32::from_rgb(14, 15, 19),
    deep: Color32::from_rgb(26, 27, 33),
    panel: Color32::from_rgb(26, 27, 33),
    panel_2: Color32::from_rgb(36, 38, 46),
    sunken: Color32::from_rgb(14, 15, 19),
    line: Color32::from_rgb(44, 46, 55),
    line_strong: Color32::from_rgb(59, 62, 73),
    text: Color32::from_rgb(243, 243, 246),
    text_2: Color32::from_rgb(184, 186, 196),
    text_3: Color32::from_rgb(146, 149, 162),
    ink: Color32::from_rgb(26, 12, 6),
    signal: Color32::from_rgb(255, 106, 61),
    signal_hi: Color32::from_rgb(255, 154, 115),
    signal_deep: Color32::from_rgb(242, 90, 44),
    signal_soft: Color32::from_rgba_premultiplied(105, 39, 19, 36),
    signal_soft_hi: Color32::from_rgba_premultiplied(129, 50, 26, 56),
    chrome: Color32::from_rgb(214, 215, 222),
    chrome_hi: Color32::from_rgb(243, 243, 246),
    chrome_lo: Color32::from_rgb(146, 149, 162),
    chrome_shade: Color32::from_rgb(146, 149, 162),
    danger: Color32::from_rgb(255, 107, 120),
    danger_hi: Color32::from_rgb(255, 154, 165),
    warn: Color32::from_rgb(255, 203, 92),
    ok: Color32::from_rgb(91, 227, 160),
    edge: Color32::from_rgba_premultiplied(69, 69, 69, 15),
    guide: Color32::from_rgba_premultiplied(0, 0, 0, 0),
    focus: Color32::from_rgb(255, 255, 255),
    on_danger: Color32::from_rgb(19, 19, 24),
    on_tile: Color32::from_rgb(19, 19, 24),
    glass: Color32::from_rgba_premultiplied(75, 75, 75, 18),
    glass_line: Color32::from_rgba_premultiplied(118, 118, 118, 46),
    shadow: Color32::from_rgba_premultiplied(0, 0, 0, 115),
    shadow_soft: Color32::from_rgba_premultiplied(0, 0, 0, 89),
    wall_a: Color32::from_rgb(90, 36, 24),
    wall_b: Color32::from_rgb(23, 62, 94),
    wall_c: Color32::from_rgb(51, 37, 107),
    glow: Color32::from_rgba_premultiplied(0, 0, 0, 0),
    wall_from: Color32::from_rgb(23, 17, 15),
    wall_to: Color32::from_rgb(20, 18, 36),
    groups: [
        Color32::from_rgb(255, 122, 80),
        Color32::from_rgb(185, 166, 255),
        Color32::from_rgb(255, 203, 92),
        Color32::from_rgb(143, 203, 255),
        Color32::from_rgb(91, 227, 160),
        Color32::from_rgb(255, 143, 177),
    ],
    brights: [
        Color32::from_rgb(255, 122, 80),
        Color32::from_rgb(185, 166, 255),
        Color32::from_rgb(255, 203, 92),
        Color32::from_rgb(143, 203, 255),
        Color32::from_rgb(91, 227, 160),
        Color32::from_rgb(255, 143, 177),
    ],
};

pub const LIGHT: Colors = Colors {
    void: Color32::from_rgb(233, 235, 240),
    deep: Color32::from_rgb(255, 255, 255),
    panel: Color32::from_rgb(255, 255, 255),
    panel_2: Color32::from_rgb(243, 244, 247),
    sunken: Color32::from_rgb(233, 235, 240),
    line: Color32::from_rgb(226, 229, 234),
    line_strong: Color32::from_rgb(205, 210, 218),
    text: Color32::from_rgb(19, 19, 24),
    text_2: Color32::from_rgb(69, 73, 86),
    text_3: Color32::from_rgb(94, 99, 114),
    ink: Color32::from_rgb(26, 12, 6),
    signal: Color32::from_rgb(255, 106, 61),
    signal_hi: Color32::from_rgb(178, 60, 20),
    signal_deep: Color32::from_rgb(242, 90, 44),
    signal_soft: Color32::from_rgba_premultiplied(98, 36, 17, 31),
    signal_soft_hi: Color32::from_rgba_premultiplied(124, 47, 24, 51),
    chrome: Color32::from_rgb(42, 44, 54),
    chrome_hi: Color32::from_rgb(19, 19, 24),
    chrome_lo: Color32::from_rgb(94, 99, 114),
    chrome_shade: Color32::from_rgb(8, 8, 11),
    danger: Color32::from_rgb(181, 32, 47),
    danger_hi: Color32::from_rgb(210, 58, 72),
    warn: Color32::from_rgb(138, 90, 0),
    ok: Color32::from_rgb(22, 113, 74),
    edge: Color32::from_rgba_premultiplied(0, 0, 0, 0),
    guide: Color32::from_rgba_premultiplied(0, 0, 0, 0),
    focus: Color32::from_rgb(19, 19, 24),
    on_danger: Color32::from_rgb(255, 255, 255),
    on_tile: Color32::from_rgb(19, 19, 24),
    glass: Color32::from_rgba_premultiplied(158, 158, 158, 87),
    glass_line: Color32::from_rgba_premultiplied(231, 231, 231, 204),
    shadow: Color32::from_rgba_premultiplied(3, 3, 4, 31),
    shadow_soft: Color32::from_rgba_premultiplied(2, 2, 2, 20),
    wall_a: Color32::from_rgb(255, 196, 175),
    wall_b: Color32::from_rgb(180, 224, 255),
    wall_c: Color32::from_rgb(210, 196, 255),
    glow: Color32::from_rgba_premultiplied(0, 0, 0, 0),
    wall_from: Color32::from_rgb(244, 232, 226),
    wall_to: Color32::from_rgb(231, 227, 246),
    groups: [
        Color32::from_rgb(178, 60, 20),
        Color32::from_rgb(106, 79, 208),
        Color32::from_rgb(138, 90, 0),
        Color32::from_rgb(31, 106, 165),
        Color32::from_rgb(22, 113, 74),
        Color32::from_rgb(181, 32, 94),
    ],
    brights: [
        Color32::from_rgb(255, 122, 80),
        Color32::from_rgb(185, 166, 255),
        Color32::from_rgb(255, 203, 92),
        Color32::from_rgb(143, 203, 255),
        Color32::from_rgb(91, 227, 160),
        Color32::from_rgb(255, 143, 177),
    ],
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FontRole {
    pub family: &'static str,
    pub weight: u16,
    pub stretch: u16,
    pub tracking: f32,
    pub upper: bool,
}

pub const FONT_DISPLAY: FontRole = FontRole { family: "Bricolage Grotesque", weight: 800, stretch: 100, tracking: -0.045, upper: false };
pub const FONT_HEADING: FontRole = FontRole { family: "Bricolage Grotesque", weight: 800, stretch: 100, tracking: -0.02, upper: false };
pub const FONT_UI: FontRole = FontRole { family: "Plus Jakarta Sans", weight: 600, stretch: 100, tracking: 0.0, upper: false };
pub const FONT_BODY: FontRole = FontRole { family: "Plus Jakarta Sans", weight: 400, stretch: 100, tracking: 0.0, upper: false };
pub const FONT_HUD: FontRole = FontRole { family: "Plus Jakarta Sans", weight: 560, stretch: 100, tracking: 0.0, upper: false };

pub const SIZE_BODY: f32 = 14.5;
pub const SIZE_CAPTION: f32 = 11.0;
pub const SIZE_HEADING: f32 = 17.0;
pub const SIZE_LABEL: f32 = 12.0;
pub const SIZE_MICRO: f32 = 10.0;
pub const SIZE_SMALL: f32 = 13.0;
pub const SIZE_TITLE: f32 = 26.0;

pub const SPACE_1: f32 = 4.0;
pub const SPACE_12: f32 = 48.0;
pub const SPACE_16: f32 = 64.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_24: f32 = 96.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_32: f32 = 128.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_6: f32 = 24.0;
pub const SPACE_8: f32 = 32.0;

pub const RADIUS_SM: f32 = 12.0;
pub const RADIUS_MD: f32 = 20.0;
pub const RADIUS_LG: f32 = 28.0;
pub const RADIUS_XL: f32 = 32.0;
pub const RADIUS_PILL: f32 = 999.0;
pub const RADIUS: f32 = RADIUS_SM;
pub const SHADOW_LIFT: [f32; 4] = [0.0, 16.0, 40.0, 0.0];
pub const SHADOW_FLOAT: [f32; 4] = [0.0, 10.0, 30.0, 0.0];
pub const SHADOW_REST: [f32; 4] = [0.0, 1.0, 2.0, 0.0];
pub const WALL_A: [f32; 5] = [0.0, 0.0, 0.55, 0.85, 0.62];
pub const WALL_B: [f32; 5] = [0.88, 0.06, 0.4, 0.5, 0.6];
pub const WALL_C: [f32; 5] = [1.0, 1.0, 0.55, 0.8, 0.62];
pub const GLOW: [f32; 5] = [0.0, 0.0, 0.75, 0.75, 0.7];
pub const LINE: f32 = 1.0;
pub const FOCUS_RING: f32 = 2.0;
pub const FOCUS_OFFSET: f32 = 2.0;
pub const GUTTER_NARROW: f32 = 20.0;
pub const GUTTER_WIDE: f32 = 40.0;
pub const COLUMNS: usize = 6;

pub const MOTION_FAST: f32 = 0.1;
pub const MOTION_QUICK: f32 = 0.15;
pub const MOTION_BASE: f32 = 0.2;
pub const MOTION_SLOW: f32 = 0.3;
pub const MOTION_HOVER: f32 = 0.24;
pub const EASE_OUT: [f32; 4] = [0.16, 1.0, 0.3, 1.0];
pub const EASE_IN: [f32; 4] = [0.7, 0.0, 0.84, 0.0];

pub const BRAND: &str = "NativeCompute";
pub const BRAND_URL: &str = "https://nativecompute.net";
