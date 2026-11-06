# Changelog

NCF follows [semantic versioning](https://semver.org). Apps pin a tag, so nothing here reaches an app until it moves to that tag.

## Unreleased

## 0.4.0

Launch polish: apps ask about a new version before they open, the splash and updater are shorter, and notices are square readouts.

### Desktop (`ncf`)
- **Update prompt at launch.** The splash waits for the update check, and when a newer version is out, it turns into a prompt with *Update now*, *Not now* and *Skip this version*. The app opens only after a choice. With the splash off, the same card covers the app until it is answered. `splash::Offer`, `splash::Choice`, `Splash::set_offer`, `Splash::take_choice` and `splash::prompt`. A skipped version is stored as `skipped_update`; the settings file is unchanged until one is.
- **Compact splash and updater.** The splash card is 540×200 instead of 540×296, and the updater window 540×308 instead of 540×364; the updater grows only when an error needs the room.
- **Notices in the NCF look.** Notices and confirmations are square panels with a hairline and a tone bar, a HUD label (DONE, NOTE, WARNING, ERROR) and the message. Confirmations use the standard primary and ghost buttons. No more pills or gradient washes.

## 0.3.0

macOS. Every check runs on GitHub's Macs, Apple Silicon and Intel, as well as on Linux and Windows.

### Desktop (`ncf`)
- `ncf::Os` names the platform, so code that differs by platform can be tested for every one of them on any machine.
- The window keeps the native macOS frame: traffic lights, edge resizing, full screen and tiling. The title bar leaves them room and draws no window buttons of its own there, and the custom resize grips are off. `chrome::viewport`, `chrome::fit` (caps the window to the screen, leaving the menu bar and Dock clear) and `chrome::window_radius` hold the rules.
- On a Mac the splash card shows inside the main window, and the window opens centred.
- `keys::label` shows shortcuts the way each platform names its keys: "Cmd+Shift+F" and "Option" on a Mac. `keys::text`, which writes settings files, is unchanged, so a settings file reads the same everywhere.
- `keys::refused` keeps the shortcuts macOS holds for itself, and Option with a letter or number, out of the keymap. A saved shortcut that's refused falls back to its default.
- `NcfApp::on_exit` runs on quit, including Cmd+Q, and the settings are written then.
- `platform`: the data folder is `~/Library/Application Support/<Name>` on a Mac; `reveal` selects the file in Finder; `pick_file` uses the system open panel; `alert` shows a real dialog on every platform.
- `AppMark::png` and `AppMark::icns` (the Mac icon file, 16 to 1024 px), and `bundle::info_plist` for the app bundle.
- On a Mac, apps draw with Metal, and fall back to OpenGL if Metal can't start (`app::renderers`).
- The NativeCompute logo replaces the name in plain type: the maker line, splash, Settings → About and the updater. Clicking it opens nativecompute.net. `ui::logo`, `ui::logo_link`, `ui::menu_logo_item` and `mark::logo`.
- The splash's and updater's progress bars sit directly under their title.
- `ncf::updater`: one-click updates on macOS, Linux and Windows. `Shell::update_now` (or `updater::start`) hands the update to a copy of the app's own program and quits. That copy opens a small window in the NCF look, waits for the app to close, downloads the new release and checks its checksum, installs it the platform's way, copies the library aside first, and reopens the app. If anything fails, nothing is changed and the old version is still installed. Settings → About offers *Update now* when the copy was installed the standard way.
- The mark draws much faster, with the same pixels.

### Tools
- `ncf mark --icns`, and `--emit-icns` / `--emit-info-plist` in new apps.
- New apps call `ncf::updater::from_args` first in `main`, and their release workflow writes a `.sha256` next to every download, so the updater can check what it fetched.
- New apps ship on the Mac: `packaging/macos/bundle.sh` builds one signed, universal app (Apple Silicon and Intel, macOS 11 and newer) and zips it; `install-macos.sh` is the one-line Terminal install, with updates, `--uninstall` and `--from-source`; `install.sh` hands over to it on a Mac; the release workflow attaches the Mac zip, its checksum and the installer.

## 0.2.0

### Desktop (`ncf`)
- The splash card shows how the update check is going, right-aligned above the progress bar: "Checking for updates…", "Up to date", "V2.2.0 is out" or "Could not check for updates". Nothing shows when checks are off. It never holds the splash open.
- `Splash::set_status` and `splash::status_line` for apps that run their own splash. `ncf::app` passes the shell's update status in by itself.
- `update::releases_latest(repo)` gives the link to the newest release page, for a "Download" button.

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
