use ncf::app::{self, Shell};
use ncf::mark;
use ncf::prelude::*;
use ncf::tokens;

use egui::{Align, Layout, Pos2, Rect, Sense, Stroke, Vec2};

pub const MARK: AppMark = AppMark {
    name: "Gallery",
    parts: &[slab((0.24, 0.83), (0.46, 0.17)), line((0.56, 0.83), (0.78, 0.17)), line((0.7, 0.83), (0.92, 0.17))],
};

pub const INFO: AppInfo = AppInfo {
    name: "NCF Gallery",
    id: "net.nativecompute.NcfGallery",
    version: ncf::VERSION,
    tagline: "Every NCF component",
    repo: "",
    mark: MARK,
};

const SECTIONS: [(&str, Icon); 8] = [
    ("Type", Icon::Heading),
    ("Controls", Icon::Check),
    ("Surfaces", Icon::Layers),
    ("Colour", Icon::Palette),
    ("Icons", Icon::Grid),
    ("Marks", Icon::Star4),
    ("Notices", Icon::Info),
    ("Motion", Icon::Refresh),
];

struct Gallery {
    section: usize,
    name: String,
    note: String,
    on: bool,
    off: bool,
    level: f32,
    seg: usize,
    tab: usize,
    menu_open: bool,
    menu_pick: String,
    opened_on: u64,
    moved: bool,
}

impl Gallery {
    fn new(section: usize) -> Gallery {
        Gallery {
            section,
            name: "Ledger".into(),
            note: String::new(),
            on: true,
            off: false,
            level: 0.6,
            seg: 0,
            tab: 1,
            menu_open: false,
            menu_pick: "Newest first".into(),
            opened_on: 0,
            moved: false,
        }
    }
}

fn block(ui: &mut egui::Ui, title: &str, call: &str, body: impl FnOnce(&mut egui::Ui)) {
    let p = pal();
    ui.horizontal(|ui| {
        ui::hud(ui, title, Some(p.text_2));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(egui::RichText::new(call).font(theme::font_mono(theme::T_MICRO + 1.0)).color(p.text_3));
        });
    });
    ui.add_space(8.0);
    body(ui);
    ui.add_space(14.0);
    ui::separator(ui);
    ui.add_space(10.0);
}

