use eframe::egui::Color32;
use std::fmt::Write;

pub const COLOR_NAMES: [&str; 25] = [
    "void", "deep", "panel", "panel_2", "sunken", "line", "line_strong", "text", "text_2", "text_3", "ink",
    "signal", "signal_hi", "signal_deep", "signal_soft", "signal_soft_hi", "chrome", "chrome_hi", "chrome_lo",
    "danger", "danger_hi", "warn", "ok", "edge", "guide",
];

pub const FONT_ROLES: [&str; 5] = ["display", "heading", "ui", "body", "hud"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

pub fn hex(s: &str) -> Result<Rgba, String> {
    let h = s.trim().trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(&h[i..i + 2], 16).map_err(|e| format!("{s}: {e}"));
    match h.len() {
        6 => Ok(Rgba(byte(0)?, byte(2)?, byte(4)?, 255)),
        8 => Ok(Rgba(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
        _ => Err(format!("{s}: a colour is #rrggbb or #rrggbbaa")),
    }
}

pub struct Tokens(pub toml::Value);

impl Tokens {
    pub fn load(text: &str) -> Result<Tokens, String> {
        text.parse::<toml::Value>().map(Tokens).map_err(|e| e.to_string())
    }

    fn get(&self, path: &str) -> Result<&toml::Value, String> {
        let mut v = &self.0;
        for part in path.split('.') {
            v = v.get(part).ok_or_else(|| format!("tokens: {path} is missing"))?;
        }
        Ok(v)
    }

    fn str(&self, path: &str) -> Result<String, String> {
        self.get(path)?.as_str().map(str::to_string).ok_or_else(|| format!("tokens: {path} is not text"))
    }

    fn num(&self, path: &str) -> Result<f64, String> {
        let v = self.get(path)?;
        v.as_float()
            .or_else(|| v.as_integer().map(|i| i as f64))
            .ok_or_else(|| format!("tokens: {path} is not a number"))
    }

    fn table(&self, path: &str) -> Result<Vec<(String, toml::Value)>, String> {
        let t = self.get(path)?.as_table().ok_or_else(|| format!("tokens: {path} is not a table"))?;
        Ok(t.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
    }

    pub fn color(&self, cast: &str, name: &str) -> Result<Rgba, String> {
        hex(&self.str(&format!("color.{cast}.{name}"))?)
    }

    pub fn groups(&self, cast: &str) -> Result<Vec<Rgba>, String> {
        let list = self
            .get(&format!("color.{cast}.groups"))?
            .as_array()
            .ok_or_else(|| format!("tokens: color.{cast}.groups is not a list"))?;
        list.iter()
            .map(|v| v.as_str().ok_or("tokens: a group colour is not text".to_string()).and_then(hex))
            .collect()
    }
}

fn premultiplied(c: Rgba) -> [u8; 4] {
    Color32::from_rgba_unmultiplied(c.0, c.1, c.2, c.3).to_array()
}

fn rust_color(c: Rgba) -> String {
    let [r, g, b, a] = premultiplied(c);
    if a == 255 {
        format!("Color32::from_rgb({r}, {g}, {b})")
    } else {
        format!("Color32::from_rgba_premultiplied({r}, {g}, {b}, {a})")
    }
}

fn css_color(c: Rgba) -> String {
    if c.3 == 255 {
        format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2)
    } else {
        let a = (c.3 as f64 / 255.0 * 1000.0).round() / 1000.0;
        format!("rgba({}, {}, {}, {})", c.0, c.1, c.2, a)
    }
}

fn upper_snake(s: &str) -> String {
    s.to_ascii_uppercase()
}

pub fn rust(t: &Tokens) -> Result<String, String> {
    let mut o = String::new();
    o.push_str("// generated from tokens/ncf.toml by ncf tokens here---\n\n");
    o.push_str("use eframe::egui::Color32;\n\n");
    o.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\npub struct Colors {\n");
    for n in COLOR_NAMES {
        writeln!(o, "    pub {n}: Color32,").unwrap();
    }
    o.push_str("    pub groups: [Color32; 6],\n}\n\n");
    writeln!(o, "impl Colors {{\n    pub fn named(&self) -> [(&'static str, Color32); {}] {{\n        [", COLOR_NAMES.len()).unwrap();
    for n in COLOR_NAMES {
        writeln!(o, "            ({:?}, self.{n}),", n.replace('_', "-")).unwrap();
    }
    o.push_str("        ]\n    }\n}\n\n");
    for (cast, name) in [("dark", "DARK"), ("light", "LIGHT")] {
        writeln!(o, "pub const {name}: Colors = Colors {{").unwrap();
        for n in COLOR_NAMES {
            writeln!(o, "    {n}: {},", rust_color(t.color(cast, n)?)).unwrap();
        }
        let groups = t.groups(cast)?;
        if groups.len() != 6 {
            return Err(format!("tokens: color.{cast}.groups needs six colours"));
        }
        let g: Vec<String> = groups.into_iter().map(rust_color).collect();
        writeln!(o, "    groups: [\n        {},\n    ],", g.join(",\n        ")).unwrap();
        o.push_str("};\n\n");
    }
    o.push_str("#[derive(Clone, Copy, Debug, PartialEq)]\npub struct FontRole {\n    pub family: &'static str,\n    pub weight: u16,\n    pub stretch: u16,\n    pub tracking: f32,\n    pub upper: bool,\n}\n\n");
    for role in FONT_ROLES {
        writeln!(
            o,
            "pub const FONT_{}: FontRole = FontRole {{ family: {:?}, weight: {}, stretch: {}, tracking: {:?}, upper: {} }};",
            upper_snake(role),
            t.str(&format!("font.{role}.family"))?,
            t.num(&format!("font.{role}.weight"))? as u16,
            t.num(&format!("font.{role}.stretch"))? as u16,
            t.num(&format!("font.{role}.tracking"))? as f32,
            t.str(&format!("font.{role}.case"))? == "upper",
        )
        .unwrap();
    }
    o.push('\n');
    for (k, v) in t.table("size.app")? {
        let n = v.as_float().or_else(|| v.as_integer().map(|i| i as f64)).ok_or("tokens: size.app is not numbers")?;
        writeln!(o, "pub const SIZE_{}: f32 = {:?};", upper_snake(&k), n as f32).unwrap();
    }
    o.push('\n');
    for (k, v) in t.table("space")? {
        let n = v.as_integer().ok_or("tokens: space is not whole numbers")?;
        writeln!(o, "pub const SPACE_{}: f32 = {:?};", upper_snake(k.trim_start_matches('s')), n as f32).unwrap();
    }
    o.push('\n');
    writeln!(o, "pub const RADIUS: f32 = {:?};", t.num("shape.radius")? as f32).unwrap();
    writeln!(o, "pub const LINE: f32 = {:?};", t.num("shape.line")? as f32).unwrap();
    writeln!(o, "pub const FOCUS_RING: f32 = {:?};", t.num("shape.focus_ring")? as f32).unwrap();
    writeln!(o, "pub const FOCUS_OFFSET: f32 = {:?};", t.num("shape.focus_offset")? as f32).unwrap();
    writeln!(o, "pub const GUTTER_NARROW: f32 = {:?};", t.num("shape.gutter_narrow")? as f32).unwrap();
    writeln!(o, "pub const GUTTER_WIDE: f32 = {:?};", t.num("shape.gutter_wide")? as f32).unwrap();
    writeln!(o, "pub const COLUMNS: usize = {};", t.num("shape.columns")? as usize).unwrap();
    o.push('\n');
    for k in ["fast", "quick", "base", "slow", "hover"] {
        writeln!(o, "pub const MOTION_{}: f32 = {:?};", upper_snake(k), (t.num(&format!("motion.{k}"))? / 1000.0) as f32).unwrap();
    }
    for k in ["ease_out", "ease_in"] {
        let arr = t.get(&format!("motion.{k}"))?.as_array().ok_or("tokens: an easing is four numbers")?;
        let v: Vec<String> = arr
            .iter()
            .map(|x| format!("{:?}", x.as_float().or_else(|| x.as_integer().map(|i| i as f64)).unwrap_or(0.0) as f32))
            .collect();
        writeln!(o, "pub const {}: [f32; 4] = [{}];", upper_snake(k), v.join(", ")).unwrap();
    }
    writeln!(o, "\npub const BRAND: &str = {:?};", t.str("meta.name")?).unwrap();
    writeln!(o, "pub const BRAND_URL: &str = {:?};", t.str("meta.url")?).unwrap();
    Ok(o)
}

fn css_name(n: &str) -> String {
    n.replace('_', "-")
}

fn css_colors(t: &Tokens, cast: &str, o: &mut String) -> Result<(), String> {
    for n in COLOR_NAMES {
        writeln!(o, "  --{}: {};", css_name(n), css_color(t.color(cast, n)?)).unwrap();
    }
    for (i, g) in t.groups(cast)?.into_iter().enumerate() {
        writeln!(o, "  --group-{}: {};", i + 1, css_color(g)).unwrap();
    }
    writeln!(o, "  --edge-shadow: inset 0 1px 0 var(--edge);").unwrap();
    Ok(())
}

pub fn css(t: &Tokens) -> Result<String, String> {
    let mut o = String::new();
    o.push_str("/* generated from tokens/ncf.toml by ncf tokens here--- */\n\n");
    o.push_str(":root {\n  color-scheme: dark;\n");
    css_colors(t, "dark", &mut o)?;
    let family = |role: &str| -> Result<String, String> { t.str(&format!("font.{role}.family")) };
    writeln!(o, "  --display: \"{}\", ui-sans-serif, system-ui, sans-serif;", family("display")?).unwrap();
    writeln!(o, "  --font: \"{}\", ui-sans-serif, system-ui, -apple-system, \"Segoe UI\", sans-serif;", family("ui")?).unwrap();
    writeln!(o, "  --mono: \"{}\", ui-monospace, \"SF Mono\", Menlo, monospace;", family("hud")?).unwrap();
    for role in FONT_ROLES {
        writeln!(o, "  --{role}-weight: {};", t.num(&format!("font.{role}.weight"))?).unwrap();
        writeln!(o, "  --{role}-stretch: {}%;", t.num(&format!("font.{role}.stretch"))?).unwrap();
        writeln!(o, "  --{role}-tracking: {}em;", t.num(&format!("font.{role}.tracking"))?).unwrap();
    }
    for (k, v) in t.table("size.web")? {
        let name = if k.starts_with('t') && k.len() == 2 { format!("t-{}", &k[1..]) } else { format!("t-{k}") };
        writeln!(o, "  --{name}: {};", v.as_str().ok_or("tokens: size.web is not text")?).unwrap();
    }
    for (k, v) in t.table("space")? {
        writeln!(o, "  --s-{}: {}px;", k.trim_start_matches('s'), v.as_integer().unwrap_or(0)).unwrap();
    }
    writeln!(o, "  --r-sm: {}px;", t.num("shape.radius")?).unwrap();
    writeln!(o, "  --focus-ring: {}px;", t.num("shape.focus_ring")?).unwrap();
    writeln!(o, "  --focus-offset: {}px;", t.num("shape.focus_offset")?).unwrap();
    writeln!(o, "  --gutter: {}px;", t.num("shape.gutter_narrow")?).unwrap();
    writeln!(o, "  --columns: {};", t.num("shape.columns")?).unwrap();
    writeln!(o, "  --vp-wide: 1440px;").unwrap();
    for k in ["fast", "quick", "base", "slow"] {
        let ms = t.num(&format!("motion.{k}"))?;
        writeln!(o, "  --m-{}: {}ms;", ms, ms).unwrap();
    }
    for k in ["ease_out", "ease_in"] {
        let arr = t.get(&format!("motion.{k}"))?.as_array().ok_or("tokens: an easing is four numbers")?;
        let v: Vec<String> = arr
            .iter()
            .map(|x| x.as_float().or_else(|| x.as_integer().map(|i| i as f64)).unwrap_or(0.0).to_string())
            .collect();
        writeln!(o, "  --{}: cubic-bezier({});", css_name(k), v.join(", ")).unwrap();
    }
    o.push_str("}\n\n");
    writeln!(o, "@media (min-width: 768px) {{ :root {{ --gutter: {}px; }} }}\n", t.num("shape.gutter_wide")?).unwrap();
    o.push_str("[data-theme=\"light\"] {\n  color-scheme: light;\n");
    css_colors(t, "light", &mut o)?;
    o.push_str("}\n");
    Ok(o)
}

pub fn luminance(c: Rgba) -> f64 {
    let ch = |v: u8| {
        let s = v as f64 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(c.0) + 0.7152 * ch(c.1) + 0.0722 * ch(c.2)
}

pub fn contrast(a: Rgba, b: Rgba) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}

pub const TOKENS_PATH: &str = "tokens/ncf.toml";
pub const RUST_PATH: &str = "crates/ncf/src/tokens.rs";
pub const CSS_PATH: &str = "web/ncf-tokens.css";

pub fn root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

pub fn generate() -> Result<(String, String), String> {
    let text = std::fs::read_to_string(root().join(TOKENS_PATH)).map_err(|e| format!("{TOKENS_PATH}: {e}"))?;
    let t = Tokens::load(&text)?;
    Ok((rust(&t)?, css(&t)?))
}

pub fn write() -> Result<(), String> {
    let (r, c) = generate()?;
    std::fs::write(root().join(RUST_PATH), r).map_err(|e| e.to_string())?;
    std::fs::write(root().join(CSS_PATH), c).map_err(|e| e.to_string())?;
    Ok(())
}
