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
    assert_eq!(d.void, egui::Color32::from_rgb(0x0E, 0x0F, 0x13));
    assert_eq!(d.signal, egui::Color32::from_rgb(0xFF, 0x6A, 0x3D));
    assert_eq!(d.signal_hi, egui::Color32::from_rgb(0xFF, 0x9A, 0x73));
    let l = theme::palette(false);
    assert!(!l.dark);
    assert_eq!(l.void, egui::Color32::from_rgb(0xE9, 0xEB, 0xF0));
    assert_eq!(l.signal, d.signal, "one coral accent in both casts");
    assert_eq!(l.brights, d.brights, "the tile colours are the same in both casts");
    assert_eq!(l.brights[0], egui::Color32::from_rgb(0xFF, 0x7A, 0x50));
    assert_ne!(l.text, d.text);
    assert_eq!(theme::RADIUS, tokens::RADIUS_SM);
    assert_eq!([tokens::RADIUS_SM, tokens::RADIUS_MD, tokens::RADIUS_LG, tokens::RADIUS_XL], [12.0, 20.0, 28.0, 32.0]);
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

const TEST_MARK: AppMark = AppMark::drawn(
    "Test",
    &[
        mark::Part::Capsule { from: (0.27, 0.83), to: (0.49, 0.17), width: mark::SLAB, ink: mark::Ink::Chrome },
        mark::Part::Capsule { from: (0.585, 0.83), to: (0.805, 0.17), width: mark::LINE, ink: mark::Ink::Signal },
    ],
);

fn px_at(px: &[u8], n: u32, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * n + x) * 4) as usize;
    [px[i], px[i + 1], px[i + 2], px[i + 3]]
}

#[test]
fn an_app_mark_sits_on_the_bento_tile() {
    let svg = TEST_MARK.svg();
    assert!(svg.starts_with("<svg"));
    assert_eq!(svg.matches("<polygon").count(), 2);
    assert!(svg.contains("radialGradient") && svg.contains("#131318"));
    let n = 128u32;
    let px = TEST_MARK.rgba(n);
    assert_eq!(px_at(&px, n, 0, 0)[3], 0, "rounded tile corner");
    let warm = px_at(&px, n, 12, 12);
    assert!(warm[0] > 150 && warm[0] > warm[1] && warm[1] > warm[2], "coral glow in the top-left: {warm:?}");
    let ink = px_at(&px, n, 118, 118);
    assert!(ink.iter().take(3).zip([0x13u8, 0x13, 0x18]).all(|(a, b)| (*a as i32 - b as i32).abs() <= 6), "ink tile: {ink:?}");
    let ico = TEST_MARK.ico();
    assert_eq!(&ico[..4], &[0, 0, 1, 0]);
    assert_eq!(u16::from_le_bytes([ico[4], ico[5]]), 6);
    let icns = TEST_MARK.icns();
    assert_eq!(&icns[..4], b"icns");
    assert_eq!(TEST_MARK.rgba(64), TEST_MARK.rgba(64), "the same pixels every time");
}

