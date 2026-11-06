use std::path::{Path, PathBuf};

use super::work::{self, Event, State, Step};
use super::*;

fn temp(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ncf-updater-test-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn only_a_standard_install_can_update_itself() {
    let home = Path::new("/Users/sam");
    let mac = |p: &str| Install::detect(Path::new(p), home, Os::Mac, "blackline");
    assert_eq!(
        mac("/Applications/Blackline.app/Contents/MacOS/blackline"),
        Some(Install::MacBundle { app: PathBuf::from("/Applications/Blackline.app") })
    );
    assert_eq!(
        mac("/Users/sam/Applications/Blackline.app/Contents/MacOS/blackline"),
        Some(Install::MacBundle { app: PathBuf::from("/Users/sam/Applications/Blackline.app") })
    );
    assert_eq!(mac("/Users/sam/Desktop/Blackline.app/Contents/MacOS/blackline"), None, "an app outside Applications");
    assert_eq!(mac("/Users/sam/blackline/target/release/blackline"), None, "a build from source");
    assert_eq!(mac("/Applications/Blackline/Contents/MacOS/blackline"), None, "not a .app");

    let home = Path::new("/home/sam");
    let linux = |p: &str| Install::detect(Path::new(p), home, Os::Linux, "blackline");
    assert_eq!(linux("/home/sam/.local/bin/blackline"), Some(Install::LinuxLocal { bin: PathBuf::from("/home/sam/.local/bin/blackline") }));
    assert_eq!(linux("/home/sam/code/blackline/target/release/blackline"), None);
    assert_eq!(linux("/usr/bin/blackline"), None);

    let win = |p: &str| Install::detect(Path::new(p), Path::new("C:/Users/sam"), Os::Windows, "blackline");
    assert!(matches!(win("D:/Apps/Blackline/blackline.exe"), Some(Install::WindowsExe { .. })));
    assert!(matches!(win("D:/Apps/Blackline/BLACKLINE.EXE"), Some(Install::WindowsExe { .. })));
    assert_eq!(win("D:/Apps/Blackline/blackline-updater.exe"), None);
    assert_eq!(Install::detect(Path::new("/x"), home, Os::Web, "blackline"), None);
}

#[test]
fn each_platform_downloads_its_own_release_files() {
    assert_eq!(assets("blackline", "2.2.1", Os::Mac), ["blackline-macos.zip", "blackline-macos.zip.sha256", "install-macos.sh"]);
    assert_eq!(assets("blackline", "2.2.1", Os::Linux), ["blackline-v2.2.1-linux-x86_64.tar.gz", "blackline-v2.2.1-linux-x86_64.tar.gz.sha256"]);
    assert_eq!(assets("blackline", "2.2.1", Os::Windows), ["blackline-v2.2.1-windows-x86_64.zip", "blackline-v2.2.1-windows-x86_64.zip.sha256"]);
    assert!(assets("blackline", "2.2.1", Os::Web).is_empty());
    assert_eq!(slug("Field Notes"), "field-notes");
    assert_eq!(slug("Blackline"), "blackline");
    assert_eq!(updater_name("blackline", Os::Windows), "blackline-updater.exe");
    assert_eq!(updater_name("blackline", Os::Mac), "blackline-updater", "a different process name from the app's");
}

#[test]
fn the_updater_gets_the_whole_plan_through_its_arguments() {
    for (os, target) in [
        (Os::Mac, "/Applications/Field Notes.app"),
        (Os::Linux, "/home/sam/.local/bin/field-notes"),
        (Os::Windows, "D:\\Apps\\field-notes.exe"),
    ] {
        let plan = Plan {
            name: "Field Notes".into(),
            slug: "field-notes".into(),
            repo: "NativeCompute/field-notes".into(),
            version: "1.4.0".into(),
            source: "https://github.com/NativeCompute/field-notes/releases/download/v1.4.0".into(),
            install: Install::from_target(PathBuf::from(target), os).unwrap(),
            data: PathBuf::from("/tmp/My Data"),
            parent: 4242,
            headless: os == Os::Windows,
            reopen: os != Os::Linux,
        };
        let mut args = vec!["field-notes-updater".to_string()];
        args.extend(plan.args());
        assert_eq!(from_args(&args, os), Some(Ok(plan.clone())), "{os:?}");
    }
    assert_eq!(from_args(&["app".into(), "--version".into()], Os::Linux), None, "a normal start is left alone");
    assert!(matches!(from_args(&["app".into(), FLAG.into(), "1.0.0".into()], Os::Linux), Some(Err(_))), "a half plan is refused");
}

#[test]
fn a_release_names_the_exact_version_it_installs() {
    std::env::remove_var("NCF_UPDATE_SOURCE");
    assert_eq!(source_for("NativeCompute/blackline", "2.2.1"), "https://github.com/NativeCompute/blackline/releases/download/v2.2.1");
}

#[test]
fn checksums_and_installer_lines_read_the_way_the_tools_write_them() {
    let dir = temp("sum");
    std::fs::write(dir.join("abc"), b"abc").unwrap();
    assert_eq!(work::sha256_hex(&dir.join("abc")).unwrap(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert_eq!(
        work::expected_sum("BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD  blackline-macos.zip\n").as_deref(),
        Some("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
    assert_eq!(work::expected_sum("not a checksum"), None);
    assert_eq!(work::clean_line("\u{1b}[1;34m::\u{1b}[0m Checksum matches"), "Checksum matches");
    assert_eq!(work::clean_line("  plain line "), "plain line");
    assert_eq!(work::stamp(0), "19700101-000000");
    assert_eq!(work::stamp(1_730_851_200 + 3_723), "20241106-010203");
    assert_eq!(work::local_path("file:///tmp/x"), Some(PathBuf::from("/tmp/x")));
    assert_eq!(work::local_path("file:///C:/Users/x"), Some(PathBuf::from("C:/Users/x")));
    assert_eq!(work::local_path("https://example.com"), None);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn the_window_follows_the_work_step_by_step() {
    let mut s = State::default();
    assert!(!s.finished() && !s.installing());
    s.apply(Event::Step(Step::Download));
    s.apply(Event::Progress(0.4));
    assert_eq!((s.step, s.progress), (Some(Step::Download), 0.4));
    s.apply(Event::Progress(7.0));
    assert_eq!(s.progress, 1.0);
    s.apply(Event::Step(Step::Install));
    assert_eq!(s.progress, 0.0, "a new step starts its own progress");
    assert!(s.installing(), "closing the window is refused while installing");
    s.apply(Event::Line("Installed to /Applications/Blackline.app".into()));
    assert_eq!(s.detail, "Installed to /Applications/Blackline.app");
    s.apply(Event::Done);
    assert!(s.finished() && !s.installing());
    let mut f = State::default();
    f.apply(Event::Step(Step::Check));
    f.apply(Event::Failed("damaged".into()));
    assert_eq!(f.outcome, Some(Err("damaged".to_string())));
    assert_eq!(Step::Reopen.index(), work::STEPS.len() - 1);
}

#[test]
fn swapping_the_program_keeps_the_old_one_until_the_new_one_is_in() {
    let dir = temp("swap");
    let exe = dir.join("blackline.exe");
    let new = dir.join("new.exe");
    std::fs::write(&exe, b"old").unwrap();
    std::fs::write(&new, b"new").unwrap();
    work::swap(&exe, &new, "blackline", std::time::Duration::from_secs(1)).unwrap();
    assert_eq!(std::fs::read(&exe).unwrap(), b"new");
    assert_eq!(std::fs::read(old_exe(&exe, "blackline")).unwrap(), b"old", "kept until the next start tidies it");

    std::fs::write(&exe, b"current").unwrap();
    let missing = dir.join("missing.exe");
    let err = work::swap(&exe, &missing, "blackline", std::time::Duration::from_secs(1)).unwrap_err();
    assert!(err.contains("old one is back"), "{err}");
    assert_eq!(std::fs::read(&exe).unwrap(), b"current", "a failed swap puts the old program back");

    let data = dir.join("Blackline");
    std::fs::create_dir_all(data.join("scripts")).unwrap();
    std::fs::write(data.join("scripts/a.md"), b"INT. HOUSE - DAY").unwrap();
    let backup = work::back_up(&data).unwrap().unwrap();
    assert_eq!(std::fs::read(backup.join("scripts/a.md")).unwrap(), b"INT. HOUSE - DAY");
    assert!(backup.file_name().unwrap().to_string_lossy().starts_with("Blackline-backup-"));
    assert_eq!(work::back_up(&dir.join("none")).unwrap(), None);
    let _ = std::fs::remove_dir_all(dir);
}

#[cfg(unix)]
mod linux_release {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::AtomicBool;

    fn release(dir: &Path, version: &str, prints: &str) -> PathBuf {
        let name = format!("tinyapp-v{version}-linux-x86_64");
        let root = dir.join("build").join(&name);
        std::fs::create_dir_all(&root).unwrap();
        let bin = root.join("tinyapp");
        std::fs::write(&bin, format!("#!/bin/sh\n[ \"$1\" = \"--version\" ] && echo \"{prints}\"\n")).unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(root.join("install.sh"), "#!/usr/bin/env bash\nset -e\ncd \"$(dirname \"$0\")\"\necho \":: Installing\" >&2\ncp tinyapp \"$TINY_TARGET\"\necho \"Installed\"\n").unwrap();
        let out = dir.join("release");
        std::fs::create_dir_all(&out).unwrap();
        let tar = out.join(format!("{name}.tar.gz"));
        assert!(std::process::Command::new("tar").arg("-czf").arg(&tar).arg("-C").arg(dir.join("build")).arg(&name).status().unwrap().success());
        let sum = work::sha256_hex(&tar).unwrap();
        std::fs::write(out.join(format!("{name}.tar.gz.sha256")), format!("{sum}  {name}.tar.gz\n")).unwrap();
        out
    }

    fn plan(dir: &Path, source: &Path) -> Plan {
        Plan {
            name: "Tiny App".into(),
            slug: "tinyapp".into(),
            repo: String::new(),
            version: "9.9.9".into(),
            source: format!("file://{}", source.display()),
            install: Install::LinuxLocal { bin: dir.join("installed") },
            data: dir.join("data"),
            parent: 0,
            headless: true,
            reopen: false,
        }
    }

    fn go(plan: &Plan) -> (Result<(), String>, Vec<Event>) {
        let (tx, rx) = std::sync::mpsc::channel();
        let result = work::update(plan, &tx, &AtomicBool::new(false));
        drop(tx);
        (result, rx.into_iter().collect())
    }

    #[test]
    fn a_release_is_downloaded_checked_and_installed() {
        let _g = crate::tests::lock();
        let dir = temp("linux-ok");
        let source = release(&dir, "9.9.9", "tinyapp 9.9.9");
        std::env::set_var("TINY_TARGET", dir.join("installed"));
        let plan = plan(&dir, &source);
        let (result, events) = go(&plan);
        assert_eq!(result, Ok(()));
        assert!(std::fs::read_to_string(dir.join("installed")).unwrap().contains("9.9.9"));
        let steps: Vec<Step> = events.iter().filter_map(|e| if let Event::Step(s) = e { Some(*s) } else { None }).collect();
        assert_eq!(steps, [Step::Wait, Step::Download, Step::Check, Step::Install]);
        assert!(events.contains(&Event::Line("Installing".into())), "the installer's own lines reach the window");
        let log = std::fs::read_to_string(dir.join("data/update.log")).unwrap();
        assert!(log.contains("Updated to 9.9.9"), "{log}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_damaged_missing_or_broken_release_changes_nothing() {
        let _g = crate::tests::lock();
        let dir = temp("linux-bad");
        std::env::set_var("TINY_TARGET", dir.join("installed"));
        std::fs::write(dir.join("installed"), "old").unwrap();

        let source = release(&dir, "9.9.9", "tinyapp 9.9.9");
        let sum = source.join("tinyapp-v9.9.9-linux-x86_64.tar.gz.sha256");
        std::fs::write(&sum, format!("{}  x\n", "0".repeat(64))).unwrap();
        let err = go(&plan(&dir, &source)).0.unwrap_err();
        assert!(err.contains("damaged"), "{err}");
        assert_eq!(std::fs::read_to_string(dir.join("installed")).unwrap(), "old");

        let err = go(&plan(&dir, &dir.join("nowhere"))).0.unwrap_err();
        assert!(err.contains("could not get"), "{err}");
        assert_eq!(std::fs::read_to_string(dir.join("installed")).unwrap(), "old");

        let _ = std::fs::remove_dir_all(dir.join("build"));
        let _ = std::fs::remove_dir_all(dir.join("release"));
        let source = release(&dir, "9.9.9", "tinyapp 1.0.0");
        let err = go(&plan(&dir, &source)).0.unwrap_err();
        assert!(err.contains("does not run"), "{err}");
        assert_eq!(std::fs::read_to_string(dir.join("installed")).unwrap(), "old", "a new version that won't start is never installed");
        let _ = std::fs::remove_dir_all(dir);
    }
}

#[test]
fn the_updater_demo_walks_the_same_steps_and_installs_nothing() {
    use crate::debug::Demo;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    assert_eq!(demo_from_args(&["app".into(), DEMO_FLAG.into(), "fail".into()]), Some(Demo::Fails));
    assert_eq!(demo_from_args(&["app".into(), DEMO_FLAG.into(), "nonsense".into()]), None);
    assert_eq!(demo_from_args(&["app".into()]), None);
    let info = crate::AppInfo { name: "Field Notes", id: "x", version: "1.3.0", tagline: "", repo: "NativeCompute/field-notes", mark: crate::mark::AppMark { name: "Field Notes", parts: &[] } };
    let plan = demo_plan(&info);
    assert_eq!((plan.version.as_str(), plan.reopen, plan.slug.as_str()), ("1.3.1", false, "field-notes-demo"));

    let steps = |events: &[Event]| events.iter().filter_map(|e| if let Event::Step(s) = e { Some(*s) } else { None }).collect::<Vec<_>>();
    let (rx, worker) = work::simulate(plan.clone(), Demo::Works, Arc::new(AtomicBool::new(false)));
    let events: Vec<Event> = rx.into_iter().collect();
    assert_eq!(worker.join().unwrap(), Ok(()));
    assert_eq!(steps(&events), [Step::Wait, Step::Download, Step::Check, Step::Install]);
    assert_eq!(events.last(), Some(&Event::Done));

    let (rx, worker) = work::simulate(plan.clone(), Demo::Fails, Arc::new(AtomicBool::new(false)));
    let events: Vec<Event> = rx.into_iter().collect();
    assert!(worker.join().unwrap().unwrap_err().contains("damaged"));
    assert_eq!(steps(&events), [Step::Wait, Step::Download, Step::Check]);

    let cancel = Arc::new(AtomicBool::new(false));
    let (rx, worker) = work::simulate(plan, Demo::Slow, cancel.clone());
    for e in rx.iter() {
        if e == Event::Step(Step::Download) {
            cancel.store(true, Ordering::SeqCst);
        }
    }
    assert!(worker.join().unwrap().unwrap_err().contains("Cancelled"), "Cancel stops the slow demo");
}

