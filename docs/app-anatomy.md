# App anatomy

A NativeCompute desktop app is a Rust binary on eframe/egui with NCF doing the shared work. The app draws its own content. NCF draws everything around it, keeps the settings, and checks for updates.

## Start one

```sh
cargo run -p ncf-cli -- new Ledger --tagline "Money, plainly." --repo NativeCompute/ledger
cd ledger
cargo run
```

Or install the CLI once with `cargo install --git https://github.com/NativeCompute/ncf ncf-cli` and run `ncf new Ledger`.

`ncf new` refuses a folder that already has files in it. `--dir` picks the folder; `--ncf-path` points the new app at a local NCF checkout instead of the `v0.2.0` tag.

## What you get

```
ledger/
  Cargo.toml              ncf pinned to a tag; ncf-build for build.rs
  build.rs                embeds the Windows icon and version info
  src/main.rs             MARK, INFO, --emit-icon/--emit-ico/--version, ncf::app::run
  src/app.rs              the app: commands, state, ui, settings tabs, answers
  src/tests.rs            headless tests through ncf::testing::Harness
  packaging/ledger.svg    the app mark
  packaging/ledger.ico    the Windows icon
  packaging/ledger.desktop
  install.sh              Linux install to ~/.local
  .github/workflows/      ci.yml (Linux + Windows, tests, clippy) and release.yml
  README.md, LICENSE, CONTRIBUTING.md, SECURITY.md
```

It runs, passes its tests and passes `cargo clippy --all-targets -- -D warnings` before you change anything.

## The pieces

### `AppInfo`

```rust
pub const INFO: AppInfo = AppInfo {
    name: "Ledger",
    id: "net.nativecompute.Ledger",
    version: env!("CARGO_PKG_VERSION"),
    tagline: "Money, plainly.",
    repo: "NativeCompute/ledger",
    mark: MARK,
};
```

- `name` is shown in the title bar, splash, settings and maker line, and names the data folder.
- `id` is the Wayland app id and the `.desktop` file name.
- `repo` is where the update check looks for releases. Leave it empty to turn the check off.
- `mark` is the app's icon, as data. See [brand.md](brand.md#app-marks).

### `ncf::app::run`

```rust
fn main() -> ncf::eframe::Result<()> {
    ncf::app::run(INFO, |_ctx, shell| app::App::load(shell))
}
```

In order, it:
1. reads `settings.conf` from the app folder;
2. opens an undecorated window with the app mark as its icon (at 128 px, which X11 needs);
3. installs the fonts and applies the cast;
4. calls your closure to build the app, then hands it the settings keys NCF did not use;
5. starts the update check if it is on;
6. shows the splash, then grows the window to 1320 × 900 (minimum 880 × 580).

Every frame it then:
1. draws the backdrop, the column guides and the resize grips;
2. draws the title bar: app mark, name, a Settings button, your `title_extras`, the window buttons;
3. gives your `ui` the rest of the window;
4. opens Settings on Ctrl+, ;
5. shows notices and confirmations, and sends answers to your `answer`;
6. saves settings when anything asked for it.

Use `ncf::app::run_with(shell, make)` when you need to change the shell before the window opens, for example to force a cast from a command-line flag.

### `NcfApp`

```rust
impl NcfApp for App {
    fn ui(&mut self, ui: &mut egui::Ui, shell: &mut Shell) { … }
    fn title_extras(&mut self, ui: &mut egui::Ui, shell: &mut Shell) { … }
    fn settings_tabs(&self) -> Vec<(&'static str, Icon)> { vec![("General", Icon::Settings), ("Keyboard", Icon::Keyboard)] }
    fn settings_ui(&mut self, tab: &str, ui: &mut egui::Ui, shell: &mut Shell) { … }
    fn read_setting(&mut self, key: &str, value: &str) { … }
    fn write_settings(&self) -> String { … }
    fn answer(&mut self, action: &str, text: Option<String>, shell: &mut Shell) { … }
    fn about(&mut self, ui: &mut egui::Ui, shell: &mut Shell) { … }
}
```

Only `ui` is required.