#[test]
fn a_raster_mark_is_centred_and_padded_for_the_mac() {
    let mut rgba = Vec::new();
    for _ in 0..(40 * 20) {
        rgba.extend_from_slice(&[255, 255, 255, 255]);
    }
    let png: &'static [u8] = Box::leak(crate::icon::png_bytes(&rgba, 40, 20).into_boxed_slice());
    let m = AppMark::raster("Raster", png);
    let n = 256u32;
    let px = m.rgba(n);
    let (mut lo, mut hi) = (n, 0);
    for x in 0..n {
        let c = px_at(&px, n, x, n / 2);
        if c[0] > 200 && c[1] > 200 && c[2] > 200 {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    let width = (hi - lo + 1) as f32;
    assert!((width - n as f32 * crate::icon::BENTO_TILE.mark).abs() <= 2.0, "mark is {width}px wide");
    assert!((((lo + hi) as f32 * 0.5) - n as f32 * 0.5).abs() <= 1.0, "and centred");
    let padded = crate::icon::decode_png(&m.png_padded(1024)).unwrap();
    assert_eq!((padded.width, padded.height), (1024, 1024));
    assert_eq!(padded.at(50, 50)[3], 0.0, "macOS icons keep a clear margin");
    assert!(padded.at(512, 1024 - 92)[3] > 0.0, "with a soft shadow under the tile");
    assert!(m.svg().contains("data:image/png;base64,"));
    let back = crate::icon::decode_png(&m.png(32)).unwrap();
    assert_eq!((back.width, back.height), (32, 32));
}

#[test]
fn base64_matches_the_standard_alphabet() {
    assert_eq!(crate::icon::base64(b""), "");
    assert_eq!(crate::icon::base64(b"f"), "Zg==");
    assert_eq!(crate::icon::base64(b"fo"), "Zm8=");
    assert_eq!(crate::icon::base64(b"foo"), "Zm9v");
    assert_eq!(crate::icon::base64(&[0xfb, 0xff]), "+/8=");
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
    assert!(l.appearance == crate::settings::Appearance::System && l.animations && l.check_updates);
    l.set_appearance(crate::settings::Appearance::Light);
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
    assert_eq!(status_line(&Status::Checking), Some(("Checking for updates…".to_string(), false)));
    assert_eq!(status_line(&Status::Latest), Some(("Up to date".to_string(), false)));
    let r = Release { version: "2.2.0".into(), url: String::new() };
    assert_eq!(status_line(&Status::Newer(r)), Some(("2.2.0 is out".to_string(), true)));
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
    let fonts = [include_bytes!("../fonts/PlusJakartaSans-Regular.ttf").as_slice(), include_bytes!("../fonts/JetBrainsMono-Regular.ttf").as_slice()];
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
    let info = crate::AppInfo { name: "Field Notes", id: "net.nativecompute.fieldnotes", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::AppMark::drawn("Field Notes", &[]) };
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
    assert_eq!(off.write(), "appearance = system\nlight_mode = no\nanimations = yes\nfont_scale = 1.00\nsplash = yes\ncheck_updates = yes\n", "light_mode stays for older builds");
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
        assert_eq!(d.inspector, crate::debug::OUTLINES);
        #[cfg(debug_assertions)]
        assert!(h.ctx.style().debug.show_interactive_widgets);
        assert!(apply(&h.ctx, &mut d, &Request::WindowSize(crate::debug::LAPTOP)));
        assert!(!apply(&h.ctx, &mut d, &Request::Crash), "left to the app");
        assert!(apply(&h.ctx, &mut d, &Request::Off));
        assert!(!d.on && !d.open && !d.inspector);
        #[cfg(debug_assertions)]
        assert!(!h.ctx.style().debug.show_interactive_widgets);
        let mut hidden = Vec::new();
        h.frames(1, |ctx| hidden = d.panel(ctx, egui::vec2(1320.0, 900.0), |_, _| {}));
        assert!(hidden.is_empty());
    }
    assert_eq!(crate::debug::next_version("2.2.0"), "2.2.1");
    assert_eq!(crate::debug::next_version("v3"), "3.0.1");
}


#[test]
fn the_logo_rasterises_to_its_own_area_with_its_counters_open() {
    let area = |poly: &[(f32, f32)]| {
        (0..poly.len())
            .map(|i| {
                let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
                a.0 * b.1 - b.0 * a.1
            })
            .sum::<f32>()
            .abs()
            * 0.5
    };
    let inside = |pt: (f32, f32), poly: &[(f32, f32)]| {
        let mut odd = false;
        for i in 0..poly.len() {
            let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
            if (a.1 > pt.1) != (b.1 > pt.1) && pt.0 < a.0 + (pt.1 - a.1) / (b.1 - a.1) * (b.0 - a.0) {
                odd = !odd;
            }
        }
        odd
    };
    let logo = crate::brand::LOGO;
    let mut expected = 0.0;
    let mut counters = 0;
    for (i, poly) in logo.iter().enumerate() {
        let depth = logo.iter().enumerate().filter(|(j, other)| *j != i && inside(poly[0], other)).count();
        if depth % 2 == 1 {
            counters += 1;
            expected -= area(poly);
        } else {
            expected += area(poly);
        }
    }
    assert!(counters >= 5, "the a, e, o, p and the mark's cut leave holes: {counters}");
    for height in [12usize, 20, 139] {
        let (w, h, cov) = mark::logo_coverage(height);
        assert_eq!(h, height);
        assert_eq!(w, mark::logo_width(height as f32).ceil() as usize);
        assert!(cov.iter().all(|c| (0.0..=1.001).contains(c)), "coverage stays a fraction");
        let s = height as f32 / crate::brand::LOGO_SIZE.1;
        let filled: f32 = cov.iter().sum::<f32>() / (s * s);
        assert!((filled - expected).abs() / expected < 0.03, "{height}px: {filled} vs {expected}");
    }
}

#[test]
fn the_logo_paints_as_one_tinted_image_and_reuses_its_texture() {
    let ctx = egui::Context::default();
    for _ in 0..2 {
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let r = ui::logo(ui, 20.0, egui::Color32::WHITE);
                assert!((r.rect.width() - mark::logo_width(20.0)).abs() < 0.5);
            });
        });
    }
    let textures = ctx.tex_manager().read().allocated().filter(|(_, m)| m.name.starts_with("ncf-logo")).count();
    assert_eq!(textures, 1, "one texture per pixel height");
}

