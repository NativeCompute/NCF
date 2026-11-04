use eframe::egui::{self, pos2, Key, Modifiers, Pos2};

use crate::icons::Icon;
use crate::keys::{self, Command, Keymap};
use crate::mark::{self, AppMark};
use crate::settings::Look;
use crate::testing::Harness;
use crate::theme::{self, pal};
use crate::{tokens, ui, update};

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn both_casts_come_from_the_tokens() {
    let _g = lock();
    let d = theme::palette(true);
    assert!(d.dark);
    assert_eq!(d.void, egui::Color32::BLACK);
    assert_eq!(d.signal, egui::Color32::from_rgb(0x3B, 0x4C, 0xFF));
    assert_eq!(d.signal_hi, egui::Color32::from_rgb(0x6E, 0x7B, 0xFF));
    let l = theme::palette(false);
    assert!(!l.dark);
    assert_eq!(l.signal, d.signal, "one signal blue in both casts");
    assert_ne!(l.text, d.text);
    assert_eq!(theme::RADIUS, 2.0);
    assert_eq!(tokens::BRAND_URL, "https://nativecompute.net");
}

#[test]
fn every_component_draws_in_both_casts() {
    let _g = lock();
    let mut h = Harness::new(1000.0, 800.0);
    for dark in [true, false] {
        theme::set_palette(dark);
        theme::apply(&h.ctx);
        let mut on = true;
        let mut value = 0.5f32;
        let mut text = String::from("Hello");
        h.frames(3, |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui::giant(ui, "Good morning", 34.0);
                ui::eyebrow(ui, "Your library / 4 scripts");
                ui::heading(ui, "Heading");
                ui::body(ui, "Body text");
                ui::caption(ui, "Caption");
                ui.horizontal(|ui| {
                    ui::button(ui, "Primary", Some(Icon::Plus), true);
                    ui::button(ui, "Ghost", None, false);
                    ui::quiet_button(ui, "Quiet", None);
                    ui::icon_button(ui, Icon::Settings, "Settings", 28.0);
                    ui::chip_button(ui, "Tag");
                    ui::link(ui, "nativecompute.net");
                });
                ui::tabs(ui, &["Write", "Cards", "Read"], 1);
                ui::toggle_row(ui, "Animations", "Every transition.", &mut on);
                ui::slider(ui, &mut value, 0.0..=1.0);
                ui::field(ui, &mut text, "Search", 300.0);
                ui::section(ui, "Section");
                ui::separator(ui);
                ui::swatch(ui, pal().group(2), "Colour", true, None);
                let r = ui.available_rect_before_wrap();
                theme::panel(ui.painter(), egui::Rect::from_min_size(r.min, egui::vec2(200.0, 80.0)));
                crate::chrome::column_guides(ui.painter(), r);
                mark::NATIVECOMPUTE.paint(ui.painter(), egui::Rect::from_min_size(r.min, egui::vec2(64.0, 64.0)));
            });
        });
    }
    theme::set_palette(true);
}

#[test]
fn a_button_a_toggle_and_tabs_answer_the_pointer() {
    let _g = lock();
    let mut h = Harness::new(600.0, 400.0);
    let mut rects: Vec<egui::Rect> = Vec::new();
    let mut clicked = false;
    let mut on = false;
    let mut picked = None;
    let draw = |ctx: &egui::Context, rects: &mut Vec<egui::Rect>, clicked: &mut bool, on: &mut bool, picked: &mut Option<usize>| {
        egui::CentralPanel::default().show(ctx, |ui| {
            rects.clear();
            let r = ui::button_response(ui, "Go", None, true, None);
            rects.push(r.rect);
            if r.clicked() {
                *clicked = true;
            }
            let before = ui.cursor().min;
            ui::toggle(ui, on);
            rects.push(egui::Rect::from_min_size(before, egui::vec2(36.0, 20.0)));
            let before = ui.cursor().min;
            if let Some(k) = ui::tabs(ui, &["One", "Two"], 0) {
                *picked = Some(k);
            }
            rects.push(egui::Rect::from_min_size(before, egui::vec2(200.0, 32.0)));
        });
    };
    h.frames(2, |ctx| draw(ctx, &mut rects, &mut clicked, &mut on, &mut picked));
    let (button, toggle, tabs) = (rects[0], rects[1], rects[2]);
    h.click(button.center(), |ctx| draw(ctx, &mut rects, &mut clicked, &mut on, &mut picked));
    assert!(clicked);
    h.click(toggle.center(), |ctx| draw(ctx, &mut rects, &mut clicked, &mut on, &mut picked));
    assert!(on);
    h.click(Pos2::new(tabs.left() + 72.0 * 1.5, tabs.center().y), |ctx| draw(ctx, &mut rects, &mut clicked, &mut on, &mut picked));
    assert_eq!(picked, Some(1));
}

