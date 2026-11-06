use std::path::{Path, PathBuf};

pub fn data_home() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        if x.is_absolute() {
            return x;
        }
    }
    dirs::data_dir().unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".local/share"))
}

pub fn folder_name(name: &str, os: crate::Os) -> String {
    match os {
        crate::Os::Windows | crate::Os::Mac => name.to_string(),
        _ => name.to_lowercase(),
    }
}

pub fn app_folder(name: &str) -> PathBuf {
    data_home().join(folder_name(name, crate::OS))
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
    #[cfg(target_os = "macos")]
    {
        if quiet("open").arg("-R").arg(path).spawn().is_err() {
            open(path.parent().unwrap_or(path));
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
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

#[cfg(target_os = "macos")]
pub fn pick_file(filter: &FileFilter) -> Result<Option<PathBuf>, String> {
    let exts: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
    let mut dialog = rfd::FileDialog::new().set_title(filter.title.as_str());
    if !exts.is_empty() {
        dialog = dialog.add_filter(filter.label.as_str(), &exts);
    }
    if let Some(home) = dirs::home_dir() {
        dialog = dialog.set_directory(home);
    }
    Ok(dialog.pick_file())
}

pub fn applescript_text(s: &str) -> String {
    let parts: Vec<String> = s
        .split('\n')
        .map(|line| format!("\"{}\"", line.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect();
    parts.join(" & return & ")
}

pub fn alert_script(title: &str, text: &str) -> String {
    format!(
        "display alert {} message {} as critical buttons {{\"OK\"}} default button \"OK\"",
        applescript_text(title),
        applescript_text(text)
    )
}

pub fn alert(title: &str, text: &str) {
    eprintln!("{title}\n{text}");
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript").arg("-e").arg(alert_script(title, text)).status();
    }
    #[cfg(windows)]
    {
        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxW(window: *mut std::ffi::c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
        }
        let wide = |s: &str| s.encode_utf16().chain(Some(0)).collect::<Vec<u16>>();
        let (body, caption) = (wide(text), wide(title));
        unsafe {
            MessageBoxW(std::ptr::null_mut(), body.as_ptr(), caption.as_ptr(), 0x10);
        }
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let shown = std::process::Command::new("zenity")
            .args(["--error", "--no-wrap", &format!("--title={title}"), &format!("--text={text}")])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !shown {
            let _ = std::process::Command::new("kdialog").args(["--title", title, "--error", text]).status();
        }
    }
}

#[cfg(not(any(windows, target_os = "macos")))]
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
