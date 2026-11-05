use crate::AppInfo;

pub const MIN_MACOS: &str = "11.0";

pub fn bundle_id(info: &AppInfo) -> String {
    let id: String = info
        .id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '-' })
        .collect();
    format!("net.nativecompute.{id}")
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

pub fn info_plist(info: &AppInfo, executable: &str, icon: &str) -> String {
    let strings = [
        ("CFBundleDevelopmentRegion", "en".to_string()),
        ("CFBundleDisplayName", info.name.to_string()),
        ("CFBundleExecutable", executable.to_string()),
        ("CFBundleIconFile", icon.to_string()),
        ("CFBundleIdentifier", bundle_id(info)),
        ("CFBundleInfoDictionaryVersion", "6.0".to_string()),
        ("CFBundleName", info.name.to_string()),
        ("CFBundlePackageType", "APPL".to_string()),
        ("CFBundleShortVersionString", info.version.to_string()),
        ("CFBundleVersion", info.version.to_string()),
        ("LSApplicationCategoryType", "public.app-category.productivity".to_string()),
        ("LSMinimumSystemVersion", MIN_MACOS.to_string()),
        ("NSHumanReadableCopyright", format!("{} by NativeCompute. MIT licence.", info.name)),
    ];
    let mut body = String::new();
    for (k, v) in strings {
        body.push_str(&format!("\t<key>{k}</key>\n\t<string>{}</string>\n", xml(&v)));
    }
    for k in ["NSHighResolutionCapable", "NSSupportsAutomaticGraphicsSwitching"] {
        body.push_str(&format!("\t<key>{k}</key>\n\t<true/>\n"));
    }
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n<plist version=\"1.0\">\n<dict>\n{body}</dict>\n</plist>\n"
    )
}
