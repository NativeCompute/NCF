use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::{AppInfo, Os};

pub mod window;
pub mod work;
#[cfg(test)]
mod tests;

pub const FLAG: &str = "--ncf-update";

#[derive(Clone, Debug, PartialEq)]
pub enum Install {
    MacBundle { app: PathBuf },
    LinuxLocal { bin: PathBuf },
    WindowsExe { exe: PathBuf },
}

impl Install {
    pub fn detect(exe: &Path, home: &Path, os: Os, slug: &str) -> Option<Install> {
        match os {
            Os::Mac => {
                let macos = exe.parent()?;
                let contents = macos.parent()?;
                let app = contents.parent()?;
                if macos.file_name()? != "MacOS" || contents.file_name()? != "Contents" || app.extension()? != "app" {
                    return None;
                }
                let dir = app.parent()?;
                (dir == Path::new("/Applications") || dir == home.join("Applications")).then(|| Install::MacBundle { app: app.to_path_buf() })
            }
            Os::Linux => (exe == home.join(".local/bin").join(slug)).then(|| Install::LinuxLocal { bin: exe.to_path_buf() }),
            Os::Windows => {
                let named = exe.file_name()?.to_string_lossy().eq_ignore_ascii_case(&format!("{slug}.exe"));
                named.then(|| Install::WindowsExe { exe: exe.to_path_buf() })
            }
            Os::Web => None,
        }
    }

    pub fn from_target(target: PathBuf, os: Os) -> Option<Install> {
        match os {
            Os::Mac => Some(Install::MacBundle { app: target }),
            Os::Linux => Some(Install::LinuxLocal { bin: target }),
            Os::Windows => Some(Install::WindowsExe { exe: target }),
            Os::Web => None,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            Install::MacBundle { app } => app,
            Install::LinuxLocal { bin } => bin,
            Install::WindowsExe { exe } => exe,
        }
    }

    pub fn os(&self) -> Os {
        match self {
            Install::MacBundle { .. } => Os::Mac,
            Install::LinuxLocal { .. } => Os::Linux,
            Install::WindowsExe { .. } => Os::Windows,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    pub name: String,
    pub slug: String,
    pub repo: String,
    pub version: String,
    pub source: String,
    pub install: Install,
    pub data: PathBuf,
    pub parent: u32,
    pub headless: bool,
    pub reopen: bool,
}

pub fn slug(name: &str) -> String {
    let mut s = String::new();
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
        } else if !s.ends_with('-') && !s.is_empty() {
            s.push('-');
        }
    }
    s.trim_end_matches('-').to_string()
}

pub fn source_for(repo: &str, version: &str) -> String {
    match std::env::var("NCF_UPDATE_SOURCE") {
        Ok(s) if !s.trim().is_empty() => s.trim().trim_end_matches('/').to_string(),
        _ => format!("https://github.com/{repo}/releases/download/v{version}"),
    }
}

pub fn assets(slug: &str, version: &str, os: Os) -> Vec<String> {
    match os {
        Os::Mac => vec![format!("{slug}-macos.zip"), format!("{slug}-macos.zip.sha256"), "install-macos.sh".to_string()],
        Os::Linux => {
            let tar = format!("{slug}-v{version}-linux-x86_64.tar.gz");
            vec![tar.clone(), format!("{tar}.sha256")]
        }
        Os::Windows => {
            let zip = format!("{slug}-v{version}-windows-x86_64.zip");
            vec![zip.clone(), format!("{zip}.sha256")]
        }
        Os::Web => Vec::new(),
    }
}

pub fn work_dir(slug: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ncf-update-{slug}"))
}

pub fn updater_name(slug: &str, os: Os) -> String {
    if os == Os::Windows {
        format!("{slug}-updater.exe")
    } else {
        format!("{slug}-updater")
    }
}

pub fn old_exe(exe: &Path, slug: &str) -> PathBuf {
    exe.with_file_name(format!("{slug}.old.exe"))
}

