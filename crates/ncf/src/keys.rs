use eframe::egui::{self, Align, Key, KeyboardShortcut, Layout, Rect, Sense, Stroke, Vec2};
pub use eframe::egui::Modifiers;

use crate::icons::Icon;
use crate::theme::{self, pal};
use crate::{anim, ui};

pub const DIGITS: [Key; 9] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];

pub fn shortcut(modifiers: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(modifiers, key)
}

pub fn chord(modifiers: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(
        Modifiers {
            alt: modifiers.alt,
            shift: modifiers.shift,
            command: modifiers.command || modifiers.ctrl || modifiers.mac_cmd,
            ctrl: false,
            mac_cmd: false,
        },
        key,
    )
}

pub fn text(c: &KeyboardShortcut) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if c.modifiers.command || c.modifiers.ctrl {
        parts.push("Ctrl");
    }
    if c.modifiers.alt {
        parts.push("Alt");
    }
    if c.modifiers.shift {
        parts.push("Shift");
    }
    let key = match c.logical_key {
        Key::Comma => ",",
        Key::Period => ".",
        Key::Plus => "+",
        Key::Equals => "=",
        Key::Minus => "-",
        Key::Slash => "/",
        Key::Backslash => "\\",
        Key::Semicolon => ";",
        Key::OpenBracket => "[",
        Key::CloseBracket => "]",
        Key::Backtick => "`",
        Key::Quote => "'",
        k => k.name(),
    };
    parts.push(key);
    parts.join("+")
}

pub fn parse(s: &str) -> Option<KeyboardShortcut> {
    let s = s.trim();
    let (mods, key) = match s.strip_suffix("++") {
        Some(head) => (head, "+"),
        None => match s.rsplit_once('+') {
            Some((m, k)) => (m, k),
            None => ("", s),
        },
    };
    let mut m = Modifiers::NONE;
    for part in mods.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "cmd" | "command" => m.command = true,
            "alt" | "option" => m.alt = true,
            "shift" => m.shift = true,
            _ => return None,
        }
    }
    let key = match key.trim() {
        "," => Key::Comma,
        "." => Key::Period,
        "+" => Key::Plus,
        "=" => Key::Equals,
        "-" => Key::Minus,
        "/" => Key::Slash,
        "\\" => Key::Backslash,
        ";" => Key::Semicolon,
        "[" => Key::OpenBracket,
        "]" => Key::CloseBracket,
        "`" => Key::Backtick,
        "'" => Key::Quote,
        k => {
            let mut c = k.chars();
            let capital = c.next().map(|f| f.to_ascii_uppercase().to_string() + &c.as_str().to_ascii_lowercase());
            Key::from_name(k)
                .or_else(|| capital.as_deref().and_then(Key::from_name))
                .or_else(|| Key::from_name(&k.to_ascii_uppercase()))?
        }
    };
    Some(KeyboardShortcut::new(m, key))
}

pub fn refused(c: &KeyboardShortcut, web: bool) -> Option<&'static str> {
    let m = c.modifiers;
    let k = c.logical_key;
    let function = matches!(
        k,
        Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F6 | Key::F7 | Key::F8 | Key::F9 | Key::F10 | Key::F11 | Key::F12
    );
    if !(m.command || m.ctrl || m.alt || function) {
        return Some("Add Ctrl or Alt: a plain key is for typing.");
    }
    if matches!(k, Key::Escape | Key::Tab | Key::Backspace | Key::Delete) {
        return Some("That key belongs to the page.");
    }
    if m.alt && !(m.command || m.ctrl) && matches!(k, Key::ArrowUp | Key::ArrowDown) {
        return Some("Alt+Up and Down move a block in the script.");
    }
    if (m.command || m.ctrl) && !m.alt && !m.shift && matches!(k, Key::C | Key::V | Key::X | Key::A) {
        return Some("That is copy, cut, paste or select all.");
    }
    if web {
        let ctrl = m.command || m.ctrl;
        let browser = ctrl
            && !m.alt
            && (matches!(k, Key::N | Key::T | Key::W | Key::Tab | Key::PageUp | Key::PageDown)
                || (!m.shift && DIGITS.contains(&k))
                || (!m.shift && matches!(k, Key::Num8 | Key::Num9)));
        if browser || matches!(k, Key::F5 | Key::F11 | Key::F12) {
            return Some("Your browser keeps that one for itself.");
        }
    }
    None
}