// launch here---

fn splash_frames(h: &mut Harness, s: &mut crate::splash::Splash, n: usize) -> bool {
    let mut showing = true;
    for _ in 0..n {
        h.run(|ctx| {
            crate::anim::set_enabled(true);
            showing = s.show(ctx);
        });
    }
    showing
}

#[test]
fn the_splash_waits_for_the_update_check() {
    use crate::splash::Splash;
    let _guard = lock();
    let mut h = Harness::new(900.0, 600.0);
    let info = crate::AppInfo { name: "Field Notes", id: "x", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::NATIVECOMPUTE };
    let mut s = Splash::inline(info);
    s.set_status(update::Status::Checking);
    splash_frames(&mut h, &mut s, 3);
    std::thread::sleep(std::time::Duration::from_millis(1900));
    assert!(splash_frames(&mut h, &mut s, 3), "still checking, so the splash stays");
    s.set_status(update::Status::Latest);
    splash_frames(&mut h, &mut s, 3);
    std::thread::sleep(std::time::Duration::from_millis(900));
    assert!(!splash_frames(&mut h, &mut s, 3), "the check came back, so the splash goes");
    crate::anim::set_enabled(false);
}

#[test]
fn the_splash_turns_into_the_update_prompt_and_waits_for_a_choice() {
    use crate::splash::{Choice, Offer, Splash};
    let _guard = lock();
    let mut h = Harness::new(900.0, 600.0);
    let info = crate::AppInfo { name: "Field Notes", id: "x", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::NATIVECOMPUTE };
    let mut s = Splash::inline(info);
    s.set_status(update::Status::Newer(update::Release { version: "1.4.0".into(), url: String::new() }));
    crate::anim::set_enabled(true);
    assert!(s.wants_offer());
    s.set_offer(Offer { version: "1.4.0".into(), can_update: true });
    splash_frames(&mut h, &mut s, 40);
    std::thread::sleep(std::time::Duration::from_millis(2100));
    assert!(splash_frames(&mut h, &mut s, 3), "it waits for a choice");
    assert_eq!(s.take_choice(), None);
    h.press(Key::Escape, Modifiers::NONE, |ctx| {
        crate::anim::set_enabled(true);
        s.show(ctx);
    });
    assert_eq!(s.take_choice(), Some(Choice::NotNow));
    std::thread::sleep(std::time::Duration::from_millis(500));
    assert!(!splash_frames(&mut h, &mut s, 3), "then it gets out of the way");

    let mut s = Splash::inline(info);
    s.set_offer(Offer { version: "1.4.0".into(), can_update: true });
    splash_frames(&mut h, &mut s, 40);
    h.press(Key::Enter, Modifiers::NONE, |ctx| {
        crate::anim::set_enabled(true);
        s.show(ctx);
    });
    assert_eq!(s.take_choice(), Some(Choice::UpdateNow("1.4.0".into())));
    crate::anim::set_enabled(false);
}

#[test]
fn the_update_prompt_blocks_until_answered_in_both_casts() {
    use crate::splash::{prompt, Choice, Offer};
    let _guard = lock();
    let info = crate::AppInfo { name: "Field Notes", id: "x", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::NATIVECOMPUTE };
    for dark in [true, false] {
        let mut h = Harness::new(1000.0, 700.0);
        theme::set_palette(dark);
        let offer = Offer { version: "1.4.0".into(), can_update: false };
        let mut got = None;
        h.frames(3, |ctx| got = prompt(ctx, &info, &offer));
        assert_eq!(got, None);
        let mut typed = false;
        h.press(Key::Enter, Modifiers::NONE, |ctx| {
            got = got.clone().or_else(|| prompt(ctx, &info, &offer));
            typed |= ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Key { .. })));
        });
        assert_eq!(got, Some(Choice::Download("1.4.0".into())), "a copy that can't update itself offers the download");
        assert!(!typed, "keys don't reach the app underneath");
    }
    theme::set_palette(true);
}

const _: () = assert!(crate::splash::CARD.y <= 210.0, "the splash stays compact");
const _: () = assert!(crate::splash::CARD.y + crate::splash::PROMPT_H <= 300.0, "and so does the update prompt");
const _: () = assert!(crate::updater::window::SIZE.y <= 310.0, "and the updater window");

