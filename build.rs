fn main() {
    println!("cargo:rerun-if-changed=assets/icons/app.ico");
    println!("cargo:rerun-if-changed=assets/application.manifest");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        winresource::WindowsResource::new()
            .set_icon("assets/icons/app.ico")
            .set_manifest(include_str!("assets/application.manifest"))
            .set("FileDescription", "Zed localization companion")
            .set("ProductName", "Zed localization companion")
            .compile()
            .expect("Windows icon resource compilation failed");
    }
}
