// build.rs - Windows resource compilation for icon embedding
fn main() {
    // Only compile Windows resources when targeting Windows
    // IMPORTANT: Use CARGO_CFG_TARGET_OS, not #[cfg(target_os)]
    // build.rs runs on HOST, cfg attributes reflect host OS
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();

        // Set the application icon (shows in Explorer, taskbar, Alt+Tab)
        res.set_icon("assets/icon.ico");

        // Version info auto-populated from Cargo.toml [package]:
        // - FileVersion from version
        // - ProductName from name
        // - FileDescription from description

        res.compile().unwrap();
    }
}
