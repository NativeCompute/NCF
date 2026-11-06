# Releasing

## An app

Apps made with `ncf new` ship the same way Blackline does.

1. Bump `version` in `Cargo.toml`. The splash, Settings → About, `--version` and the Windows file properties all read it.
2. Run the checks:
   ```sh
   cargo test
   cargo clippy --all-targets -- -D warnings
   ```
3. Commit and push to `main`. CI runs the same checks on Linux, Windows and GitHub's Macs.
4. Publish a release on GitHub with a tag such as `v1.2.0`, either by pushing the tag or from **Releases → Draft a new release** on GitHub (create the tag there, then **Publish release**).
5. `.github/workflows/release.yml` builds:
   - `<app>-v1.2.0-linux-x86_64.tar.gz`: the binary, `install.sh`, the `.desktop` file, the icon, README and LICENSE;
   - `<app>-v1.2.0-windows-x86_64.zip`: the `.exe` with its icon and version info, README and LICENSE;
   - `<app>-v1.2.0-macos-universal.zip`: the signed app for Apple Silicon and Intel, with its `.sha256`;
   - `<app>-macos.zip`, its `.sha256` and `install-macos.sh` under fixed names, for the Terminal install;

   and attaches them to the release.
6. Running copies see the new version at their next launch through the update check, as long as `INFO.repo` names the repository. Copies installed the standard way can update with one click (*Update now*). The updater downloads that release's files and their `.sha256`, so keep the asset names the template's release workflow writes.

Write the release notes yourself, in the voice of [writing.md](writing.md): what's new, what's fixed, then anything a person has to do. Leave out internal changes.

### Linux install

```sh
tar -xzf ledger-v1.2.0-linux-x86_64.tar.gz
cd ledger-v1.2.0-linux-x86_64
./install.sh
```

`install.sh` puts the binary in `~/.local/bin`, the icon in `~/.local/share/icons/hicolor/scalable/apps`, and the launcher in `~/.local/share/applications`. `./install.sh --uninstall` removes them and leaves the person's data alone.

### macOS install

```sh
curl -fsSL https://github.com/<owner>/<app>/releases/latest/download/install-macos.sh | bash
```

It installs `<Name>.app` into `/Applications` (or `~/Applications` when that isn't writable), with no password. Running it again updates the app, and `bash -s -- --uninstall` removes it and leaves the person's data alone. A copy fetched with `curl` isn't quarantined, so macOS opens it without the *Open Anyway* step a browser download needs: the app is signed ad hoc, not notarized.

`packaging/macos/bundle.sh` builds the zip on a Mac. It checks each chip's build runs on macOS 11, that `Info.plist` and the icon are valid, and that the signature verifies.

### Windows

Unzip and run the `.exe`. The icon and the version on the file's Details tab come from `build.rs`, which calls `ncf_build::windows(name, "packaging/<app>.ico")`.

Building the Windows binary from Linux works with the `x86_64-pc-windows-gnu` target and `mingw-w64`:

```sh
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

## NCF itself

NCF follows semantic versioning. Apps pin a tag, so nothing changes for an app until it moves to a new one.

- **Patch** (0.1.1): fixes that change no look and no API.
- **Minor** (0.2.0): new components, tokens or API. While NCF is below 1.0, a minor version may also change a look or an API, and the CHANGELOG says how to move.
- **Major** (1.0.0 and up): anything that breaks an app.

To release:

1. Update `CHANGELOG.md` and the `version` in the workspace `Cargo.toml`.
2. Update the tag in the README and docs. `ncf new` writes `v` plus the workspace version into new apps by itself.
3. Run every check in [AGENTS.md](../AGENTS.md#checking-your-work), and `tools/shots.sh` to refresh the screenshots.
4. Push to `main`, wait for CI, then publish a release with the tag `vX.Y.Z` on GitHub.

### Moving an app to a new NCF

1. Change the tag in the app's `Cargo.toml`.
2. Read the CHANGELOG entries in between.
3. Build, test, and compare screenshots of every screen before and after.
4. Checksum the app's data folder before and after a launch. The app must open existing files without changing them.