impl Plan {
    pub fn for_release(info: &AppInfo, version: &str, data: PathBuf) -> Option<Plan> {
        let slug = slug(info.name);
        let exe = std::env::current_exe().ok()?;
        let exe = exe.canonicalize().unwrap_or(exe);
        let home = dirs::home_dir()?;
        let home = home.canonicalize().unwrap_or(home);
        let install = Install::detect(&exe, &home, crate::OS, &slug)?;
        if let Install::WindowsExe { exe } = &install {
            if !writable(exe.parent()?) {
                return None;
            }
        }
        Some(Plan {
            name: info.name.to_string(),
            slug: slug.clone(),
            repo: info.repo.to_string(),
            version: version.trim_start_matches(['v', 'V']).to_string(),
            source: source_for(info.repo, version.trim_start_matches(['v', 'V'])),
            install,
            data,
            parent: std::process::id(),
            headless: false,
            reopen: true,
        })
    }

    pub fn args(&self) -> Vec<String> {
        let mut out = vec![
            FLAG.to_string(),
            self.version.clone(),
            "--parent".into(),
            self.parent.to_string(),
            "--target".into(),
            self.install.path().display().to_string(),
            "--data".into(),
            self.data.display().to_string(),
            "--source".into(),
            self.source.clone(),
            "--name".into(),
            self.name.clone(),
            "--slug".into(),
            self.slug.clone(),
            "--repo".into(),
            self.repo.clone(),
        ];
        if self.headless {
            out.push("--headless".into());
        }
        if !self.reopen {
            out.push("--no-reopen".into());
        }
        out
    }

    pub fn page(&self) -> String {
        crate::update::releases_page(&self.repo)
    }
}

pub fn from_args(args: &[String], os: Os) -> Option<Result<Plan, String>> {
    let at = args.iter().position(|a| a == FLAG)?;
    let value = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    let need = |flag: &str| value(flag).ok_or_else(|| format!("{flag} is missing"));
    let parse = || -> Result<Plan, String> {
        let version = args.get(at + 1).filter(|v| !v.starts_with("--")).cloned().ok_or("the version to update to is missing")?;
        let parent = need("--parent")?.parse::<u32>().map_err(|_| "--parent is not a process id".to_string())?;
        let target = PathBuf::from(need("--target")?);
        let install = Install::from_target(target, os).ok_or("this platform can't update itself")?;
        Ok(Plan {
            name: need("--name")?,
            slug: need("--slug")?,
            repo: value("--repo").unwrap_or_default(),
            version: version.trim_start_matches(['v', 'V']).to_string(),
            source: need("--source")?.trim_end_matches('/').to_string(),
            install,
            data: PathBuf::from(need("--data")?),
            parent,
            headless: args.iter().any(|a| a == "--headless"),
            reopen: !args.iter().any(|a| a == "--no-reopen"),
        })
    };
    Some(parse())
}

pub fn writable(dir: &Path) -> bool {
    let probe = dir.join(format!(".ncf-write-test-{}", std::process::id()));
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

pub fn detach(cmd: &mut Command) {
    cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0000_0008 | 0x0000_0200);
    }
}

pub fn start(plan: &Plan) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("could not find this program: {e}"))?;
    let dir = work_dir(&plan.slug);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("could not make {}: {e}", dir.display()))?;
    let copy = dir.join(updater_name(&plan.slug, crate::OS));
    std::fs::copy(&exe, &copy).map_err(|e| format!("could not copy the updater: {e}"))?;
    let mut cmd = Command::new(&copy);
    cmd.args(plan.args()).current_dir(&dir);
    detach(&mut cmd);
    cmd.spawn().map(|_| ()).map_err(|e| format!("could not start the updater: {e}"))
}

pub fn tidy(slug: &str) {
    let dir = work_dir(slug);
    let mine = std::env::current_exe().ok().map(|e| e.starts_with(&dir)).unwrap_or(false);
    if !mine {
        let _ = std::fs::remove_dir_all(&dir);
    }
    if crate::OS == Os::Windows {
        if let Ok(exe) = std::env::current_exe() {
            let _ = std::fs::remove_file(old_exe(&exe, slug));
        }
    }
}

pub fn run(info: AppInfo, plan: Plan) -> i32 {
    if plan.headless {
        return work::headless(&plan);
    }
    window::run(info, plan)
}
