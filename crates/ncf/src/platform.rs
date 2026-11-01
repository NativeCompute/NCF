use std::path::{Path, PathBuf};

pub fn data_home() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        if x.is_absolute() {
            return x;
        }
    }
    dirs::data_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".local/share"))
}

pub fn app_folder(name: &str) -> PathBuf {
    let folder = if cfg!(windows) { name.to_string() } else { name.to_lowercase() };
    data_home().join(folder)
}

pub fn app_folder_or_legacy(name: &str, legacy: &[&str]) -> PathBuf {
    let fresh = app_folder(name);
    if fresh.exists() {
        return fresh;
    }
    for old in legacy {
        let dir = data_home().join(old);
        if dir.is_dir() {
            return dir;
        }
    }
    fresh
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp-write");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

fn quiet(program: &str) -> std::process::Command {
    #[allow(unused_mut)]
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub fn open(target: impl AsRef<std::ffi::OsStr>) {
    let target = target.as_ref();
    #[cfg(windows)]
    let _ = quiet("explorer").arg(target).spawn();
    #[cfg(target_os = "macos")]
    let _ = quiet("open").arg(target).spawn();
    #[cfg(not(any(windows, target_os = "macos")))]
    let _ = quiet("xdg-open").arg(target).spawn();
}

pub fn reveal(path: &Path) {
    #[cfg(windows)]
    {
        let mut select = std::ffi::OsString::from("/select,");
        select.push(path.as_os_str());
        let _ = quiet("explorer").arg(select).spawn();
    }
    #[cfg(not(windows))]
    {
        let uri = format!("file://{}", path.display());
        let spoke = quiet("dbus-send")
            .args([
                "--session",
                "--print-reply",
                "--dest=org.freedesktop.FileManager1",
                "--type=method_call",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowItems",
            ])
            .arg(format!("array:string:{uri}"))
            .arg("string:")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !spoke {
            open(path.parent().unwrap_or(path));
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileFilter {
    pub title: String,
    pub label: String,
    pub extensions: Vec<String>,
}

impl FileFilter {
    pub fn new(title: &str, label: &str, extensions: &[&str]) -> FileFilter {
        FileFilter { title: title.into(), label: label.into(), extensions: extensions.iter().map(|e| e.to_string()).collect() }
    }
}

#[cfg(windows)]
pub fn pick_file(filter: &FileFilter) -> Result<Option<PathBuf>, String> {
    let pattern = filter.extensions.iter().map(|e| format!("*.{e}")).collect::<Vec<_>>().join(";");
    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; $d = New-Object System.Windows.Forms.OpenFileDialog; \
         $d.Title = '{}'; $d.Filter = '{}|{}|All files|*.*'; \
         if ($d.ShowDialog() -eq 'OK') {{ [Console]::Out.Write($d.FileName) }}",
        filter.title.replace('\'', "''"),
        filter.label.replace('\'', "''"),
        pattern
    );
    let out = quiet("powershell")
        .args(["-NoProfile", "-NonInteractive", "-STA", "-Command", &script])
        .output()
        .map_err(|e| format!("Could not open the file dialog ({e})."))?;
    let picked = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if out.status.success() && !picked.is_empty() { Some(PathBuf::from(picked)) } else { None })
}

#[cfg(not(windows))]
pub fn pick_file(filter: &FileFilter) -> Result<Option<PathBuf>, String> {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let globs = filter.extensions.iter().map(|e| format!("*.{e}")).collect::<Vec<_>>().join(" ");
    let tries: [(&str, Vec<String>); 2] = [
        (
            "kdialog",
            vec![
                "--title".into(),
                filter.title.clone(),
                "--getopenfilename".into(),
                home.display().to_string(),
                format!("{} ({globs})", filter.label),
            ],
        ),
        (
            "zenity",
            vec![
                "--file-selection".into(),
                format!("--title={}", filter.title),
                format!("--file-filter={} | {globs}", filter.label),
            ],
        ),
    ];
    for (tool, args) in tries {
        match std::process::Command::new(tool).args(&args).output() {
            Ok(out) => {
                let picked = String::from_utf8_lossy(&out.stdout).trim().to_string();
                return Ok(if out.status.success() && !picked.is_empty() { Some(PathBuf::from(picked)) } else { None });
            }
            Err(_) => continue,
        }
    }
    Err("Neither kdialog nor zenity is installed — drop the file onto the window instead.".into())
}
