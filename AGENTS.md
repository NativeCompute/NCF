# Working with NCF

NCF is the NativeCompute Framework. It holds the NativeCompute look and the plumbing every NativeCompute app shares. If you are an agent building or changing a NativeCompute app or site, this file is the contract. Read it before you write code, and read it again before you hand work back.

## What to read, in order

1. This file.
2. [`docs/design-language.md`](docs/design-language.md): what the brand looks like and why.
3. [`docs/components.md`](docs/components.md): every component, its Rust call, its HTML class, and when to use it.
4. [`docs/app-anatomy.md`](docs/app-anatomy.md): how a desktop app is put together on `ncf::app`.
5. [`docs/recipes.md`](docs/recipes.md): screens we have already solved. Copy these before inventing anything.
6. [`docs/writing.md`](docs/writing.md): how NativeCompute talks.
7. [`docs/motion.md`](docs/motion.md) and [`docs/brand.md`](docs/brand.md) when your work touches either.
8. [`docs/cloud.md`](docs/cloud.md) when the app signs in to NativeCloud.

New app? Follow [`.claude/skills/ncf-app/SKILL.md`](.claude/skills/ncf-app/SKILL.md) from step one.

## The rules

### Tokens only
- Every colour comes from `pal()` in Rust or a `var(--…)` in CSS. Never write a hex value, `Color32::from_rgb` or `rgb()` in app code.
- Every size comes from `theme::T_*`, `tokens::SPACE_*`, the radius scale (`theme::R_SM`, `R_MD`, `R_LG`, `R_XL`, `R_PILL`) or the CSS `--s-*`, `--t-*` and `--r-*` variables.
- Every shadow comes from `theme::Lift` (`Rest`, `Float`, `Lift`) or the CSS `--shadow*` variables.
- Every duration comes from `tokens::MOTION_*` or `anim::HOVER` in Rust, `--m-*` in CSS.
- A value that is missing from the tokens goes into `tokens/ncf.toml`, then `cargo run -p ncf-tokens`, in an NCF change. It never goes into the app.

### Components before custom drawing
- Use `ncf::ui` and `ncf::theme` first. Look in [`docs/components.md`](docs/components.md) and the gallery (`cargo run -p ncf-gallery`) before drawing anything by hand.
- When you have to draw by hand, build from `theme::surface`, `theme::tile`, `theme::glass`, `theme::hover_surface`, `theme::tracked_text` and the tokens, so the result reads as ours.
- On the web, use the class names in `web/ncf.css` as they are. Do not restyle `.btn`, `.panel` or `.tabs` per page.

### Do
- Bento: content sits on rounded tiles over a soft wallpaper. White surfaces (dark grey in the dark cast) for content, colour tiles (`pal().tile(i)`: coral, lilac, sun, sky) for the few things that should stand out, always with `on_tile` ink text.
- One accent: coral (`signal`). It fills primary buttons, toggles, progress and the caret. Text in the accent colour uses `signal-hi`, never `signal`: coral on light paper is not readable.
- Text on a coral fill is `ink` (near-black), never white.
- Selection inverts: the chosen tab, segment or nav item becomes a `text`-coloured pill with `deep` text (`pal().selected()`).
- Corners from the scale: 12 for controls and small cards, 20 for panels and menus, 28 for large cards and windows, 32 for hero tiles, pills (`R_PILL`) for buttons, chips, toggles and floating bars.
- Soft shadows from `theme::Lift` only. Floating chrome (`ui::pill_bar`, popovers, the header on the web) is drawn frost: the `glass` fill, a `glass_line` hairline and a float shadow. The desktop cannot blur; the web may use `backdrop-filter`.
- The window background is the wallpaper (`chrome::backdrop`). The dark cast adds a coral glow in the top-left corner, the same glow as the app icon.
- Display type is Bricolage Grotesque 800, sentence case, tracked -0.045em, for page titles, big numbers and giant words, 26 px and up. Headings below that use the heading cut of the same face.
- UI and body text are Plus Jakarta Sans (600 for controls, 400 for running text).
- Labels and readouts (`ui::hud`, `theme::font_hud`) are the Plus Jakarta Sans label cut: weight 560, tabular figures, sentence case, no tracking, in `text-2` or `text-3`. Eyebrows (`ui::eyebrow`, and `ui::section` for group headings) are Jakarta 700 in upper case at 0.06em, and they are the only capitals in the interface. JetBrains Mono is for keycaps and code only.
- Glass is for chrome that floats (sidebars, toolbar and status pills, the search trigger, segmented tabs). Anything a person reads or works on is a solid tile.
- Sentence case for buttons, labels and body copy. Verbs first on buttons.
- Every interactive element has a focus ring: 2 px `focus` (ink in light, white in dark), offset 2 px, following the element's corners.
- Hit targets are at least 28 px in the desktop app and 44 px on the web.
- Every text and background pair meets WCAG AA. `cargo test -p ncf-tokens` checks the palette, including text on tiles, glass and the wallpaper; you check your own pairs.
- Respect the Animations switch (desktop) and `prefers-reduced-motion` (web).
- The appearance follows the computer by default (`Appearance::System`), with Light and Dark as choices. Never special-case one cast in an app.
- Long text is measured and shortened with `ui::elide_to_width`, never cut by character count.

