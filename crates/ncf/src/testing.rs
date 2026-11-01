use eframe::egui::{self, pos2, vec2, Event, Key, Modifiers, PointerButton, Pos2, RawInput, Rect, Vec2};

pub struct Harness {
    pub ctx: egui::Context,
    pub size: Vec2,
}

impl Harness {
    pub fn new(w: f32, h: f32) -> Harness {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::set_palette(true);
        crate::theme::apply(&ctx);
        crate::anim::set_enabled(false);
        let warm = RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))), ..Default::default() };
        let _ = ctx.run(warm, |_| {});
        Harness { ctx, size: vec2(w, h) }
    }

    pub fn run_with(&mut self, events: Vec<Event>, mut ui: impl FnMut(&egui::Context)) {
        let input = RawInput { screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), self.size)), events, ..Default::default() };
        let ctx = self.ctx.clone();
        let _ = ctx.run(input, |c| ui(c));
    }

    pub fn run(&mut self, ui: impl FnMut(&egui::Context)) {
        self.run_with(Vec::new(), ui);
    }

    pub fn frames(&mut self, n: usize, mut ui: impl FnMut(&egui::Context)) {
        for _ in 0..n {
            self.run(&mut ui);
        }
    }

    pub fn click(&mut self, at: Pos2, mut ui: impl FnMut(&egui::Context)) {
        self.run_with(vec![Event::PointerMoved(at)], &mut ui);
        self.run_with(
            vec![
                Event::PointerMoved(at),
                Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: true, modifiers: Modifiers::NONE },
            ],
            &mut ui,
        );
        self.run_with(vec![Event::PointerButton { pos: at, button: PointerButton::Primary, pressed: false, modifiers: Modifiers::NONE }], &mut ui);
        self.run(&mut ui);
    }

    pub fn press(&mut self, key: Key, modifiers: Modifiers, mut ui: impl FnMut(&egui::Context)) {
        self.run_with(vec![Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }], &mut ui);
        self.run(&mut ui);
    }

    pub fn type_text(&mut self, text: &str, mut ui: impl FnMut(&egui::Context)) {
        self.run_with(vec![Event::Text(text.to_string())], &mut ui);
        self.run(&mut ui);
    }
}
