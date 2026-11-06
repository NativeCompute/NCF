use eframe::egui::{self, pos2, Key, Modifiers, Pos2};

use crate::icons::Icon;
use crate::keys::{self, Command, Keymap};
use crate::mark::{self, AppMark};
use crate::settings::Look;
use crate::testing::Harness;
use crate::theme::{self, pal};
use crate::{tokens, ui, update};

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn lock() -> std::sync::MutexGuard<'static, ()> {
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
    assert_eq!(k.label(Cmd::Save, false), format!("{}+S", crate::OS.command_key()));
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

#[test]
fn shortcut_labels_name_the_keys_each_platform_has() {
    use crate::Os;
    let c = keys::parse("Ctrl+Alt+Shift+F").unwrap();
    assert_eq!(keys::label_for(&c, Os::Linux), "Ctrl+Alt+Shift+F");
    assert_eq!(keys::label_for(&c, Os::Windows), "Ctrl+Alt+Shift+F");
    assert_eq!(keys::label_for(&c, Os::Mac), "Cmd+Option+Shift+F");
    assert_eq!(keys::text(&c), "Ctrl+Alt+Shift+F", "the settings file spells it the same on every platform");
    assert_eq!(keys::parse("Cmd+Option+F"), keys::parse("Ctrl+Alt+F"));
    let fonts = [include_bytes!("../fonts/Anybody-Regular.ttf").as_slice(), include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice()];
    let label = keys::label_for(&c, Os::Mac);
    assert!(label.is_ascii(), "plain letters only: the bundled fonts have no Mac key symbols");
    assert!(fonts.iter().all(|f| f.len() > 1000));
}

#[test]
fn the_mac_keeps_its_own_shortcuts_and_option_types() {
    use crate::Os;
    let p = |s: &str| keys::parse(s).unwrap();
    for taken in ["Cmd+Q", "Cmd+H", "Cmd+Tab", "Cmd+Space", "Cmd+`", "Cmd+Shift+4"] {
        assert_eq!(keys::refused_for(&p(taken), false, Os::Mac), Some("macOS keeps that one for itself."), "{taken}");
        assert_eq!(keys::refused_for(&p(taken), false, Os::Linux).filter(|w| w.contains("macOS")), None, "{taken} is fine elsewhere");
    }
    assert!(keys::refused_for(&p("Option+1"), false, Os::Mac).unwrap().contains("types a character"));
    assert!(keys::refused_for(&p("Option+N"), false, Os::Mac).is_some());
    assert_eq!(keys::refused_for(&p("Alt+1"), false, Os::Linux), None);
    assert_eq!(keys::refused_for(&p("Cmd+Option+F"), false, Os::Mac), None);
    assert_eq!(keys::refused_for(&p("Cmd+Shift+H"), false, Os::Mac), None);
    assert!(keys::refused_for(&p("Option+Up"), false, Os::Mac).unwrap().starts_with("Option+Up"));
    assert!(keys::refused_for(&p("F"), false, Os::Mac).unwrap().contains("Cmd or Option"));
}

#[test]
fn a_mac_draws_with_metal_first_and_opengl_after() {
    let order = crate::app::renderers();
    assert_eq!(order.last(), Some(&eframe::Renderer::Glow), "OpenGL is always there to fall back on");
    if cfg!(target_os = "macos") {
        assert_eq!(order.len(), 2);
        assert_ne!(order[0], eframe::Renderer::Glow, "Metal comes first on a Mac");
    } else {
        assert_eq!(order, vec![eframe::Renderer::Glow]);
    }
}

#[test]
fn a_mac_window_uses_the_native_frame_and_fits_the_screen() {
    use crate::{chrome, splash, Os};
    use eframe::egui::Vec2;
    assert!(chrome::native_frame(Os::Mac));
    assert!(!chrome::native_frame(Os::Linux) && !chrome::native_frame(Os::Windows));
    assert_eq!(chrome::title_inset(Os::Mac, false), chrome::TRAFFIC_LIGHTS);
    assert_eq!(chrome::title_inset(Os::Mac, true), 2.0, "in full screen the traffic lights are hidden");
    assert_eq!(chrome::title_inset(Os::Linux, false), 2.0);
    let want = Vec2::new(1320.0, 900.0);
    let air = chrome::fit(want, Some(Vec2::new(1440.0, 900.0)), Os::Mac);
    assert!(air.y <= 760.0 && air.x == 1320.0, "{air:?}");
    assert_eq!(chrome::fit(want, Some(Vec2::new(2560.0, 1440.0)), Os::Mac), want);
    assert_eq!(chrome::fit(want, None, Os::Mac), want);
    let at = splash::centred(Vec2::new(1440.0, 900.0), air, Os::Mac);
    assert!(at.y >= 38.0, "below the menu bar: {at:?}");
    assert!(!splash::own_window(Os::Mac) && splash::own_window(Os::Linux));
    assert_eq!(crate::app::first_size(true, Os::Mac), (crate::app::SIZE, crate::app::MIN_SIZE));
    assert_eq!(crate::app::first_size(true, Os::Linux), (splash::CARD, splash::CARD));
    assert_eq!(chrome::clear_color(Os::Linux, eframe::egui::Color32::WHITE), [0.0; 4]);
    assert_eq!(chrome::clear_color(Os::Mac, eframe::egui::Color32::WHITE)[3], 1.0);
}

