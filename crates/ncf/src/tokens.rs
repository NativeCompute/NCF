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
    pub groups: [Color32; 6],
}

impl Colors {
    pub fn named(&self) -> [(&'static str, Color32); 26] {
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
        ]
    }
}

pub const DARK: Colors = Colors {
    void: Color32::from_rgb(0, 0, 0),
    deep: Color32::from_rgb(7, 8, 13),
    panel: Color32::from_rgb(12, 14, 22),
    panel_2: Color32::from_rgb(19, 22, 36),
    sunken: Color32::from_rgb(0, 0, 0),
    line: Color32::from_rgba_premultiplied(98, 98, 98, 31),
    line_strong: Color32::from_rgba_premultiplied(139, 139, 139, 66),
    text: Color32::from_rgb(242, 243, 247),
    text_2: Color32::from_rgb(166, 171, 194),
    text_3: Color32::from_rgb(138, 143, 168),
    ink: Color32::from_rgb(255, 255, 255),
    signal: Color32::from_rgb(59, 76, 255),
    signal_hi: Color32::from_rgb(110, 123, 255),
    signal_deep: Color32::from_rgb(42, 54, 200),
    signal_soft: Color32::from_rgba_premultiplied(18, 26, 105, 36),
    signal_soft_hi: Color32::from_rgba_premultiplied(25, 34, 129, 56),
    chrome: Color32::from_rgb(201, 205, 219),
    chrome_hi: Color32::from_rgb(242, 243, 247),
    chrome_lo: Color32::from_rgb(138, 143, 168),
    chrome_shade: Color32::from_rgb(138, 143, 168),
    danger: Color32::from_rgb(255, 107, 122),
    danger_hi: Color32::from_rgb(255, 154, 165),
    warn: Color32::from_rgb(245, 196, 81),
    ok: Color32::from_rgb(94, 224, 160),
    edge: Color32::from_rgba_premultiplied(69, 69, 69, 15),
    guide: Color32::from_rgba_premultiplied(53, 53, 53, 9),
    groups: [
        Color32::from_rgb(110, 123, 255),
        Color32::from_rgb(92, 200, 255),
        Color32::from_rgb(167, 139, 250),
        Color32::from_rgb(94, 224, 160),
        Color32::from_rgb(245, 196, 81),
        Color32::from_rgb(255, 107, 122),
    ],
};

pub const LIGHT: Colors = Colors {
    void: Color32::from_rgb(244, 245, 248),
    deep: Color32::from_rgb(255, 255, 255),
    panel: Color32::from_rgb(246, 247, 250),
    panel_2: Color32::from_rgb(230, 233, 241),
    sunken: Color32::from_rgb(236, 238, 244),
    line: Color32::from_rgba_premultiplied(0, 0, 0, 31),
    line_strong: Color32::from_rgba_premultiplied(0, 0, 0, 66),
    text: Color32::from_rgb(11, 12, 18),
    text_2: Color32::from_rgb(69, 74, 94),
    text_3: Color32::from_rgb(95, 100, 122),
    ink: Color32::from_rgb(255, 255, 255),
    signal: Color32::from_rgb(59, 76, 255),
    signal_hi: Color32::from_rgb(43, 58, 224),
    signal_deep: Color32::from_rgb(31, 42, 176),
    signal_soft: Color32::from_rgba_premultiplied(14, 21, 90, 26),
    signal_soft_hi: Color32::from_rgba_premultiplied(21, 29, 115, 44),
    chrome: Color32::from_rgb(42, 45, 58),
    chrome_hi: Color32::from_rgb(11, 12, 18),
    chrome_lo: Color32::from_rgb(95, 100, 122),
    chrome_shade: Color32::from_rgb(5, 6, 10),
    danger: Color32::from_rgb(209, 36, 59),
    danger_hi: Color32::from_rgb(229, 75, 94),
    warn: Color32::from_rgb(183, 123, 0),
    ok: Color32::from_rgb(20, 138, 79),
    edge: Color32::from_rgba_premultiplied(0, 0, 0, 0),
    guide: Color32::from_rgba_premultiplied(0, 0, 0, 12),
    groups: [
        Color32::from_rgb(59, 76, 255),
        Color32::from_rgb(0, 134, 194),
        Color32::from_rgb(124, 77, 232),
        Color32::from_rgb(20, 138, 79),
        Color32::from_rgb(183, 123, 0),
        Color32::from_rgb(209, 36, 59),
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

pub const FONT_DISPLAY: FontRole = FontRole { family: "Anybody", weight: 800, stretch: 150, tracking: -0.02, upper: true };
pub const FONT_HEADING: FontRole = FontRole { family: "Anybody", weight: 700, stretch: 130, tracking: 0.0, upper: false };
pub const FONT_UI: FontRole = FontRole { family: "Anybody", weight: 600, stretch: 112, tracking: 0.0, upper: false };
pub const FONT_BODY: FontRole = FontRole { family: "Anybody", weight: 400, stretch: 100, tracking: 0.0, upper: false };
pub const FONT_HUD: FontRole = FontRole { family: "JetBrains Mono", weight: 500, stretch: 100, tracking: 0.08, upper: true };

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

pub const RADIUS: f32 = 2.0;
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
