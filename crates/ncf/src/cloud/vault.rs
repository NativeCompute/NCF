use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stored {
    pub refresh_token: String,
    pub user_id: String,
    pub email: String,
}

#[derive(Clone, Debug)]
pub struct Vault {
    file: PathBuf,
    account: String,
    system: bool,
}

impl Vault {
    pub fn system(folder: &Path, app_id: &str) -> Vault {
        Vault { file: folder.join("cloud-session"), account: app_id.to_string(), system: cfg!(any(windows, target_os = "macos")) }
    }

    pub fn file_only(file: PathBuf) -> Vault {
        Vault { file, account: String::new(), system: false }
    }

    pub fn account(&self) -> &str {
        &self.account
    }

    pub fn uses_system_store(&self) -> bool {
        self.system
    }

    pub fn load(&self) -> Option<Stored> {
        let text = self.read_system().or_else(|| std::fs::read_to_string(&self.file).ok())?;
        serde_json::from_str::<Stored>(&text).ok().filter(|s| !s.refresh_token.is_empty())
    }

    pub fn save(&self, s: &Stored) {
        let text = serde_json::to_string(s).unwrap_or_default();
        if self.write_system(&text) {
            let _ = std::fs::remove_file(&self.file);
            return;
        }
        let _ = write_private(&self.file, &text);
    }

    pub fn clear(&self) {
        self.clear_system();
        let _ = std::fs::remove_file(&self.file);
    }

    #[cfg(all(feature = "cloud", any(windows, target_os = "macos")))]
    fn entry(&self) -> Option<keyring::Entry> {
        if !self.system {
            return None;
        }
        keyring::Entry::new("NativeCloud", &self.account).ok()
    }

    #[cfg(all(feature = "cloud", any(windows, target_os = "macos")))]
    fn read_system(&self) -> Option<String> {
        self.entry()?.get_password().ok()
    }

    #[cfg(all(feature = "cloud", any(windows, target_os = "macos")))]
    fn write_system(&self, text: &str) -> bool {
        self.entry().map(|e| e.set_password(text).is_ok()).unwrap_or(false)
    }

    #[cfg(all(feature = "cloud", any(windows, target_os = "macos")))]
    fn clear_system(&self) {
        if let Some(e) = self.entry() {
            let _ = e.delete_credential();
        }
    }

    #[cfg(not(all(feature = "cloud", any(windows, target_os = "macos"))))]
    fn read_system(&self) -> Option<String> {
        None
    }

    #[cfg(not(all(feature = "cloud", any(windows, target_os = "macos"))))]
    fn write_system(&self, _text: &str) -> bool {
        false
    }

    #[cfg(not(all(feature = "cloud", any(windows, target_os = "macos"))))]
    fn clear_system(&self) {}
}

pub fn write_private(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("tmp-write");
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&tmp)?;
        f.write_all(text.as_bytes())?;
    }
    #[cfg(not(unix))]
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}
