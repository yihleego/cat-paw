fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        // Path to the icon file (relative to the project root)
        res.set_icon("build/windows/icon.ico");

        // Compile resources
        if let Err(e) = res.compile() {
            eprintln!("Error compiling Windows resources: {}", e);
            std::process::exit(1);
        }
    }

    // Monitor changes to the icon file
    println!("cargo:rerun-if-changed=build/windows/icon.ico");
}
