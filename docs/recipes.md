# Recipes

Screens NativeCompute has already solved. Start from these before designing anything new. The code is from the `ncf new` template (`templates/app/src/app.rs.tmpl`) and Blackline, trimmed to the part that matters.

## Page header: eyebrow, giant title, primary action

Every main screen opens the same way: a HUD breadcrumb, a giant title that fits its space, and the one primary action on the right.

```rust
let w = ui.available_width().min(1100.0);
ui.horizontal(|ui| {
    ui.set_width(w);
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui::eyebrow(ui, &format!("{} / {} items", shell.info.name, self.items.len()));
        ui.add_space(6.0);
        ui::giant_fit(ui, "Your items", 34.0, 22.0, (w - 200.0).max(120.0));
    });
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui::button_response(ui, "New item", Some(Icon::Plus), true, Some(140.0))
            .on_hover_text("New item · Ctrl+N")
            .clicked()
        {
            self.adding = true;
        }
    });
});
```

- Content is capped at 1100 px wide and sits on the left, so the column guides show to the right on wide windows.
- `giant_fit` shrinks a long title down to its minimum, then ellipsises. A title a person typed always goes through it.

## Empty state

```rust
theme::panel_frame().inner_margin(egui::Margin::symmetric(28.0, 26.0)).show(ui, |ui| {
    ui.set_width(w - 58.0);
    ui::giant(ui, "Nothing yet", 26.0);
    ui.add_space(4.0);
    ui::body(ui, "Make your first item and it shows up here.");
    ui.add_space(12.0);
    if ui::button(ui, "New item", Some(Icon::Plus), true) {
        self.adding = true;
    }
});
```

Say the state in the title, the way out in one line, and give the button. A search with no results gets a caption ("Nothing matches."), not a panel.

## Card grid with long titles

```rust
let gap = 14.0;
let cols = (((w + gap) / (CARD_W + gap)).floor() as usize).max(1);
let card_w = (w - gap * (cols - 1) as f32) / cols as f32;
for row in shown.chunks(cols) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = gap;
        for (i, title) in row {
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, 104.0), Sense::click());
            let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
            ui.painter().rect_filled(rect, egui::Rounding::same(theme::RADIUS), theme::mix(p.deep, p.panel, hot));
            ui.painter().rect_stroke(rect.shrink(0.5), egui::Rounding::same(theme::RADIUS), egui::Stroke::new(1.0_f32, theme::mix(p.line, p.text_3, hot)));
            theme::top_edge(ui.painter(), rect);
            let clip = ui.painter().with_clip_rect(rect.shrink(1.0));
            let font = theme::font_bold(theme::T_BODY);
            clip.text(
                Pos2::new(rect.left() + 16.0, rect.top() + 24.0),
                egui::Align2::LEFT_CENTER,
                ui::elide_to_width(ui.ctx(), title, font.clone(), 0.0, card_w - 60.0),
                font,
                p.text,
            );
            theme::tracked_text(&clip, Pos2::new(rect.left() + 16.0, rect.bottom() - 22.0), &format!("ITEM {:02}", i + 1), theme::font_hud(theme::T_MICRO + 0.5), p.text_3, theme::HUD_TRACKING);
        }
    });
    ui.add_space(gap);
}
```

- Columns come from the width, so the grid reflows when the window or the interface size changes.
- The title is measured with `elide_to_width` and the painter is clipped to the card, so nothing ever spills out. Leave room on the right for the hover actions.
- Secondary facts go at the foot in HUD type.
- Actions such as delete appear on hover in the top right, as a 24 px icon button.

## Confirm before delete

```rust
if ui::icon_button(&mut child, Icon::Trash, "Delete", 24.0) {
    shell.ask(&format!("Delete “{}”?", ui::elide(title, 40)), "It can't be brought back.", "Delete", true, &format!("delete:{i}"));
}

fn answer(&mut self, action: &str, _text: Option<String>, shell: &mut Shell) {
    if let Some(i) = action.strip_prefix("delete:").and_then(|n| n.parse::<usize>().ok()) {
        if i < self.items.len() {
            let gone = self.items.remove(i);
            self.save(shell);
            shell.ok("Deleted", &gone);
        }
    }
}
```

- The action is a string that carries what to delete. Check it is still valid when the answer arrives: the list may have changed.
- After the deed, confirm it with a notice that names the thing.
- If the thing can be restored, say how in the body ("It goes to the bin.") and use a ghost button instead of danger.

## Inline add panel

```rust
theme::panel_frame().inner_margin(egui::Margin::same(18.0)).show(ui, |ui| {
    ui.set_width(w - 38.0);
    ui::section(ui, "New item");
    let r = ui::field(ui, &mut self.draft, "What is it called?", w - 38.0);
    r.request_focus();
    let enter = r.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter));
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        if ui::button(ui, "Add", None, true) || enter { self.add(shell); }
        if ui::button(ui, "Cancel", None, false) || ui.input(|i| i.key_pressed(Key::Escape)) {
            self.adding = false;
            self.draft.clear();
        }
    });
});
```

Enter adds, Escape cancels, and focus is already in the field. Prefer this to a modal for one or two fields.

## Settings tabs