#[test]
fn app_folders_follow_each_platforms_custom() {
    use crate::{platform, Os};
    assert_eq!(platform::folder_name("Field Notes", Os::Linux), "field notes");
    assert_eq!(platform::folder_name("Field Notes", Os::Windows), "Field Notes");
    assert_eq!(platform::folder_name("Field Notes", Os::Mac), "Field Notes");
}

#[test]
fn a_mac_alert_is_safe_applescript() {
    use crate::platform::{alert_script, applescript_text};
    assert_eq!(applescript_text("a \"b\"\\c"), "\"a \\\"b\\\"\\\\c\"");
    assert_eq!(applescript_text("one\ntwo"), "\"one\" & return & \"two\"");
    let script = alert_script("App", "Line one\nIt said \"no\"");
    assert!(script.starts_with("display alert \"App\" message \"Line one\" & return & \"It said \\\"no\\\"\""), "{script}");
    assert!(script.ends_with("as critical buttons {\"OK\"} default button \"OK\""));
}

#[test]
fn the_mac_icon_file_holds_every_size_as_png() {
    let icns = crate::mark::NATIVECOMPUTE.icns();
    assert_eq!(&icns[0..4], b"icns");
    assert_eq!(u32::from_be_bytes(icns[4..8].try_into().unwrap()) as usize, icns.len());
    let mut at = 8;
    let mut seen = Vec::new();
    while at < icns.len() {
        let kind = String::from_utf8(icns[at..at + 4].to_vec()).unwrap();
        let len = u32::from_be_bytes(icns[at + 4..at + 8].try_into().unwrap()) as usize;
        let png = &icns[at + 8..at + len];
        assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n", "{kind}");
        let decoder = png::Decoder::new(std::io::Cursor::new(png));
        let reader = decoder.read_info().unwrap();
        let info = reader.info();
        assert_eq!(info.width, info.height);
        seen.push((kind, info.width));
        at += len;
    }
    assert_eq!(at, icns.len());
    let want = [("icp4", 16), ("icp5", 32), ("ic11", 32), ("ic12", 64), ("ic07", 128), ("ic13", 256), ("ic08", 256), ("ic14", 512), ("ic09", 512), ("ic10", 1024)];
    assert_eq!(seen, want.iter().map(|(k, s)| (k.to_string(), *s)).collect::<Vec<_>>());
}

#[test]
fn the_mac_info_plist_has_what_finder_and_retina_need() {
    let info = crate::AppInfo {
        name: "Field & Notes",
        id: "field-notes",
        version: "1.2.3",
        tagline: "",
        repo: "",
        mark: crate::mark::NATIVECOMPUTE,
    };
    let plist = crate::bundle::info_plist(&info, "field-notes", "Field Notes");
    for (k, v) in [
        ("CFBundleExecutable", "<string>field-notes</string>"),
        ("CFBundleIdentifier", "<string>net.nativecompute.field-notes</string>"),
        ("CFBundleShortVersionString", "<string>1.2.3</string>"),
        ("LSMinimumSystemVersion", "<string>11.0</string>"),
        ("CFBundleName", "<string>Field &amp; Notes</string>"),
        ("CFBundlePackageType", "<string>APPL</string>"),
        ("NSHighResolutionCapable", "<true/>"),
    ] {
        let at = plist.find(&format!("<key>{k}</key>")).unwrap_or_else(|| panic!("{k} missing"));
        assert!(plist[at..].lines().nth(1).unwrap().trim() == v, "{k}");
    }
    assert!(plist.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
    assert_eq!(plist.matches("<key>").count(), plist.matches("</key>").count());
    assert!(!plist.contains("CFBundleDocumentTypes"), "no file types claimed until opening from Finder works");
}

#[test]
fn the_updater_window_draws_every_step_in_both_casts() {
    use crate::updater::work::{Event, Step};
    use crate::updater::{Install, Plan};
    let _g = lock();
    let plan = Plan {
        name: "Field Notes".into(),
        slug: "field-notes".into(),
        repo: "NativeCompute/field-notes".into(),
        version: "1.4.0".into(),
        source: "file:///nowhere".into(),
        install: Install::LinuxLocal { bin: std::path::PathBuf::from("/nowhere/field-notes") },
        data: std::env::temp_dir().join(format!("ncf-updater-ui-{}", std::process::id())),
        parent: 0,
        headless: false,
        reopen: false,
    };
    let info = crate::AppInfo { name: "Field Notes", id: "net.nativecompute.fieldnotes", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::AppMark { name: "Field Notes", parts: &[] } };
    for dark in [true, false] {
        theme::set_palette(dark);
        let mut h = Harness::new(540.0, 340.0);
        theme::apply(&h.ctx);
        let mut window = crate::updater::window::Updater::new(info, plan.clone(), None, std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)));
        for step in crate::updater::work::STEPS {
            window.state.apply(Event::Step(step));
            if step == Step::Download {
                window.state.apply(Event::Progress(0.42));
            }
            window.state.apply(Event::Line(format!("{step:?} detail")));
            h.frames(2, |ctx| window.ui(ctx));
        }
        window.state.apply(Event::Failed("the download is damaged (its checksum does not match). Nothing was changed".into()));
        h.frames(2, |ctx| window.ui(ctx));
        assert!(window.state.finished());
    }
    let _ = std::fs::remove_dir_all(&plan.data);
}