#[test]
fn text_is_cut_to_its_measured_width() {
    let _g = lock();
    let mut h = Harness::new(400.0, 300.0);
    h.run(|ctx| {
        let font = theme::font_display(22.0);
        let long = "A TITLE FAR TOO LONG FOR ANY CARD IT COULD BE PUT ON";
        let cut = ui::elide_to_width(ctx, long, font.clone(), 0.0, 180.0);
        assert!(theme::tracked_width(ctx, &cut, font.clone(), 0.0) <= 180.0);
        assert!(cut.ends_with('\u{2026}'));
        assert_eq!(ui::elide_to_width(ctx, "SHORT", font, 0.0, 400.0), "SHORT");
    });
}

const TEST_MARK: AppMark = AppMark {
    name: "Test",
    parts: &[
        mark::Part::Capsule { from: (0.27, 0.83), to: (0.49, 0.17), width: mark::SLAB, ink: mark::Ink::Chrome },
        mark::Part::Capsule { from: (0.585, 0.83), to: (0.805, 0.17), width: mark::LINE, ink: mark::Ink::Signal },
    ],
};

#[test]
fn an_app_mark_becomes_svg_pixels_and_an_ico() {
    let svg = TEST_MARK.svg();
    assert!(svg.starts_with("<svg"));
    assert_eq!(svg.matches("<polygon").count(), 2);
    assert!(svg.contains("#3B4CFF"));
    let n = 64u32;
    let px = TEST_MARK.rgba(n);
    let at = |x: u32, y: u32| {
        let i = ((y * n + x) * 4) as usize;
        [px[i], px[i + 1], px[i + 2], px[i + 3]]
    };
    assert_eq!(at(0, 0)[3], 0, "rounded tile corner");
    assert_eq!(at(4, 32), [0, 0, 0, 255], "black tile");
    assert_eq!(at(26, 32), [0xF2, 0xF3, 0xF7, 255], "chrome slab");
    assert_eq!(at(42, 32), [0x3B, 0x4C, 0xFF, 255], "signal line");
    let ico = TEST_MARK.ico();
    assert_eq!(&ico[..4], &[0, 0, 1, 0]);
    assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 6);
}

