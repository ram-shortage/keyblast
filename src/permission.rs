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
/// On macOS, this will prompt the user to grant permission if not already granted,
/// and print detailed guidance if permission is denied.
/// On other platforms, this always returns `true`.
///
/// # Returns
///
/// `true` if the application has permission (or no permission is needed),
/// `false` if permission was denied or not yet granted on macOS.
#[cfg(target_os = "macos")]
pub fn check_accessibility_permission() -> bool {
    use macos_accessibility_client::accessibility::application_is_trusted_with_prompt;

    let trusted = application_is_trusted_with_prompt();

    if !trusted {
        print_accessibility_guidance();
    }

    trusted
}

#[cfg(target_os = "macos")]
fn print_accessibility_guidance() {
    eprintln!();
    eprintln!("=====================================================================");
    eprintln!("         KeyBlast Accessibility Permission Required                  ");
    eprintln!("=====================================================================");
    eprintln!();
    eprintln!("KeyBlast needs Accessibility permission to inject keystrokes.");
    eprintln!("Without this permission, macros will not work.");
    eprintln!();
    eprintln!("To grant permission:");
    eprintln!();
    eprintln!("  1. Open System Settings (or System Preferences on older macOS)");
    eprintln!("  2. Go to: Privacy & Security -> Accessibility");
    eprintln!("  3. Click the lock icon to make changes (enter your password)");
    eprintln!("  4. Click the '+' button");
    eprintln!("  5. Navigate to KeyBlast and add it");
    eprintln!("  6. Make sure the checkbox next to KeyBlast is checked");
    eprintln!("  7. Restart KeyBlast");
    eprintln!();
    eprintln!("TIP: The system permission dialog may have appeared behind other windows.");
    eprintln!("     Check your other windows or look for a notification.");
    eprintln!();
    eprintln!("KeyBlast will continue running but macros will not work until");
    eprintln!("permission is granted and the app is restarted.");
    eprintln!();
}

#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    // Windows and Linux don't need special permissions for input simulation
    true
}
