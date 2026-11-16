pub mod alerts;
pub mod anim;
pub mod brand;
pub mod bundle;
pub mod caret;
pub mod chrome;
pub mod debug;
pub mod icon;
pub mod icons;
pub mod keys;
pub mod mark;
pub mod os;
pub mod settings;
pub mod splash;
pub mod theme;
pub mod tokens;
pub mod ui;
pub mod update;

#[cfg(not(target_arch = "wasm32"))]
pub mod app;
#[cfg(all(feature = "cloud", not(target_arch = "wasm32")))]
pub mod cloud;
#[cfg(not(target_arch = "wasm32"))]
pub mod platform;
#[cfg(not(target_arch = "wasm32"))]
pub mod updater;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

#[cfg(test)]
mod tests;

pub use eframe;

pub const WEB: bool = cfg!(target_arch = "wasm32");
pub use os::{Os, OS};
pub const NATIVECOMPUTE_URL: &str = tokens::BRAND_URL;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug)]
pub struct AppInfo {
    pub name: &'static str,
    pub id: &'static str,
    pub version: &'static str,
    pub tagline: &'static str,
    pub repo: &'static str,
    pub mark: mark::AppMark,
    pub formerly: &'static [&'static str],
}

impl AppInfo {
    pub fn former_slugs(&self) -> Vec<String> {
        self.formerly.iter().map(|n| slug_of(n)).collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn folder(&self) -> std::path::PathBuf {
        self.folder_in(&platform::data_home(), OS)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn folder_in(&self, home: &std::path::Path, os: Os) -> std::path::PathBuf {
        let fresh = home.join(platform::folder_name(self.name, os));
        if fresh.exists() {
            return fresh;
        }
        self.formerly.iter().map(|n| home.join(platform::folder_name(n, os))).find(|d| d.is_dir()).unwrap_or(fresh)
    }
}

pub fn slug_of(name: &str) -> String {
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

pub mod prelude {
    pub use crate::alerts::{Deck, Tone};
    pub use crate::icons::Icon;
    pub use crate::mark::{line, slab, AppMark, Ink, Part};
    pub use crate::theme::{self, pal};
    pub use crate::{anim, ui, AppInfo};
    #[cfg(not(target_arch = "wasm32"))]
    pub use crate::app::{NcfApp, Shell};
    pub use eframe::egui;
}