impl NcfApp for Gallery {
    fn title_extras(&mut self, ui: &mut egui::Ui, shell: &mut Shell) {
        let light = shell.look.light_mode;
        if ui::icon_button(ui, if light { Icon::Moon } else { Icon::Sun }, if light { "Dark" } else { "Light" }, 28.0) {
            shell.set_light(!light);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, shell: &mut Shell) {
        let full = ui.available_rect_before_wrap();
        let nav = Rect::from_min_size(full.min + Vec2::new(0.0, 12.0), Vec2::new(196.0, full.height() - 12.0));
        let mut n = ui.new_child(egui::UiBuilder::new().max_rect(nav).layout(Layout::top_down(Align::Min)));
        ui::eyebrow(&mut n, &format!("NCF / V{}", ncf::VERSION));
        n.add_space(10.0);
        if let Some(k) = ui::vertical_tabs(&mut n, &SECTIONS, self.section) {
            self.section = k;
        }
        let maker = Rect::from_min_max(Pos2::new(nav.left(), nav.bottom() - 24.0), nav.right_bottom());
        if ui::maker_line(&mut n, maker, false, "NCF") {
            shell.open_url(ncf::NATIVECOMPUTE_URL);
        }

        let pane = Rect::from_min_max(Pos2::new(nav.right() + 28.0, full.top() + 12.0), full.max);
        let mut c = ui.new_child(egui::UiBuilder::new().max_rect(pane).layout(Layout::top_down(Align::Min)));
        let (name, _) = SECTIONS[self.section];
        ui::eyebrow(&mut c, &format!("Gallery / {}", if pal().dark { "dark" } else { "light" }));
        ui::giant_fit(&mut c, &name.to_uppercase(), 46.0, 28.0, pane.width());
        c.add_space(14.0);
        egui::ScrollArea::vertical().id_salt(("gallery", self.section)).auto_shrink([false, false]).show(&mut c, |ui| {
            ui.set_max_width(ui.available_width() - 18.0);
            match self.section {
                0 => self.type_page(ui),
                1 => self.controls_page(ui, shell),
                2 => self.surfaces_page(ui),
                3 => self.colour_page(ui),
                4 => self.icons_page(ui),
                5 => self.marks_page(ui),
                6 => self.notices_page(ui, shell),
                _ => self.motion_page(ui),
            }
        });
    }

    fn answer(&mut self, action: &str, _text: Option<String>, shell: &mut Shell) {
        if action == "delete" {
            shell.ok("Deleted", "The item went to the bin. Nothing else changed.");
        }
    }
}

// pages here---

impl Gallery {
    fn type_page(&mut self, ui: &mut egui::Ui) {
        block(ui, "Display", "ui::giant(ui, text, size)", |ui| {
            ui::giant(ui, "NATIVECOMPUTE", 54.0);
        });
        block(ui, "Eyebrow", "ui::eyebrow(ui, text)", |ui| {
            ui::eyebrow(ui, "Ledger / 12 items");
        });
        block(ui, "Heading", "ui::heading(ui, text)", |ui| {
            ui::heading(ui, "Recent work");
        });
        block(ui, "Body", "ui::body(ui, text)", |ui| {
            ui::body(ui, "Body copy is short and plain. It says what happened and what to do next, then stops.");
        });
        block(ui, "Caption", "ui::caption(ui, text)", |ui| {
            ui::caption(ui, "Saved 2 minutes ago · 14 KB");
        });
        block(ui, "HUD", "ui::hud(ui, text, None)", |ui| {
            ui.horizontal(|ui| {
                ui::hud(ui, "V0.1.0 / STABLE", None);
                ui.add_space(16.0);
                ui::hud(ui, "SIGNAL", Some(pal().signal_hi));
            });
        });
        block(ui, "Scale", "theme::T_TITLE … T_MICRO", |ui| {
            let p = pal();
            for (label, size) in [("Title 26", theme::T_TITLE), ("Heading 17", theme::T_H), ("Body 14.5", theme::T_BODY), ("Small 13", theme::T_SM), ("Label 12", theme::T_LABEL), ("Caption 11", theme::T_CAP), ("Micro 10", theme::T_MICRO)] {
                ui.label(egui::RichText::new(label).font(theme::font_semi(size)).color(p.text));
            }
        });
        block(ui, "Fit to width", "ui::elide_to_width(ctx, text, font, tracking, width)", |ui| {
            let long = "A very long project title that would never fit in a card this narrow";
            let fitted = ui::elide_to_width(ui.ctx(), long, theme::font_semi(theme::T_BODY), 0.0, 260.0);
            ui.label(egui::RichText::new(fitted).font(theme::font_semi(theme::T_BODY)).color(pal().text));
        });
    }

