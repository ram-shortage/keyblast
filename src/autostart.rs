/// Auto-start at login management for KeyBlast.
///
/// Uses the auto-launch crate for cross-platform login item management.
/// - macOS: LaunchAgent plist in ~/Library/LaunchAgents/
/// - Windows: Registry key in HKCU\Software\Microsoft\Windows\CurrentVersion\Run

use auto_launch::{AutoLaunch, AutoLaunchBuilder};

#[cfg(target_os = "macos")]
use auto_launch::MacOSLaunchMode;

/// Create an AutoLaunch instance configured for KeyBlast.
///
/// Uses the current executable path and platform-appropriate launch mode.
pub fn create_auto_launch() -> Result<AutoLaunch, auto_launch::Error> {
    let app_name = "KeyBlast";
    let app_path = std::env::current_exe()
        .map_err(auto_launch::Error::Io)?
        .to_string_lossy()
        .to_string();

    #[cfg(target_os = "macos")]
    {
        AutoLaunchBuilder::new()
            .set_app_name(app_name)
            .set_app_path(&app_path)
            .set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
            .build()
    }

    #[cfg(not(target_os = "macos"))]
    {
        AutoLaunchBuilder::new()
            .set_app_name(app_name)
            .set_app_path(&app_path)
            .build()
    }
}

/// Check if auto-start at login is currently enabled.
///
/// Returns false if unable to determine (e.g., permission issues).
pub fn is_auto_start_enabled() -> bool {
    create_auto_launch()
        .map(|al| al.is_enabled().unwrap_or(false))
        .unwrap_or(false)
}

/// Enable or disable auto-start at login.
///
/// On macOS: Creates/removes a LaunchAgent plist file.
/// On Windows: Creates/removes a registry entry.
pub fn set_auto_start(enabled: bool) -> Result<(), auto_launch::Error> {
    let auto_launch = create_auto_launch()?;
    if enabled {
        auto_launch.enable()
    } else {
        auto_launch.disable()
    }
}