```rust
fn settings_tabs(&self) -> Vec<(&'static str, Icon)> {
    vec![("General", Icon::Settings), ("Keyboard", Icon::Keyboard)]
}

fn settings_ui(&mut self, tab: &str, ui: &mut egui::Ui, shell: &mut Shell) {
    match tab {
        "General" => {
            ui::section(ui, "Cards");
            let mut v = self.compact;
            if ui::toggle_row(ui, "Compact cards", "One line per item.", &mut v) {
                self.compact = v;
                shell.save_settings();
            }
            ui::separator(ui);
            ui::section(ui, "Where things live");
            ui::caption(ui, &shell.folder().display().to_string());
            ui.add_space(6.0);
            if ui::button(ui, "Open the folder", Some(Icon::Folder), false) {
                platform::open(shell.folder());
            }
        }
        "Keyboard" => {
            let changed = keys::settings_ui(ui, &mut self.keymap, &mut self.capture, &Cmd::all(), false);
            if changed {
                shell.save_settings();
            }
        }
        _ => {}
    }
}
```

- Appearance and About are NCF's. Add your tabs between them.
- Every tab is a column of `section`, rows, and `separator`. No nested panels inside the pane.
- Save on change. There is no Apply button.
- While Settings is open, feed key presses to `capture.poll` so rebinding works, and skip your own shortcuts.

## Keyboard shortcuts

```rust
if shell.settings_open() {
    if self.capture.poll(ui.ctx(), &mut self.keymap, &Cmd::all(), false) {
        shell.save_settings();
    }
} else if !shell.deck.asking() {
    for cmd in keys::fired(ui.ctx(), &self.keymap, &Cmd::all(), false) {
        match cmd {
            Cmd::NewItem => self.adding = true,
            Cmd::Search => self.focus_search = true,
        }
    }
}
```

Shortcuts do nothing while a confirmation is waiting. Ctrl+, (Settings) and Escape (close) belong to NCF.

## Search field that takes focus from a shortcut

```rust
let search = ui::field(ui, &mut self.search, "Search  ·  Ctrl+F", w.min(380.0));
if self.focus_search {
    search.request_focus();
    self.focus_search = false;
}
```

## List and detail

Blackline's library uses a rail on the left and the open item on the right. Build it from two child UIs:

```rust
let full = ui.available_rect_before_wrap();
let rail = Rect::from_min_size(full.min, Vec2::new(260.0, full.height()));
let detail = Rect::from_min_max(Pos2::new(rail.right() + 24.0, full.top()), full.max);

let mut r = ui.new_child(egui::UiBuilder::new().max_rect(rail).layout(Layout::top_down(Align::Min)));
egui::ScrollArea::vertical().id_salt("rail").show(&mut r, |ui| {
    for (i, item) in self.items.iter().enumerate() {
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(rail.width(), 34.0), Sense::click());
        let hot = anim::ease(ui.ctx(), resp.id, resp.hovered(), anim::HOVER);
        theme::hover_surface(ui.painter(), rect, theme::RADIUS, p.signal, hot, i == self.open);
        let font = theme::font_semi(theme::T_SM);
        ui.painter().text(rect.left_center() + Vec2::new(12.0, 0.0), egui::Align2::LEFT_CENTER,
            ui::elide_to_width(ui.ctx(), &item.title, font.clone(), 0.0, rect.width() - 24.0), font, p.text);
        if resp.clicked() { self.open = i; }
    }
});

let mut d = ui.new_child(egui::UiBuilder::new().max_rect(detail).layout(Layout::top_down(Align::Min)));
ui::eyebrow(&mut d, "Library / open");
ui::giant_fit(&mut d, &self.items[self.open].title, 40.0, 24.0, detail.width());
```

The chosen row keeps the hover fill with a signal edge, so it reads as selected without a second colour.

## Home picker

When an app has several kinds of work (Blackline: Write, Cards, Read), the home screen greets the person with a giant title and offers each as a large ghost button with an icon, in a row, with the most common one as the primary.

```rust
ui::eyebrow(ui, &format!("{} / V{}", shell.info.name, shell.info.version));
ui::giant_fit(ui, "Pick up where you left off", 54.0, 30.0, w);
ui.add_space(18.0);
ui.horizontal(|ui| {
    if ui::button_sized(ui, "Write", Some(Icon::Pencil), true, Some(180.0)) { … }
    if ui::button_sized(ui, "Cards", Some(Icon::Grid), false, Some(180.0)) { … }
    if ui::button_sized(ui, "Read", Some(Icon::Book), false, Some(180.0)) { … }
});
```

Below it, a short list of recent items in the card grid recipe.

## Maker line at the foot

```rust
ui.add_space(24.0);
let (r, _) = ui.allocate_exact_size(Vec2::new(w, 16.0), Sense::hover());
if ui::maker_line(ui, r, true, shell.info.name) {
    shell.open_url(ncf::NATIVECOMPUTE_URL);
}
```

## Web: page head and cards

```html
<section class="page-hero">
  <p class="hud">Pricing / Fixed quotes</p>
  <h1 class="giant"><span class="fit" data-fit>Pricing</span></h1>
  <p class="lead">One price, agreed before we start.</p>
</section>

<div class="cards">
  <article class="card">
    <div class="card-head"><h3>Simple</h3></div>
    <p class="price"><span class="num">€20</span></p>
    <ul><li>One feature</li><li>Fixed quote</li></ul>
    <a class="btn btn--ghost" href="/contact">Choose</a>
  </article>
  <article class="card card--chosen">
    <div class="card-head"><h3>Advanced</h3><span class="card-flag">Most chosen</span></div>
    …
    <a class="btn btn--primary" href="/contact">Choose</a>
  </article>
</div>
```

Only the chosen card gets the primary button.

## Web: confirm before delete

```js
const yes = await NCF.confirm({ title: "Delete this project?", body: "It can't be brought back.", yes: "Delete", danger: true });
if (yes) { await remove(id); NCF.toast("Deleted."); }
```