| Method | Called | Do |
|---|---|---|
| `ui` | Every frame | Draw the app. |
| `title_extras` | Every frame, inside the title bar | Up to three icon buttons for global actions. |
| `settings_tabs` | When Settings is open | Your tabs. They sit between Appearance and About. |
| `settings_ui` | When one of your tabs is showing | Draw it with `ui::section`, `ui::toggle_row`, `ui::separator`. Call `shell.save_settings()` after a change. |
| `read_setting` | Once at start, for each key NCF does not own | Parse your values. Ignore keys you do not know. |
| `write_settings` | When settings are saved | Return `key = value` lines. NCF writes its own lines first. |
| `answer` | When a person answers `shell.ask` | Match on the action string you gave. |
| `about` | In Settings → About | Extra lines about the app. NCF already shows the version, tagline, update status and the NativeCompute link. |

### `Shell`

What the app can ask NCF for:

| Call | Does |
|---|---|
| `shell.folder()` | The app's data folder: `$XDG_DATA_HOME/<name in lower case>` or `~/.local/share/<name in lower case>` on Linux, `%APPDATA%\<Name>` on Windows. |
| `shell.say / ok / warn(title, body)` | A notice. |
| `shell.ask(title, body, yes, danger, action)` | A confirmation. The answer arrives in `NcfApp::answer` with `action`. |
| `shell.deck.prompt(title, body, hint, seed, yes, action)` | A confirmation with a text field. The text arrives in `answer` as `Some(text)`. |
| `shell.open_settings(tab)`, `shell.settings_open()` | Open Settings at a tab, or check whether it is open. |
| `shell.set_light(bool)` | Change cast. |
| `shell.save_settings()` | Write settings at the end of this frame. |
| `shell.open_url(url)` | Open a link in the browser. |
| `shell.look` | The Appearance settings: `light_mode`, `animations`, `font_scale`, `splash`, `check_updates`. |
| `shell.update` | The update check: `Off`, `Checking`, `Latest`, `Newer(release)`, `Failed`. |
| `shell.frame_no` | A frame counter, for `ui::Dismisser`. |

### Settings file

`settings.conf` in the app folder, one `key = value` per line, written atomically. NCF owns `light_mode`, `animations`, `font_scale`, `splash` and `check_updates`. Everything else is the app's. Never rename a key without reading the old name too.

### Platform helpers (`ncf::platform`)

| Call | Does |
|---|---|
| `platform::app_folder(name)` | The data folder for any app name. |
| `platform::app_folder_or_legacy(name, &["OldName"])` | Uses an old folder if it exists and the new one does not, so a renamed app keeps its data. |
| `platform::write_atomic(path, bytes)` | Writes to a temporary file and renames it over the target. Use it for every save. |
| `platform::open(target)` | Opens a file, folder or URL with the system handler. |
| `platform::reveal(path)` | Shows a file in the file manager. |
| `platform::pick_file(&FileFilter::new(title, label, &["md"]))` | A native open dialog: PowerShell on Windows, kdialog or zenity on Linux. |

### Update check (`ncf::update`)

The check fetches `api.github.com/repos/<repo>/releases/latest` once per launch on a background thread, compares versions, and shows a notice when a newer one is out. Settings → About shows the status, a switch to turn it off, and a link to the release. Publish releases with tags like `v1.2.0`.

## Testing

Enable the `testing` feature in dev-dependencies (the template does) and drive the app headless:

```rust
let mut h = Harness::new(1320.0, 900.0);
let mut shell = Shell::new(INFO);
shell.look.check_updates = false;
let mut app = start(&h.ctx, &mut shell, |_, s| App::load(s));
h.frames(3, |ctx| frame(ctx, &mut app, &mut shell));
h.click(pos, |ctx| frame(ctx, &mut app, &mut shell));
h.press(Key::N, Modifiers::COMMAND, |ctx| frame(ctx, &mut app, &mut shell));
h.type_text("Rent", |ctx| frame(ctx, &mut app, &mut shell));
```

Point `XDG_DATA_HOME` at a temporary folder first so tests never touch real data. The template's `sandbox()` does this.

Draw each screen in both casts in at least one test. Most layout bugs show up as a panic or an overflow in one cast only.

## Shipping

See [releasing.md](releasing.md).
