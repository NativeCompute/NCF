#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Os {
    Linux,
    Windows,
    Mac,
    Web,
}

impl Os {
    pub const fn current() -> Os {
        if cfg!(target_arch = "wasm32") {
            Os::Web
        } else if cfg!(target_os = "macos") {
            Os::Mac
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        }
    }

    pub const fn is_mac(self) -> bool {
        matches!(self, Os::Mac)
    }

    pub const fn command_key(self) -> &'static str {
        if self.is_mac() {
            "Cmd"
        } else {
            "Ctrl"
        }
    }

    pub const fn alt_key(self) -> &'static str {
        if self.is_mac() {
            "Option"
        } else {
            "Alt"
        }
    }
}

pub const OS: Os = Os::current();