#[test]
fn debug_mode_opens_only_on_five_held_knocks_within_three_seconds() {
    use crate::debug::Debug;
    let mut d = Debug::default();
    for k in 0..4 {
        assert!(!d.count(true, k as f64 * 0.5), "not before the fifth knock");
    }
    assert!(d.count(true, 2.1));
    assert!(d.on && d.open, "on, with its panel open");
    for k in 0..5 {
        d.count(true, 10.0 + k as f64 * 0.2);
    }
    assert!(!d.on && !d.open, "the same knock turns it off");

    let mut slow = Debug::default();
    for k in 0..8 {
        assert!(!slow.count(true, k as f64 * 1.0), "knocks spread over more than three seconds never add up");
    }
    let mut loose = Debug::default();
    for k in 0..10 {
        assert!(!loose.count(k % 2 == 0, k as f64 * 0.1), "letting go of the keys starts the count again");
    }
    assert!(!loose.on);
}

#[test]
fn the_settings_file_only_mentions_debug_mode_while_it_is_on() {
    let off = Look::default();
    assert_eq!(off.write(), "light_mode = no\nanimations = yes\nfont_scale = 1.00\nsplash = yes\ncheck_updates = yes\n", "byte-identical to before debug mode existed");
    assert!(!off.write().contains("debug"));
    let on = Look { debug: true, ..Look::default() };
    assert!(on.write().ends_with("debug = yes\n"));
    assert_eq!(Look::parse(&on.write()), on);
    assert_eq!(Look::parse(&off.write()), off);
}

#[test]
fn the_debug_panel_draws_in_both_casts_and_its_display_switches_apply() {
    use crate::debug::{apply, Debug, Request};
    let _g = lock();
    for dark in [true, false] {
        theme::set_palette(dark);
        let mut h = Harness::new(1320.0, 900.0);
        theme::apply(&h.ctx);
        let mut d = Debug::new(true);
        d.open = true;
        let mut got = Vec::new();
        h.frames(3, |ctx| {
            got = d.panel(ctx, egui::vec2(1320.0, 900.0), |ui, out| {
                if ui::quiet_button(ui, "App trigger", None) {
                    out.push(Request::Confirm);
                }
            });
        });
        assert!(got.is_empty(), "nothing is triggered without a click");
        assert!(apply(&h.ctx, &mut d, &Request::Inspector(true)));
        assert!(d.inspector && h.ctx.style().debug.show_interactive_widgets);
        assert!(apply(&h.ctx, &mut d, &Request::WindowSize(crate::debug::LAPTOP)));
        assert!(!apply(&h.ctx, &mut d, &Request::Crash), "left to the app");
        assert!(apply(&h.ctx, &mut d, &Request::Off));
        assert!(!d.on && !d.open && !d.inspector && !h.ctx.style().debug.show_interactive_widgets);
        let mut hidden = Vec::new();
        h.frames(1, |ctx| hidden = d.panel(ctx, egui::vec2(1320.0, 900.0), |_, _| {}));
        assert!(hidden.is_empty());
    }
    assert_eq!(crate::debug::next_version("2.2.0"), "2.2.1");
    assert_eq!(crate::debug::next_version("v3"), "3.0.1");
}