#[test]
fn a_skipped_update_is_remembered_and_absent_until_then() {
    let look = Look::default();
    assert!(!look.write().contains("skipped_update"));
    let skipped = Look { skipped_update: "2.3.0".into(), ..Look::default() };
    assert!(skipped.write().contains("skipped_update = 2.3.0\n"));
    assert_eq!(Look::parse(&skipped.write()).skipped_update, "2.3.0");
}

#[test]
fn every_bento_font_covers_latin_and_kerns() {
    use eframe::egui::{FontData, FontDefinitions, FontFamily, FontId};
    use eframe::epaint::text::Fonts;
    let faces: [(&str, &[u8]); 8] = [
        ("PlusJakartaSans-Regular", include_bytes!("../fonts/PlusJakartaSans-Regular.ttf")),
        ("PlusJakartaSans-Label", include_bytes!("../fonts/PlusJakartaSans-Label.ttf")),
        ("PlusJakartaSans-Medium", include_bytes!("../fonts/PlusJakartaSans-Medium.ttf")),
        ("PlusJakartaSans-SemiBold", include_bytes!("../fonts/PlusJakartaSans-SemiBold.ttf")),
        ("PlusJakartaSans-Bold", include_bytes!("../fonts/PlusJakartaSans-Bold.ttf")),
        ("PlusJakartaSans-ExtraBold", include_bytes!("../fonts/PlusJakartaSans-ExtraBold.ttf")),
        ("BricolageGrotesque-Display", include_bytes!("../fonts/BricolageGrotesque-Display.ttf")),
        ("BricolageGrotesque-Heading", include_bytes!("../fonts/BricolageGrotesque-Heading.ttf")),
    ];
    for (name, bytes) in faces {
        let mut defs = FontDefinitions::empty();
        defs.font_data.insert(name.into(), FontData::from_static(bytes));
        defs.families.insert(FontFamily::Proportional, vec![name.into()]);
        let fonts = Fonts::new(1.0, 2048, defs);
        let id = FontId::proportional(40.0);
        assert!(fonts.has_glyphs(&id, "Aa Zz 0123456789 —–…·“”‘’ éñŁčß€→"), "{name} misses a glyph");
        let w = |s: &str| fonts.layout_no_wrap(s.into(), id.clone(), egui::Color32::WHITE).rect.width();
        assert!(w("AV") < w("A") + w("V") - 0.5, "{name} lost its kerning");
    }
}

#[test]
fn labels_are_jakarta_with_figures_that_line_up() {
    let ctx = egui::Context::default();
    crate::theme::install_fonts(&ctx);
    let _ = ctx.run(Default::default(), |_| {});
    let font = crate::theme::font_hud(crate::theme::T_MICRO);
    assert!(font.size > crate::theme::T_MICRO, "the label cut is lifted so mixed case reads at the old size");
    let w = |s: &str| ctx.fonts(|f| f.layout_no_wrap(s.into(), font.clone(), egui::Color32::WHITE).rect.width());
    let one = w("1");
    for d in "023456789".chars() {
        assert!((w(&d.to_string()) - one).abs() < 0.01, "{d} is not as wide as 1");
    }
    assert!((w("1,111") - w("8,808")).abs() < 0.01, "counts don't wobble as they change");
    assert!(w("il") < w("mw"), "letters stay proportional");
    assert_eq!(crate::theme::tracked_width(&ctx, "Writers room", font.clone(), 0.8), crate::theme::tracked_width(&ctx, "Writers room", font, 0.0), "labels are never letter-spaced");
}

#[test]
fn appearance_reads_old_and_new_settings_files() {
    use crate::settings::Appearance::{self, *};
    let a = |text: &str| Look::parse(text).appearance;
    assert_eq!(a(""), System, "a fresh install follows the computer");
    assert_eq!(a("light_mode = no\n"), Dark, "someone who kept dark keeps dark");
    assert_eq!(a("light_mode = yes\n"), Light);
    assert_eq!(a("appearance = system\nlight_mode = yes\n"), System);
    assert_eq!(a("light_mode = yes\nappearance = system\n"), System, "the new key wins in either order");
    assert_eq!(a("appearance = dark\nlight_mode = yes\n"), Dark);
    assert_eq!(a("appearance = sepia\nlight_mode = yes\n"), Light, "an unknown value falls back to the old key");
    for x in Appearance::ALL {
        let mut l = Look::default();
        l.set_appearance(x);
        assert_eq!(Look::parse(&l.write()).appearance, x);
        assert_eq!(Appearance::from_slug(x.slug()), Some(x));
    }
    assert!(Look { appearance: Light, ..Look::default() }.write().contains("light_mode = yes"));
    assert!(Look::default().write().contains("light_mode = no"), "older builds open dark when the choice is the computer's");
}

