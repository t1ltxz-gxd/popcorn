fn main() {
    // The exe icon is a Windows-only feature, we do nothing on other platforms.
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not specified");
    let png_path = std::path::Path::new(&manifest_dir)
        .join("assets")
        .join("favicon.png");

    println!("cargo:rerun-if-changed=assets/favicon.png");

    if !png_path.exists() {
        println!("cargo:warning=assets/favicon.png not found – the exe icon will not be installed");
        return;
    }

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not specified");
    let ico_path = std::path::Path::new(&out_dir).join("favicon.ico");

    let image = match image::open(&png_path) {
        Ok(img) => img,
        Err(e) => {
            println!("cargo:warning=Unable to open assets/favicon.png: {e}");
            return;
        }
    };

    if let Err(e) = image.save_with_format(&ico_path, image::ImageFormat::Ico) {
        println!("cargo:warning=Failed to convert favicon.png to ICO: {e}");
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico_path.to_str().expect("The path to ICO is invalid as UTF-8"));
    if let Err(e) = res.compile() {
        println!("cargo:warning=Failed to embed icon in exe: {e}");
    }
}
