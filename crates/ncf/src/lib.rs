pub mod alerts;
pub mod anim;
pub mod brand;
pub mod bundle;
pub mod caret;
pub mod chrome;
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
#[cfg(not(target_arch = "wasm32"))]
pub mod platform;
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