    fn controls_page(&mut self, ui: &mut egui::Ui, shell: &mut Shell) {
        block(ui, "Buttons", "ui::button(ui, label, icon, accent) · ui::quiet_button", |ui| {
            ui.horizontal(|ui| {
                ui::button(ui, "New item", Some(Icon::Plus), true);
                ui::button(ui, "Export", Some(Icon::Download), false);
                ui::button(ui, "Cancel", None, false);
                ui::quiet_button(ui, "Skip", None);
            });
        });
        block(ui, "Icon buttons", "ui::icon_button(ui, icon, tip, 28.0)", |ui| {
            ui.horizontal(|ui| {
                for (icon, tip) in [(Icon::Search, "Search"), (Icon::Settings, "Settings"), (Icon::Trash, "Delete"), (Icon::More, "More")] {
                    ui::icon_button(ui, icon, tip, 28.0);
                }
            });
        });
        block(ui, "Ribbon", "ui::ribbon_button(ui, icon, label, tip, on)", |ui| {
            ui.horizontal(|ui| {
                ui::ribbon_button(ui, Icon::Bold, "Bold", "Bold · Ctrl+B", true);
                ui::ribbon_button(ui, Icon::Italic, "Italic", "Italic · Ctrl+I", false);
                ui::ribbon_button(ui, Icon::List, "List", "List", false);
            });
        });
        block(ui, "Chips and links", "ui::chip · ui::chip_button · ui::link", |ui| {
            ui.horizontal(|ui| {
                ui::chip(ui, "Draft", None);
                ui::chip(ui, "Shared", Some(pal().group(2)));
                ui::chip_button(ui, "+ Tag");
                ui.add_space(12.0);
                if ui::link(ui, "nativecompute.net") {
                    shell.open_url(ncf::NATIVECOMPUTE_URL);
                }
            });
        });
        block(ui, "Field", "ui::field(ui, &mut text, hint, width)", |ui| {
            ui.horizontal(|ui| {
                ui::field(ui, &mut self.name, "Name", 240.0);
                ui::field(ui, &mut self.note, "Search · Ctrl+F", 240.0);
            });
        });
        block(ui, "Switches", "ui::toggle · ui::toggle_row", |ui| {
            ui::toggle_row(ui, "Check for updates", "Once a day, from GitHub releases.", &mut self.on);
            ui::toggle_row(ui, "Compact cards", "One line per item.", &mut self.off);
        });
        block(ui, "Slider", "ui::slider(ui, &mut value, 0.0..=1.0)", |ui| {
            ui.set_max_width(420.0);
            ui::slider(ui, &mut self.level, 0.0..=1.0);
        });
        block(ui, "Segmented and tabs", "ui::segmented · ui::tabs", |ui| {
            if let Some(k) = ui::segmented(ui, &[("Grid", Icon::Grid), ("List", Icon::List), ("Graph", Icon::Graph)], self.seg) {
                self.seg = k;
            }
            ui.add_space(10.0);
            if let Some(k) = ui::tabs(ui, &["Dark", "Light", "System"], self.tab) {
                self.tab = k;
            }
        });
        block(ui, "Dropdown and menu", "ui::dropdown · ui::popover_frame · ui::menu_item", |ui| {
            let resp = ui::dropdown(ui, Icon::List, &self.menu_pick, self.menu_open, 200.0);
            if resp.clicked() {
                self.menu_open = !self.menu_open;
                self.opened_on = shell.frame_no;
            }
            if self.menu_open {
                let mut rects = vec![resp.rect];
                egui::Area::new(egui::Id::new("gallery-menu")).order(egui::Order::Foreground).fixed_pos(resp.rect.left_bottom() + Vec2::new(0.0, 4.0)).show(ui.ctx(), |ui| {
                    ui::popover_frame().show(ui, |ui| {
                        ui.set_width(200.0 - 8.0);
                        for (label, icon) in [("Newest first", Icon::History), ("A to Z", Icon::List), ("Starred", Icon::Star)] {
                            if ui::menu_item(ui, &mut rects, label, icon, false) {
                                self.menu_pick = label.into();
                                self.menu_open = false;
                            }
                        }
                        if ui::menu_item(ui, &mut rects, "Clear all", Icon::Trash, true) {
                            self.menu_open = false;
                        }
                    });
                });
                if (ui::Dismisser { opened_on: self.opened_on }).should_close(ui.ctx(), shell.frame_no, &rects) {
                    self.menu_open = false;
                }
            }
        });
    }