pub trait Command: Copy + Eq + std::fmt::Debug + 'static {
    fn all() -> Vec<Self>;
    fn slug(self) -> String;
    fn label(self) -> String;
    fn group(self) -> &'static str;
    fn defaults(self, web: bool) -> Vec<KeyboardShortcut>;
    fn from_slug(s: &str) -> Option<Self> {
        Self::all().into_iter().find(|c| c.slug() == s)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Keymap<C: Command> {
    custom: Vec<(C, KeyboardShortcut)>,
}

impl<C: Command> Default for Keymap<C> {
    fn default() -> Self {
        Keymap { custom: Vec::new() }
    }
}

impl<C: Command> Keymap<C> {
    pub fn chords(&self, cmd: C, web: bool) -> Vec<KeyboardShortcut> {
        match self.custom.iter().find(|(c, _)| *c == cmd) {
            Some((_, k)) => vec![*k],
            None => cmd.defaults(web),
        }
    }

    pub fn label(&self, cmd: C, web: bool) -> String {
        self.chords(cmd, web).first().map(text).unwrap_or_default()
    }

    pub fn is_custom(&self, cmd: C) -> bool {
        self.custom.iter().any(|(c, _)| *c == cmd)
    }

    pub fn is_default(&self) -> bool {
        self.custom.is_empty()
    }

    pub fn set(&mut self, cmd: C, chord: KeyboardShortcut, web: bool) {
        self.custom.retain(|(c, _)| *c != cmd);
        if cmd.defaults(web).first() != Some(&chord) {
            self.custom.push((cmd, chord));
        }
    }

    pub fn reset(&mut self, cmd: C) {
        self.custom.retain(|(c, _)| *c != cmd);
    }

    pub fn reset_all(&mut self) {
        self.custom.clear();
    }

    pub fn taken_by(&self, chord: &KeyboardShortcut, except: C, commands: &[C], web: bool) -> Option<C> {
        commands
            .iter()
            .copied()
            .filter(|c| *c != except)
            .find(|c| self.chords(*c, web).contains(chord))
    }

    pub fn listen(&self, commands: &[C], web: bool) -> Vec<(C, KeyboardShortcut)> {
        let mut out: Vec<(C, KeyboardShortcut)> = commands
            .iter()
            .flat_map(|c| self.chords(*c, web).into_iter().map(move |k| (*c, k)))
            .collect();
        let weight = |k: &KeyboardShortcut| k.modifiers.alt as u8 + k.modifiers.shift as u8;
        out.sort_by_key(|(_, k)| std::cmp::Reverse(weight(k)));
        out
    }

    pub fn serialize(&self) -> String {
        self.custom
            .iter()
            .map(|(c, k)| format!("key.{} = {}\n", c.slug(), text(k)))
            .collect()
    }

    pub fn parse_line(&mut self, key: &str, value: &str) -> bool {
        let Some(slug) = key.strip_prefix("key.") else {
            return false;
        };
        if let (Some(cmd), Some(chord)) = (C::from_slug(slug.trim()), parse(value)) {
            self.custom.retain(|(c, _)| *c != cmd);
            self.custom.push((cmd, chord));
        }
        true
    }
}

// capture here---

#[derive(Clone, Debug, PartialEq)]
pub struct Capture<C: Command> {
    pub listening: Option<C>,
    pub refusal: Option<(C, String)>,
}

impl<C: Command> Default for Capture<C> {
    fn default() -> Self {
        Capture { listening: None, refusal: None }
    }
}

impl<C: Command> Capture<C> {
    pub fn poll(&mut self, ctx: &egui::Context, keymap: &mut Keymap<C>, commands: &[C], web: bool) -> bool {
        let Some(cmd) = self.listening else {
            return false;
        };
        let pressed = ctx.input_mut(|i| {
            let mut found = None;
            i.events.retain(|e| match e {
                egui::Event::Key { key, pressed: true, modifiers, .. } => {
                    if found.is_none() {
                        found = Some((*key, *modifiers));
                    }
                    false
                }
                egui::Event::Key { .. } | egui::Event::Text(_) => false,
                _ => true,
            });
            found
        });
        let Some((key, mods)) = pressed else {
            return false;
        };
        if key == Key::Escape && !(mods.ctrl || mods.command || mods.alt || mods.shift) {
            self.listening = None;
            self.refusal = None;
            return false;
        }
        let c = chord(mods, key);
        if let Some(why) = refused(&c, web) {
            self.refusal = Some((cmd, format!("{}: {why}", text(&c))));
            return false;
        }
        if let Some(other) = keymap.taken_by(&c, cmd, commands, web) {
            self.refusal = Some((cmd, format!("{} is already {}. Give that one another shortcut first.", text(&c), other.label())));
            return false;
        }
        keymap.set(cmd, c, web);
        self.listening = None;
        self.refusal = None;
        true
    }
}

pub fn fired<C: Command>(ctx: &egui::Context, keymap: &Keymap<C>, commands: &[C], web: bool) -> Vec<C> {
    let mut out = Vec::new();
    for (cmd, chord) in keymap.listen(commands, web) {
        if ctx.input_mut(|i| i.consume_shortcut(&chord)) && !out.contains(&cmd) {
            out.push(cmd);
        }
    }
    out
}

// keyboard settings here---

pub fn settings_ui<C: Command>(ui: &mut egui::Ui, keymap: &mut Keymap<C>, capture: &mut Capture<C>, commands: &[C], web: bool) -> bool {
    let p = pal();
    let mut changed = false;
    ui::caption(ui, "Click a shortcut, then press the keys you want it to be. Esc cancels.");
    ui.add_space(6.0);
    if !keymap.is_default() && ui::button(ui, "Reset all to default", Some(Icon::Refresh), false) {
        keymap.reset_all();
        capture.listening = None;
        capture.refusal = None;
        changed = true;
    }
    ui.add_space(4.0);
    let mut groups: Vec<&'static str> = Vec::new();
    for c in commands {
        if !groups.contains(&c.group()) {
            groups.push(c.group());
        }
    }
    for group in groups {
        ui::section(ui, group);
        for cmd in commands.iter().copied().filter(|c| c.group() == group) {
            let listening = capture.listening == Some(cmd);
            let custom = keymap.is_custom(cmd);
            ui.horizontal(|ui| {
                ui.set_min_height(28.0);
                ui.label(egui::RichText::new(cmd.label()).font(theme::font(theme::T_SM)).color(p.text));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    if custom {
                        let back = cmd.defaults(web).first().map(text).unwrap_or_default();
                        if ui::icon_button(ui, Icon::Refresh, &format!("Back to {back}"), 24.0) {
                            keymap.reset(cmd);
                            capture.refusal = None;
                            changed = true;
                        }
                    }
                    let chords: Vec<String> = keymap.chords(cmd, web).iter().map(text).collect();
                    let label = if listening { "Press keys…".to_string() } else { chords.join("  or  ") };
                    if keycap(ui, &label, listening, custom) {
                        capture.listening = if listening { None } else { Some(cmd) };
                        capture.refusal = None;
                    }
                });
            });
            if let Some((c, why)) = &capture.refusal {
                if *c == cmd {
                    ui.label(egui::RichText::new(why).font(theme::font(theme::T_CAP)).color(p.warn));
                }
            }
        }
        ui.add_space(4.0);
    }
    changed
}

