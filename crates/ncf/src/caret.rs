use eframe::egui::{
    self,
    text::{CCursor, CCursorRange},
};

pub fn range(ctx: &egui::Context, id: egui::Id) -> Option<(usize, usize)> {
    let state = egui::TextEdit::load_state(ctx, id)?;
    let r = state.cursor.char_range()?;
    let a = r.primary.index;
    let b = r.secondary.index;
    Some((a.min(b), a.max(b)))
}

pub fn caret(ctx: &egui::Context, id: egui::Id) -> Option<usize> {
    let (a, b) = range(ctx, id)?;
    if a == b {
        Some(a)
    } else {
        None
    }
}

pub fn set(ctx: &egui::Context, id: egui::Id, index: usize) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
    let c = CCursor::new(index);
    state.cursor.set_char_range(Some(CCursorRange::one(c)));
    state.store(ctx, id);
}

pub fn split_at_char(s: &str, index: usize) -> (String, String) {
    let head: String = s.chars().take(index).collect();
    let tail: String = s.chars().skip(index).collect();
    (head, tail)
}

pub fn char_len(s: &str) -> usize {
    s.chars().count()
}

pub fn select(ctx: &egui::Context, id: egui::Id, from: usize, to: usize) {
    let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
    state.cursor.set_char_range(Some(CCursorRange::two(
        CCursor::new(from),
        CCursor::new(to),
    )));
    state.store(ctx, id);
}