    fn surfaces_page(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        block(ui, "Panel", "theme::panel_box() · theme::panel(painter, rect)", |ui| {
            theme::panel_box().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui::section(ui, "Where things live");
                ui::caption(ui, "~/.local/share/Ledger");
                ui.add_space(6.0);
                ui::button(ui, "Open the folder", Some(Icon::Folder), false);
            });
        });
        block(ui, "Cards", "theme::hover_surface(painter, rect, RADIUS, signal, hot, chosen)", |ui| {
            ui.horizontal(|ui| {
                for (i, title) in ["Quarterly plan", "Launch notes", "Field guide"].iter().enumerate() {
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(220.0, 108.0), Sense::click());
                    let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
                    theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, i == 1);
                    let painter = ui.painter();
                    painter.rect_filled(Rect::from_min_size(rect.min, Vec2::new(rect.width(), 2.0)), egui::Rounding::ZERO, p.group(i));
                    theme::tracked_text(painter, rect.left_top() + Vec2::new(14.0, 22.0), &format!("ITEM {:02}", i + 1), theme::font_hud(theme::T_MICRO), p.text_3, theme::HUD_TRACKING);
                    painter.text(rect.left_top() + Vec2::new(14.0, 44.0), egui::Align2::LEFT_CENTER, *title, theme::font_semi(theme::T_BODY), p.text);
                    painter.text(rect.left_top() + Vec2::new(14.0, 66.0), egui::Align2::LEFT_CENTER, "Edited today", theme::font(theme::T_SM), p.text_2);
                }
            });
        });
        block(ui, "Section and separator", "ui::section(ui, label) · ui::separator(ui)", |ui| {
            ui::section(ui, "Motion");
            ui::caption(ui, "A section label groups the rows under it.");
            ui::separator(ui);
        });
        block(ui, "Focus ring", "theme::focus_ring(painter, rect)", |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(160.0, 34.0), Sense::hover());
            ui.painter().rect_filled(rect, egui::Rounding::same(theme::RADIUS), p.panel_2);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "Keyboard focus", theme::font_semi(theme::T_SM), p.text);
            theme::focus_ring(ui.painter(), rect);
        });
        block(ui, "Empty state", "ui::giant + ui::body + ui::button", |ui| {
            theme::panel_box().show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui::giant(ui, "NOTHING YET", 26.0);
                ui::body(ui, "Make your first item and it shows up here.");
                ui.add_space(8.0);
                ui::button(ui, "New item", Some(Icon::Plus), true);
            });
        });
    }

    fn colour_page(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        block(ui, "Tokens", "pal().void, pal().signal, …", |ui| {
            let named = p.colors.named();
            let cols = ((ui.available_width() + 10.0) / 190.0).floor().max(1.0) as usize;
            egui::Grid::new("gallery-colours").spacing(Vec2::new(10.0, 10.0)).show(ui, |ui| {
                for (i, (name, col)) in named.iter().enumerate() {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(180.0, 54.0), Sense::hover());
                    let chip = Rect::from_min_size(rect.min, Vec2::new(54.0, 54.0));
                    ui.painter().rect_filled(chip, egui::Rounding::same(theme::RADIUS), *col);
                    ui.painter().rect_stroke(chip.shrink(0.5), egui::Rounding::same(theme::RADIUS), Stroke::new(1.0_f32, p.line_strong));
                    ui.painter().text(Pos2::new(chip.right() + 10.0, rect.top() + 18.0), egui::Align2::LEFT_CENTER, *name, theme::font_semi(theme::T_SM), p.text);
                    let [r, g, b, a] = col.to_srgba_unmultiplied();
                    let hex = if a == 255 { format!("#{r:02x}{g:02x}{b:02x}") } else { format!("#{r:02x}{g:02x}{b:02x}{a:02x}") };
                    theme::tracked_text(ui.painter(), Pos2::new(chip.right() + 10.0, rect.top() + 38.0), &hex.to_uppercase(), theme::font_hud(theme::T_MICRO), p.text_3, theme::HUD_TRACKING);
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
        });
        block(ui, "Groups", "pal().group(i)", |ui| {
            ui.horizontal(|ui| {
                for i in 0..6 {
                    ui::swatch(ui, p.group(i), &format!("group {i}"), i == 0, None);
                }
            });
        });
    }

    fn icons_page(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        block(ui, "Icon set", "icons::show(ui, Icon::Search, 16.0, color)", |ui| {
            let cols = ((ui.available_width()) / 120.0).floor().max(1.0) as usize;
            egui::Grid::new("gallery-icons").spacing(Vec2::new(0.0, 6.0)).show(ui, |ui| {
                for (i, icon) in Icon::ALL.iter().enumerate() {
                    ui.allocate_ui_with_layout(Vec2::new(120.0, 28.0), Layout::left_to_right(Align::Center), |ui| {
                        ui.set_width(120.0);
                        ncf::icons::show(ui, *icon, 16.0, p.text);
                        ui.label(egui::RichText::new(format!("{icon:?}")).font(theme::font(theme::T_CAP)).color(p.text_2));
                    });
                    if (i + 1) % cols == 0 {
                        ui.end_row();
                    }
                }
            });
        });
    }

    fn marks_page(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        block(ui, "NativeCompute", "mark::NATIVECOMPUTE.paint(painter, rect)", |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
            mark::NATIVECOMPUTE.paint(ui.painter(), Rect::from_min_size(rect.min, Vec2::splat(120.0)));
            ui.painter().text(Pos2::new(rect.left() + 150.0, rect.center().y), egui::Align2::LEFT_CENTER, tokens::BRAND, theme::font_bold(30.0), p.text);
        });
        block(ui, "App mark", "AppMark { parts: &[slab(..), line(..)] }", |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 128.0), Sense::hover());
            let mut x = rect.left();
            for size in [128.0, 64.0, 32.0, 16.0] {
                let r = Rect::from_min_size(Pos2::new(x, rect.bottom() - size), Vec2::splat(size));
                ui.painter().rect_stroke(r, egui::Rounding::ZERO, Stroke::new(1.0_f32, p.line));
                MARK.paint(ui.painter(), r);
                x += size + 24.0;
            }
        });
        block(ui, "Spinner", "mark::spinner(painter, centre, radius, anim::clock(ctx), color)", |ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(60.0, 60.0), Sense::hover());
            mark::spinner(ui.painter(), rect.center(), 20.0, anim::clock(ui.ctx()), p.signal_hi);
            anim::keep_going(ui.ctx());
        });
    }

    fn notices_page(&mut self, ui: &mut egui::Ui, shell: &mut Shell) {
        block(ui, "Notices", "shell.say · shell.ok · shell.warn", |ui| {
            ui.horizontal(|ui| {
                if ui::button(ui, "Say", Some(Icon::Info), false) {
                    shell.say("Ledger 0.2.0 is out", "Download it from Settings → About.");
                }
                if ui::button(ui, "Ok", Some(Icon::Check), false) {
                    shell.ok("Exported", "Saved to Documents as Ledger.pdf.");
                }
                if ui::button(ui, "Warn", Some(Icon::Warning), false) {
                    shell.warn("Could not save", "The folder is read-only. Pick another one in Settings.");
                }
            });
        });
        block(ui, "Confirm", "shell.ask(title, body, yes, danger, action)", |ui| {
            if ui::button(ui, "Delete item", Some(Icon::Trash), false) {
                shell.ask("Delete Launch notes?", "It goes to the bin. You can bring it back for 30 days.", "Delete", true, "delete");
            }
        });
        block(ui, "Settings window", "shell.open_settings(tab)", |ui| {
            if ui::button(ui, "Open Settings", Some(Icon::Settings), false) {
                shell.open_settings("Appearance");
            }
        });
    }

    fn motion_page(&mut self, ui: &mut egui::Ui) {
        let p = pal();
        block(ui, "Ease", "anim::ease(ctx, id, on, tokens::MOTION_BASE)", |ui| {
            if ui::button(ui, if self.moved { "Back" } else { "Move" }, Some(Icon::ArrowRight), false) {
                self.moved = !self.moved;
            }
            ui.add_space(8.0);
            let (track, _) = ui.allocate_exact_size(Vec2::new(ui.available_width().min(520.0), 34.0), Sense::hover());
            ui.painter().rect_filled(Rect::from_min_size(Pos2::new(track.left(), track.center().y), Vec2::new(track.width(), 1.0)), egui::Rounding::ZERO, p.line_strong);
            for (row, secs) in [(0.0, tokens::MOTION_FAST), (1.0, tokens::MOTION_BASE), (2.0, tokens::MOTION_SLOW)] {
                let t = anim::ease(ui.ctx(), ("gallery-move", row as i32), self.moved, secs);
                let x = track.left() + (track.width() - 10.0) * t;
                let y = track.top() + 6.0 + row * 11.0;
                ui.painter().rect_filled(Rect::from_min_size(Pos2::new(x, y), Vec2::new(10.0, 6.0)), egui::Rounding::ZERO, p.signal_hi);
            }
        });
        block(ui, "Hover wipe", "ui::wipe(painter, rect, t, color)", |ui| {
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(220.0, 40.0), Sense::hover());
            let t = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
            ui.painter().rect_stroke(rect.shrink(0.5), egui::Rounding::same(theme::RADIUS), Stroke::new(1.0_f32, p.line_strong));
            ui::wipe(ui.painter(), rect, t, p.signal);
            ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "Hover me", theme::font_semi(theme::T_SM), theme::mix(p.text, p.ink, t));
        });
        block(ui, "Durations", "tokens::MOTION_*", |ui| {
            for (name, v) in [("fast", tokens::MOTION_FAST), ("quick", tokens::MOTION_QUICK), ("base", tokens::MOTION_BASE), ("slow", tokens::MOTION_SLOW), ("hover", tokens::MOTION_HOVER)] {
                ui::hud(ui, &format!("{name} / {} ms", (v * 1000.0).round()), None);
            }
        });
    }
}

