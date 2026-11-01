---
name: ncf-app
description: Build or change a NativeCompute desktop app or website with NCF, the NativeCompute Framework, so it matches the NativeCompute look, motion and voice exactly. Use when asked to make a new NativeCompute app, add a screen or feature to one, restyle something to the NativeCompute brand, or build a NativeCompute web page.
---

# Build a NativeCompute app with NCF

Follow these steps in order. Do not skip the checks at the end.

## 1. Load the rules

Read `AGENTS.md` at the root of the NCF repository in full. Then read:

- `docs/design-language.md`
- `docs/components.md`
- `docs/recipes.md`
- `docs/writing.md`

Keep `docs/shots/` open as the visual reference. If you cannot see images, read the gallery source, `examples/gallery/src/main.rs`, instead: it calls every component with sensible values.

## 2. Scaffold

For a new desktop app:

```sh
cargo run -p ncf-cli -- new "<Name>" --dir ../<slug> --tagline "<one short sentence>" --repo NativeCompute/<slug>
```

Run this from the NCF checkout, or use `ncf new` if the CLI is installed. Inside the NCF repository itself, add `--ncf-path .` so the app builds against the local crate.

Then, in the new app:

```sh
cargo test
cargo clippy --all-targets -- -D warnings
```

Both must pass before you change anything. If they do not, the NCF checkout is broken: stop and report it.

For a web page, start from `web/gallery.html`: keep the header, `main.layer-top`, the section pattern and the footer, and delete the sections you do not need.

## 3. Give the app its mark

Edit `MARK` in `src/main.rs`: one `slab` and one or two `line`s, leaning right, feet at y 0.83 and heads at y 0.17 (see `docs/brand.md#app-marks`). Then refresh the packaged icons:

```sh
cargo run -- --emit-icon packaging/<slug>.svg
cargo run -- --emit-ico packaging/<slug>.ico
```

## 4. Plan the screens

Before writing code, write down each screen and map every part of it to a component or recipe:

| Screen part | Use |
|---|---|
| Title area | Page header recipe: `ui::eyebrow` + `ui::giant_fit` + one primary `ui::button` |
| Collection of items | Card grid recipe |
| Nothing there yet | Empty state recipe |
| Add one thing | Inline add panel recipe |
| Item list beside its detail | List and detail recipe |
| Several kinds of work | Home picker recipe |
| Preferences | `settings_tabs` + `settings_ui` with `ui::section`, `ui::toggle_row`, `ui::separator` |
| Shortcuts | A `Cmd` enum implementing `keys::Command`, and the Keyboard tab |
| Destructive action | `shell.ask(.., danger: true, ..)` and `answer` |
| Result of an action | `shell.ok` / `shell.say` / `shell.warn` |
| Loading | `mark::spinner` |

If a part has no match, compose it from `theme::panel`, `theme::hover_surface`, `theme::tracked_text`, `ui::hud` and the tokens. Do not invent a new visual idea: no new colours, radii, shadows, gradients or motion curves.

## 5. Build it

- Put app code in `src/app.rs`, or in modules under `src/` once it grows. Keep `main.rs` as `MARK`, `INFO` and `run`.
- Colours from `pal()`, sizes from `theme::T_*` and `tokens::SPACE_*`, durations from `tokens::MOTION_*` and `anim::HOVER`.
- Every string follows `docs/writing.md`: sentence case, verbs first on buttons, notices as "what happened" plus "what next".
- Save with `platform::write_atomic` into `shell.folder()`. Never touch files the app did not create.
- No comments in source. If you need a marker, write a lazy one: `// thing part here`.

## 6. Test it

Extend `src/tests.rs`:

- Drive every screen with `ncf::testing::Harness` in both casts (`shell.set_light(true)` and `false`).
- Click the primary action and assert the state changed.
- Answer each confirmation through `app.answer(...)` and assert the result.
- Round-trip settings: `write_settings`, then `read_setting` on a fresh app, and compare.
- Keep `XDG_DATA_HOME` pointed at the sandbox folder.

## 7. Check it against the gallery

```sh
cargo test
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --target x86_64-pc-windows-gnu -- -D warnings
```

Then run the app and look at each screen in dark and in light, at the default size and at the minimum window size (880 × 580), next to the matching screen in `docs/shots/`. Look for:

- [ ] Text that is cut, overlapping or running outside its box.
- [ ] More than one signal-blue element competing on a screen.
- [ ] Any colour, radius, shadow or gradient that is not in the gallery.
- [ ] Display type below 26 px, or display type on anything but a title.
- [ ] A button label that is not sentence case or does not start with a verb.
- [ ] Hover states missing, or different from the gallery's.
- [ ] The light cast looking wrong anywhere. Fix it with tokens, never with `if p.dark`.

If you can take screenshots (for example under Xvfb with `import -window root`), do, and compare them side by side with `docs/shots/`.

## 8. Ship it

Follow `docs/releasing.md`: bump the version, push to `main`, wait for CI, publish a `vX.Y.Z` release. The release workflow builds the Linux and Windows downloads.

## When NCF itself is missing something

If the app needs a component, icon or token NCF does not have, add it to NCF rather than the app:

1. Add it under `crates/ncf` (and `web/` if it applies), using existing tokens.
2. Show it in `examples/gallery` and `web/gallery.html`.
3. Document it in `docs/components.md`.
4. Run every check in `AGENTS.md`, then `tools/shots.sh`.
5. Note it in `CHANGELOG.md` under "Unreleased".