### Never
- Real blur on the desktop, or shadows, glows and gradients other than the token ones. The wallpaper and the dark-cast corner glow are the only multi-hue gradients, and NCF draws both.
- Coral as text on a light surface, or white text on coral or on a tile.
- Corners off the radius scale.
- More than one accent. Tile colours are fills for whole tiles, avatars and tags, not a second accent for controls.
- Pills or badges on static text. A chip is for a tag a person can act on.
- Emoji in the interface.
- Motion longer than 300 ms, except what NCF already does: the splash, notices arriving (420 ms) and a web page-load reveal (600 ms).
- Bouncy or elastic motion on anything a person waits on. The overshoot on notices and confirmations is built in; do not add more.
- Custom cursors, fake metrics, fake testimonials, cartoon illustration.
- Text set in the display face below 26 px (use the heading cut instead).

### Code
- No comments in source code. If a marker helps you find your place later, use a lazy one: `// thing part here` or `// function_name here---`. Explanations go in `docs/`.
- Keep `cargo test` and `cargo clippy --all-targets -- -D warnings` clean on Linux and on `x86_64-pc-windows-gnu`. CI runs both on GitHub's Macs too (Apple Silicon and Intel), and nothing merges until they pass there.
- Anything that differs by platform takes an `Os` (`ncf::Os`) instead of reading `cfg!` in place, so tests on Linux can run the macOS and Windows branches. On a Mac the window keeps the native frame (`chrome::viewport`), shortcuts are shown with `keys::label` ("Cmd", "Option"; never `keys::text`, which is for files), and Option with a letter or number is never a default shortcut because it types a character.
- The template under `templates/app` must build, test and pass clippy straight from `ncf new`. CI checks it.
- Generated files (`crates/ncf/src/tokens.rs`, `web/ncf-tokens.css`) are never edited by hand.

### People's data
- Never delete, rewrite or move a person's files as a side effect. Write with `platform::write_atomic`. Ask with `shell.ask` before anything destructive.
- Settings live in `platform::app_folder(name)/settings.conf`. Keep old keys readable when you rename one.
- When an app is moved onto a new NCF version, its saved files must open exactly as before. Checksum them before and after.

## Checking your work

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo clippy --workspace --all-targets --all-features --target x86_64-pc-windows-gnu -- -D warnings
cargo build -p ncf --lib --target wasm32-unknown-unknown
cargo run -p ncf-tokens -- --check
tools/shots.sh
```

`tools/shots.sh` writes every gallery screen in both casts to `docs/shots/`. Compare your screens with them side by side. If yours look different, yours are wrong.