fn main() -> ncf::eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut shell = Shell::new(INFO);
    if args.iter().any(|a| a == "--light") {
        shell.look.light_mode = true;
    }
    if args.iter().any(|a| a == "--dark") {
        shell.look.light_mode = false;
    }
    if args.iter().any(|a| a == "--no-splash") {
        shell.look.splash = false;
    }
    shell.look.check_updates = false;
    let section = args
        .iter()
        .position(|a| a == "--section")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| SECTIONS.iter().position(|(n, _)| n.eq_ignore_ascii_case(s)))
        .unwrap_or(0);
    app::run_with(shell, move |_ctx, _shell| Gallery::new(section))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ncf::testing::Harness;

    #[test]
    fn every_page_draws_in_both_casts() {
        let dir = std::env::temp_dir().join(format!("ncf-gallery-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("XDG_DATA_HOME", &dir);
        let mut h = Harness::new(1320.0, 900.0);
        let mut shell = Shell::new(INFO);
        shell.look.check_updates = false;
        let mut app = app::start(&h.ctx, &mut shell, |_, _| Gallery::new(0));
        for light in [false, true] {
            shell.set_light(light);
            for section in 0..SECTIONS.len() {
                app.section = section;
                h.frames(3, |ctx| app::frame(ctx, &mut app, &mut shell));
            }
        }
        app.menu_open = true;
        app.section = 1;
        h.frames(3, |ctx| app::frame(ctx, &mut app, &mut shell));
        shell.ask("Delete Launch notes?", "It goes to the bin.", "Delete", true, "delete");
        h.frames(3, |ctx| app::frame(ctx, &mut app, &mut shell));
        app.answer("delete", None, &mut shell);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