pub fn keycap(ui: &mut egui::Ui, text: &str, listening: bool, custom: bool) -> bool {
    let p = pal();
    let font = theme::font_mono(theme::T_CAP);
    let g = ui.painter().layout_no_wrap(text.to_string(), font, p.text);
    let size = Vec2::new(g.rect.width() + 20.0, 24.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
    let lit = anim::ease(ui.ctx(), resp.id.with("lit"), listening, anim::HOVER);
    let fill = theme::mix(p.sunken, theme::mix(p.sunken, p.signal, 0.25), lit.max(0.4 * hot));
    ui.painter().rect_filled(rect, egui::Rounding::same(6.0), fill);
    ui.painter().line_segment(
        [
            rect.left_bottom() + Vec2::new(5.0, -0.5),
            rect.right_bottom() + Vec2::new(-5.0, -0.5),
        ],
        Stroke::new(1.0_f32, theme::wash(p.line_strong, 160)),
    );
    let edge = if listening {
        theme::wash(p.signal_hi, 200)
    } else if custom {
        theme::wash(p.signal_hi, 110)
    } else {
        theme::wash(p.line_strong, 140)
    };
    ui.painter().rect_stroke(rect, egui::Rounding::same(6.0), Stroke::new(1.0_f32, edge));
    let ink = if listening {
        p.signal_hi
    } else if custom {
        theme::mix(p.text, p.signal_hi, 0.35)
    } else {
        theme::mix(p.text_2, p.text, hot)
    };
    let at = Rect::from_center_size(rect.center(), g.rect.size()).min;
    ui.painter().galley(at, g, ink);
    if listening {
        anim::keep_going(ui.ctx());
    }
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(if listening { "Press the keys · Esc cancels" } else { "Click, then press the keys you want" })
        .clicked()
}