#[test]
fn the_cast_follows_the_choice_then_the_computer_then_dark() {
    use crate::settings::Appearance::*;
    assert!(!theme::dark_for(Light, Some(true), Some(true)));
    assert!(theme::dark_for(Dark, Some(false), Some(false)));
    assert!(!theme::dark_for(System, Some(false), Some(true)), "the computer's own answer comes first");
    assert!(theme::dark_for(System, Some(true), None));
    assert!(!theme::dark_for(System, None, Some(false)), "then the desktop settings");
    assert!(theme::dark_for(System, None, None), "and dark when nothing answers");
}

#[test]
fn a_desktop_scheme_name_says_dark_or_light() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::platform::scheme_says_dark;
        assert_eq!(scheme_says_dark("'Adwaita-dark'"), Some(true));
        assert_eq!(scheme_says_dark("BreezeDark"), Some(true));
        assert_eq!(scheme_says_dark("'Adwaita'"), Some(false));
        assert_eq!(scheme_says_dark("  "), None);
    }
}

#[test]
fn debug_mode_can_pick_each_appearance() {
    use crate::settings::Appearance;
    let _g = lock();
    let info = crate::AppInfo { name: "Field Notes", id: "x", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::NATIVECOMPUTE };
    let mut shell = crate::app::Shell::new(info);
    let h = Harness::new(400.0, 300.0);
    for a in Appearance::ALL {
        crate::app::apply_debug(&h.ctx, &mut shell, &crate::debug::Request::Appearance(a));
        assert_eq!(shell.look.appearance, a);
    }
}

fn contrast32(a: egui::Color32, b: egui::Color32) -> f32 {
    let l = |c: egui::Color32| {
        let ch = |v: u8| {
            let s = v as f32 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(c.r()) + 0.7152 * ch(c.g()) + 0.0722 * ch(c.b())
    };
    let (x, y) = (l(a), l(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[test]
fn the_wallpaper_stays_inside_the_window_and_tints_its_corners() {
    let _g = lock();
    for dark in [true, false] {
        theme::set_palette(dark);
        let p = pal();
        let rect = egui::Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(1200.0, 800.0));
        let mesh = theme::wallpaper_shape(rect, theme::R_SM);
        assert!(!mesh.vertices.is_empty());
        for v in &mesh.vertices {
            assert!(rect.expand(1.0).contains(v.pos), "a wallpaper vertex escaped the window");
        }
        let near = |q: Pos2| mesh.vertices.iter().min_by(|a, b| a.pos.distance(q).total_cmp(&b.pos.distance(q))).unwrap().color;
        let centre = near(rect.center());
        let diff = |a: egui::Color32, b: egui::Color32| (a.r() as i32 - b.r() as i32).abs().max((a.g() as i32 - b.g() as i32).abs()).max((a.b() as i32 - b.b() as i32).abs());
        let base = theme::mix(p.wall_from, p.wall_to, 0.5);
        assert!(diff(centre, base) <= 4, "the middle of the window is the plain base gradient");
        let corner = near(pos2(0.0, 0.0));
        assert!(diff(corner, p.wall_from) > 6, "the top-left corner carries the warm wash");
        assert!(corner.r() > corner.b(), "and it is warm");
    }
}

#[test]
fn surfaces_cast_soft_shadows_and_selection_reads() {
    let _g = lock();
    let h = Harness::new(400.0, 300.0);
    for dark in [true, false] {
        theme::set_palette(dark);
        theme::apply(&h.ctx);
        let v = h.ctx.style().visuals.clone();
        assert_ne!(v.window_shadow, egui::epaint::Shadow::NONE);
        assert_ne!(v.popup_shadow, egui::epaint::Shadow::NONE);
        assert_eq!(v.window_rounding, egui::Rounding::same(theme::R_LG));
        let p = pal();
        let (fill, ink) = p.selected();
        assert!(contrast32(fill, ink) >= 4.5);
        let (fill, _, ink) = ui::button_colors(true, 0.0);
        assert_eq!(ink, p.ink, "coral buttons carry dark text");
        assert!(contrast32(fill, ink) >= 4.5);
        let (fill, _, ink) = ui::button_colors(true, 1.0);
        assert!(contrast32(fill, ink) >= 4.5, "and keep it on hover");
        for i in 0..6 {
            assert!(contrast32(p.bright(i), p.on_tile) >= 4.5);
        }
    }
    theme::set_palette(true);
}
