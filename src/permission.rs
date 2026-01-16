/// Accessibility permission checking for KeyBlast.
///
/// On macOS, keystroke injection requires Accessibility permissions.
/// This module provides a cross-platform abstraction for checking these permissions.
///
/// # macOS Requirements
///
/// macOS requires applications to be granted Accessibility permissions in
/// System Preferences > Privacy & Security > Accessibility before they can
/// simulate keyboard input. This function will prompt the user to grant
/// permission if it has not been granted yet.
///
/// # Other Platforms
///
/// Windows and Linux do not require special permissions for input simulation,
/// so this function returns `true` on those platforms.

/// Check if the application has accessibility permission to inject keystrokes.
///
/// On macOS, this will prompt the user to grant permission if not already granted.
/// On other platforms, this always returns `true`.
///
/// # Returns
///
/// `true` if the application has permission (or no permission is needed),
/// `false` if permission was denied or not yet granted on macOS.
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    use macos_accessibility_client::accessibility::application_is_trusted_with_prompt;
    application_is_trusted_with_prompt()
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    // Windows and Linux don't need special permissions for input simulation
    true
}
