# Changelog

NCF follows [semantic versioning](https://semver.org). Apps pin a tag, so nothing here reaches an app until it moves to that tag.

## Unreleased

## 0.1.0

The first release: the NativeCompute look and app plumbing, lifted out of Blackline and nativecompute.net into one framework.

### Tokens
- `tokens/ncf.toml` holds every colour (dark and light), type role and size, spacing step, shape value and motion duration.
- `ncf-tokens` generates `crates/ncf/src/tokens.rs` and `web/ncf-tokens.css`, and checks WCAG AA contrast for every text colour on every surface.

### Desktop (`ncf`)
- Theme: palettes, fonts (Anybody and JetBrains Mono), colour helpers, surfaces, focus ring.
- Components: buttons, icon and ribbon buttons, chips, links, fields, toggles, sliders, segmented controls, tabs, vertical tabs, dropdowns, menus, popovers, swatches, sections, separators, panels, cards, giant and HUD type, and measured text fitting.
- Window chrome: title bar, window buttons, resize grips, backdrop and column guides.
- Splash card, notices and confirmations, the icon set, and the spinner.
- App marks as data, with SVG, PNG and Windows ICO export. The NativeCompute mark.
- `ncf::app`: the shell that runs an app, with Settings (Appearance, the app's tabs, About), saved settings, an update check against GitHub releases, and the maker line.
- Keyboard: commands, rebindable keymaps and the Settings → Keyboard page.
- Platform helpers: data folders, atomic writes, open, reveal and a native file picker on Linux and Windows.
- `ncf::testing::Harness` for headless UI tests.
- Builds for WebAssembly (without `app` and `platform`).

### Web
- `ncf.css`: tokens, fonts, column guides, type roles, buttons, icon buttons and tooltips, header and navigation, panels, cards, tabs, forms, FAQ, tray, toast, confirm, footer, view transitions and reduced motion.
- `ncf.js`: header, tabs, fitted giant text, clock, copy buttons, tray, toast, confirm and the theme switch.
- `gallery.html` with every web component.

### Tools
- `ncf new` writes a complete app: splash, Home, Settings, icons, Windows resources, `install.sh`, CI and release workflows, and docs.
- `ncf-build` embeds the Windows icon and version in `build.rs`.
- `ncf-gallery` shows every desktop component in both casts; `tools/shots.sh` captures them.
- Docs, `AGENTS.md` and the `ncf-app` skill.
