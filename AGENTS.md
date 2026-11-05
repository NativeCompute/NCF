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

New app? Follow [`.claude/skills/ncf-app/SKILL.md`](.claude/skills/ncf-app/SKILL.md) from step one.

## The rules

### Tokens only
- Every colour comes from `pal()` in Rust or a `var(--…)` in CSS. Never write a hex value, `Color32::from_rgb` or `rgb()` in app code.
- Every size comes from `theme::T_*`, `tokens::SPACE_*`, `tokens::RADIUS` or the CSS `--s-*`, `--t-*` and `--r-sm` variables.
- Every duration comes from `tokens::MOTION_*` or `anim::HOVER` in Rust, `--m-*` in CSS.
- A value that is missing from the tokens goes into `tokens/ncf.toml`, then `cargo run -p ncf-tokens`, in an NCF change. It never goes into the app.

### Components before custom drawing
- Use `ncf::ui` and `ncf::theme` first. Look in [`docs/components.md`](docs/components.md) and the gallery (`cargo run -p ncf-gallery`) before drawing anything by hand.
- When you have to draw by hand, build from `theme::panel`, `theme::hover_surface`, `theme::tracked_text`, `ui::wipe` and the tokens, so the result reads as ours.
- On the web, use the class names in `web/ncf.css` as they are. Do not restyle `.btn`, `.panel` or `.tabs` per page.

### Do
- One accent: signal blue (`signal`, `signal-hi`). Everything else is neutral.
- Corners at 2 px (`RADIUS`). Hairline borders (`line`, `line-strong`), 1 px. The two built-in exceptions are notices, which have fully rounded ends, and keycaps, at 6 px; both are drawn by NCF, so never copy them onto other things.
- Solid fills: `void` behind everything, then `deep`, `panel`, `panel-2`.
- Display type is Anybody 800 at width 150, upper case, for giant words and page titles only.
- HUD readouts are JetBrains Mono 500, upper case, tracked 0.08em, in `text-2` or `text-3`.
- UI text is Anybody 600 at width 112; body text is Anybody 400.
- Sentence case for buttons, labels and body copy. Verbs first on buttons.
- Show the six column guides behind content (`chrome::column_guides`, or the CSS `body::before`).
- Every interactive element has a focus ring: 2 px `signal-hi`, offset 2 px.
- Hit targets are at least 28 px in the desktop app and 44 px on the web.
- Every text and background pair meets WCAG AA. `cargo test -p ncf-tokens` checks the palette; you check your own pairs.
- Respect the Animations switch (desktop) and `prefers-reduced-motion` (web).
- Long text is measured and shortened with `ui::elide_to_width`, never cut by character count.

### Never
- Glass, blur, glows or drop shadows.
- Multi-hue gradients, new gradients on buttons or surfaces, or more than one accent hue. The single-hue shading on confirmation buttons and the faint accent wash on notices are built in; do not add more.
- Pills or badges on static text. A chip is for a tag a person can act on.
- Rounded corners above 2 px, other than the notices and keycaps NCF already draws.
- Emoji in the interface.
- Motion longer than 300 ms, except what NCF already does: the splash, notices arriving (420 ms) and a web page-load reveal (600 ms).
- Bouncy or elastic motion on anything a person waits on. The overshoot on notices and confirmations is built in; do not add more.
- Custom cursors, fake metrics, fake testimonials, cartoon illustration.
- Text set in the display face below 26 px.

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