#[test]
fn the_nativecompute_mark_fills_its_slabs() {
    for slab in crate::brand::SLABS {
        let pts: Vec<Pos2> = slab.iter().map(|(x, y)| pos2(*x * 100.0, *y * 100.0)).collect();
        let tris = mark::triangulate(&pts);
        assert_eq!(tris.len(), (pts.len() - 2) * 3, "every vertex is used once per ear");
        let area: f32 = tris
            .chunks(3)
            .map(|t| {
                let (a, b, c) = (pts[t[0] as usize], pts[t[1] as usize], pts[t[2] as usize]);
                ((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)).abs() * 0.5
            })
            .sum();
        let shoelace: f32 = (0..pts.len())
            .map(|i| {
                let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
                a.x * b.y - b.x * a.y
            })
            .sum::<f32>()
            .abs()
            * 0.5;
        assert!((area - shoelace).abs() / shoelace < 0.01, "triangles cover the slab: {area} vs {shoelace}");
    }
    assert!(mark::NATIVECOMPUTE.svg().contains("<polygon"));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cmd {
    Save,
    Find,
}

impl Command for Cmd {
    fn all() -> Vec<Self> {
        vec![Cmd::Save, Cmd::Find]
    }
    fn slug(self) -> String {
        match self {
            Cmd::Save => "save".into(),
            Cmd::Find => "find".into(),
        }
    }
    fn label(self) -> String {
        format!("{self:?}")
    }
    fn group(self) -> &'static str {
        "General"
    }
    fn defaults(self, _web: bool) -> Vec<egui::KeyboardShortcut> {
        match self {
            Cmd::Save => vec![keys::shortcut(Modifiers::COMMAND, Key::S)],
            Cmd::Find => vec![keys::shortcut(Modifiers::COMMAND, Key::F)],
        }
    }
}

#[test]
fn a_keymap_works_for_any_apps_commands() {
    let mut k: Keymap<Cmd> = Keymap::default();
    assert!(k.is_default());
    assert_eq!(k.label(Cmd::Save, false), "Ctrl+S");
    let chord = keys::parse("Ctrl+Shift+S").unwrap();
    k.set(Cmd::Save, chord, false);
    assert!(k.is_custom(Cmd::Save));
    let text = k.serialize();
    assert_eq!(text, "key.save = Ctrl+Shift+S\n");
    let mut back: Keymap<Cmd> = Keymap::default();
    for line in text.lines() {
        let (key, value) = line.split_once('=').unwrap();
        assert!(back.parse_line(key.trim(), value.trim()));
    }
    assert_eq!(back, k);
    assert_eq!(k.taken_by(&keys::parse("Ctrl+F").unwrap(), Cmd::Save, &Cmd::all(), false), Some(Cmd::Find));
}

#[test]
fn look_settings_round_trip_and_unknown_keys_are_left_alone() {
    let mut l = Look::default();
    assert!(!l.light_mode && l.animations && l.check_updates);
    l.light_mode = true;
    l.font_scale = 1.2;
    assert_eq!(Look::parse(&l.write()), l);
    assert!(!Look::default().read("youtrack_url", "x"));
    assert_eq!(Look::parse("font_scale = 9\n").font_scale, 1.4);
}

#[test]
fn the_update_check_compares_versions_and_reads_github() {
    assert!(update::newer("0.1.0", "v0.1.1"));
    assert!(update::newer("0.9.0", "0.10.0"));
    assert!(!update::newer("1.0.0", "1.0.0-rc1"));
    let r = update::parse(r#"{"tag_name":"v2.1.0","html_url":"https://github.com/NativeCompute/x/releases/tag/v2.1.0"}"#, "NativeCompute/x").unwrap();
    assert_eq!(r.version, "2.1.0");
    let r = update::parse(r#"{"tag_name":"v2.1.0"}"#, "NativeCompute/x").unwrap();
    assert_eq!(r.url, "https://github.com/NativeCompute/x/releases");
}

#[test]
fn the_splash_says_how_the_update_check_went() {
    use crate::splash::status_line;
    use update::{Release, Status};
    assert_eq!(status_line(&Status::Off), None);
    assert_eq!(status_line(&Status::Checking), Some(("CHECKING FOR UPDATES…".to_string(), false)));
    assert_eq!(status_line(&Status::Latest), Some(("UP TO DATE".to_string(), false)));
    let r = Release { version: "2.2.0".into(), url: String::new() };
    assert_eq!(status_line(&Status::Newer(r)), Some(("V2.2.0 IS OUT".to_string(), true)));
    assert!(status_line(&Status::Failed).is_some());
    assert_eq!(update::releases_latest("NativeCompute/x"), "https://github.com/NativeCompute/x/releases/latest");
}
