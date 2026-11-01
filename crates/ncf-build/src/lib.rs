pub fn windows(app_name: &str, ico: &str) {
    println!("cargo:rerun-if-changed={ico}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico);
    res.set("ProductName", app_name);
    res.set("FileDescription", app_name);
    res.set("CompanyName", "NativeCompute");
    res.set("LegalCopyright", "Copyright (c) NativeCompute");
    if let Err(e) = res.compile() {
        panic!("could not embed the Windows icon: {e}");
    }
}
