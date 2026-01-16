# Phase 6: Platform Polish - Research

**Researched:** 2026-01-16
**Domain:** Cross-platform deployment, auto-launch, permissions UX, visual feedback
**Confidence:** MEDIUM-HIGH

## Summary

Phase 6 focuses on production-ready cross-platform support for KeyBlast. The research covers five key areas: Windows compatibility verification, macOS Accessibility permission UX, auto-start at login functionality, tray icon visual feedback, and code signing/notarization for distribution.

The existing codebase already has solid cross-platform foundations using `tray-icon`, `muda`, `global-hotkey`, and `enigo` crates. Windows compatibility is largely handled by these crates' cross-platform abstractions. The main work involves: adding auto-launch capability with the `auto-launch` crate, improving the macOS Accessibility permission flow with user guidance, and implementing tray icon flashing for macro feedback using `TrayIcon::set_icon()`.

Code signing and notarization are optional for initial release but documented for future distribution readiness.

**Primary recommendation:** Use `auto-launch` crate with `MacOSLaunchMode::LaunchAgent` for macOS and default Windows registry approach. Implement icon flash via timer-based `set_icon()` alternation. Enhance Accessibility permission UX with clear user instructions and retry mechanism.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| auto-launch | 0.6.0 | Auto-start at login | Cross-platform, Tauri uses it internally |
| tray-icon | 0.21 | Tray icon management | Already in use, supports `set_icon()` for flash |
| macos-accessibility-client | 0.0.1 | Accessibility permission check | Already in use, provides prompt |

### Supporting (Already in Project)
| Library | Version | Purpose | Status |
|---------|---------|---------|--------|
| enigo | 0.6 | Keystroke injection | Already working, cross-platform |
| global-hotkey | 0.7 | Global hotkey registration | Already working, cross-platform |
| image | 0.25 | Icon loading | Already in use |

### Optional (Code Signing/Distribution)
| Tool | Purpose | When to Use |
|------|---------|-------------|
| rcodesign (apple-codesign) | macOS code signing from any OS | CI/CD distribution |
| xcrun notarytool | Apple notarization | Native macOS builds |
| signtool.exe | Windows code signing | Windows distribution |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| auto-launch | smappservice-rs | macOS-only but more native UX. Use if macOS polish is priority |
| Timer-based flash | OS notification API | More visible but loses tray context |

**Installation:**
```bash
# Add to Cargo.toml
cargo add auto-launch
```

```toml
[dependencies]
auto-launch = "0.6"
```

## Architecture Patterns

### Recommended Project Structure Addition
```
src/
├── autostart.rs     # Auto-launch management (new)
├── permission.rs    # Accessibility permission (enhance)
├── tray.rs          # Tray icon management (enhance for flash)
└── ...
```

### Pattern 1: Auto-Launch Configuration
**What:** Cross-platform auto-start using AutoLaunchBuilder
**When to use:** Enable/disable auto-start from tray menu

```rust
// Source: https://docs.rs/auto-launch/latest/auto_launch/
use auto_launch::{AutoLaunch, AutoLaunchBuilder, MacOSLaunchMode};

pub fn create_auto_launch() -> Result<AutoLaunch, auto_launch::Error> {
    let app_name = "KeyBlast";
    let app_path = std::env::current_exe()
        .map_err(|e| auto_launch::Error::Io(e))?
        .to_string_lossy()
        .to_string();

    AutoLaunchBuilder::new()
        .set_app_name(app_name)
        .set_app_path(&app_path)
        .set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)
        .set_args(&["--minimized"])  // Start minimized
        .build()
}

pub fn is_auto_start_enabled() -> bool {
    create_auto_launch()
        .map(|al| al.is_enabled().unwrap_or(false))
        .unwrap_or(false)
}

pub fn set_auto_start(enabled: bool) -> Result<(), auto_launch::Error> {
    let auto_launch = create_auto_launch()?;
    if enabled {
        auto_launch.enable()
    } else {
        auto_launch.disable()
    }
}
```

### Pattern 2: Tray Icon Flash Feedback
**What:** Briefly flash tray icon when macro triggers
**When to use:** Visual confirmation of macro execution

```rust
// Source: https://docs.rs/tray-icon/0.21.0/tray_icon/struct.TrayIcon.html
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Flash the tray icon briefly to indicate macro triggered.
/// Alternates between normal icon and a "flash" variant.
pub fn flash_tray_icon(
    tray_icon: &tray_icon::TrayIcon,
    normal_icon: &tray_icon::Icon,
    flash_icon: &tray_icon::Icon,
    flash_count: u32,
    flash_duration_ms: u64,
) {
    for _ in 0..flash_count {
        // Show flash icon
        let _ = tray_icon.set_icon(Some(flash_icon.clone()));
        thread::sleep(Duration::from_millis(flash_duration_ms));

        // Restore normal icon
        let _ = tray_icon.set_icon(Some(normal_icon.clone()));
        thread::sleep(Duration::from_millis(flash_duration_ms));
    }
}

// Alternative: Non-blocking flash using spawn
pub fn flash_tray_icon_async(/* params */) {
    // Clone necessary handles and spawn thread
    // Use channel or atomic to coordinate with main loop
}
```

### Pattern 3: Accessibility Permission UX Enhancement
**What:** Guide user through macOS Accessibility permission grant
**When to use:** On app startup when permission not granted

```rust
// Source: macos-accessibility-client crate + custom UX
#[cfg(target_os = "macos")]
pub fn check_and_guide_accessibility_permission() -> bool {
    use macos_accessibility_client::accessibility::application_is_trusted_with_prompt;

    // First check triggers system prompt if needed
    let trusted = application_is_trusted_with_prompt();

    if !trusted {
        // Show user-friendly guidance via dialog or console
        eprintln!("=== KeyBlast Accessibility Permission Required ===");
        eprintln!("KeyBlast needs Accessibility permission to inject keystrokes.");
        eprintln!();
        eprintln!("Steps to grant permission:");
        eprintln!("1. Open System Settings > Privacy & Security > Accessibility");
        eprintln!("2. Click the '+' button");
        eprintln!("3. Navigate to KeyBlast and add it");
        eprintln!("4. Restart KeyBlast");
        eprintln!();
        eprintln!("The system permission dialog should have appeared.");
        eprintln!("If not, open System Settings manually.");

        // Could also open System Preferences directly:
        // std::process::Command::new("open")
        //     .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
        //     .spawn();
    }

    trusted
}

#[cfg(not(target_os = "macos"))]
pub fn check_and_guide_accessibility_permission() -> bool {
    true  // No permission needed on Windows/Linux
}
```

### Pattern 4: Conditional Platform Code
**What:** Handle platform-specific behavior cleanly
**When to use:** Throughout platform polish code

```rust
// Already used in project - consistent pattern
#[cfg(target_os = "macos")]
fn platform_specific_init() {
    // macOS-specific code
}

#[cfg(target_os = "windows")]
fn platform_specific_init() {
    // Windows-specific code
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_specific_init() {
    // Fallback for other platforms
}
```

### Anti-Patterns to Avoid
- **Don't poll for permission continuously:** Check at startup, guide user, let them restart
- **Don't block UI during icon flash:** Use async/spawn for flash animation
- **Don't hardcode paths:** Use `std::env::current_exe()` for auto-launch
- **Don't use `--deep` for codesign:** Sign components individually, bottom-up

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Auto-start at login | Manual plist/registry | `auto-launch` crate | Handles edge cases, cross-platform |
| macOS permission check | FFI to Accessibility API | `macos-accessibility-client` | Already wrapped, maintained |
| Code signing | Shell scripts | `rcodesign` or `xcrun` | Correct flags, error handling |
| Windows registry | Manual winreg | `auto-launch` crate | Handles registry paths, approval flags |

**Key insight:** Auto-launch appears simple (write a plist/registry key) but has many edge cases: detecting if already enabled, handling system disabling, paths with spaces, proper uninstall cleanup. The `auto-launch` crate handles these.

## Common Pitfalls

### Pitfall 1: Auto-Launch Path Issues
**What goes wrong:** App doesn't auto-start because path is incorrect
**Why it happens:** Relative paths, paths with spaces, dev vs release paths
**How to avoid:** Always use `std::env::current_exe()` for absolute path
**Warning signs:** Works in dev, fails in release; works without spaces in path

### Pitfall 2: macOS Accessibility Permission Not Taking Effect
**What goes wrong:** User grants permission but app still can't inject
**Why it happens:** TCC cache, need to restart app after granting
**How to avoid:** Clear guidance to restart after granting; consider quit button
**Warning signs:** Permission shows as granted in System Settings but enigo fails

### Pitfall 3: Tray Icon Flash Blocks Event Loop
**What goes wrong:** App becomes unresponsive during icon flash
**Why it happens:** `thread::sleep` in main thread
**How to avoid:** Use separate thread or async task for flash animation
**Warning signs:** Can't trigger new hotkeys during flash, menu won't open

### Pitfall 4: Windows UIPI Blocking Injection
**What goes wrong:** Keystroke injection fails in elevated windows
**Why it happens:** Windows blocks medium-integrity apps from sending to high-integrity
**How to avoid:** Document limitation; not fixable without running elevated
**Warning signs:** Works in Notepad, fails in admin Command Prompt

### Pitfall 5: Code Signing Identity Mismatch
**What goes wrong:** Notarization fails with "invalid signature"
**Why it happens:** Mixing signing identities, signing top-level before nested
**How to avoid:** Sign bottom-up (helpers, frameworks, then app), same identity
**Warning signs:** `codesign -vvv` shows different identities in bundle

### Pitfall 6: Windows SmartScreen Warning
**What goes wrong:** Users see "Windows protected your PC" warning
**Why it happens:** Unsigned or newly-signed app with no reputation
**How to avoid:** Sign with OV/EV cert, build reputation through downloads
**Warning signs:** Fresh installs always show warning

## Code Examples

Verified patterns from official sources:

### Menu Item for Auto-Start Toggle
```rust
// Add to tray menu building (src/tray.rs enhancement)
use muda::{CheckMenuItem, MenuId};

// In build_menu function:
let auto_start_enabled = crate::autostart::is_auto_start_enabled();
let auto_start_item = CheckMenuItem::new(
    "Start at Login",
    true,  // enabled
    auto_start_enabled,
    None::<muda::accelerator::Accelerator>,
);
let auto_start_id = auto_start_item.id().clone();

// Add to MenuIds struct:
pub struct MenuIds {
    // ... existing fields
    pub auto_start: MenuId,
}
```

### Handle Auto-Start Toggle Event
```rust
// In about_to_wait event handling (src/main.rs):
if event.id == self.menu_ids.auto_start {
    let currently_enabled = crate::autostart::is_auto_start_enabled();
    match crate::autostart::set_auto_start(!currently_enabled) {
        Ok(()) => {
            println!("Auto-start {}", if !currently_enabled { "enabled" } else { "disabled" });
            // Update checkbox state
            for item in self.menu.items() {
                if let muda::MenuItemKind::Check(check_item) = item {
                    if check_item.id() == &self.menu_ids.auto_start {
                        check_item.set_checked(!currently_enabled);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to toggle auto-start: {}", e);
        }
    }
}
```

### Flash Icon After Macro Execution
```rust
// In hotkey handler after successful injection:
if injection_success {
    println!("Injection complete");

    // Flash tray icon for visual feedback
    if let Some(ref tray_icon) = self._tray_icon {
        // Clone what we need for the flash thread
        // Note: TrayIcon is not Send, so we need to flash in main thread
        // Use a flag/counter approach checked in about_to_wait
        self.flash_remaining = 4;  // 2 cycles of on/off
    }
}

// In about_to_wait, check flash_remaining and alternate icons
```

### Load Two Icon Variants
```rust
// In src/tray.rs - load normal and flash icons
pub fn load_icons() -> (Icon, Icon) {
    let normal_bytes = include_bytes!("../assets/icon.png");
    let flash_bytes = include_bytes!("../assets/icon-flash.png");

    let normal = load_icon_from_bytes(normal_bytes);
    let flash = load_icon_from_bytes(flash_bytes);

    (normal, flash)
}

fn load_icon_from_bytes(bytes: &[u8]) -> Icon {
    let image = image::load_from_memory(bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SMLoginItemSetEnabled | SMAppService | macOS 13 (2022) | New API for login items |
| altool for notarization | notarytool | Nov 2023 | altool deprecated |
| EV cert instant SmartScreen bypass | Reputation-based for all | March 2024 | EV no longer special |
| Hardcoded plist paths | auto-launch crate | Ongoing | Handles edge cases |

**Deprecated/outdated:**
- `altool`: Replaced by `notarytool` for Apple notarization
- `SMLoginItemSetEnabled`: Replaced by `SMAppService` on macOS 13+
- Instant SmartScreen bypass with EV certs: No longer works as of March 2024

## Distribution Notes (Future Reference)

### macOS Notarization Steps (Not Required for Phase 6)
```bash
# 1. Sign with hardened runtime
codesign -s "Developer ID Application: YOUR_NAME" \
    -o runtime \
    --entitlements entitlements.plist \
    KeyBlast.app

# 2. Create zip for submission
ditto -c -k --keepParent KeyBlast.app KeyBlast.zip

# 3. Submit for notarization
xcrun notarytool submit KeyBlast.zip \
    --keychain-profile "notary-profile" \
    --wait

# 4. Staple ticket
xcrun stapler staple KeyBlast.app
```

### Windows Signing (Not Required for Phase 6)
```bash
# Requires Windows SDK and code signing certificate
signtool sign /f certificate.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 keyblast.exe
```

## Open Questions

Things that couldn't be fully resolved:

1. **Icon flash thread safety**
   - What we know: `TrayIcon` is not `Send`/`Sync`
   - What's unclear: Best pattern for non-blocking flash in winit event loop
   - Recommendation: Use counter/flag in app state, check in `about_to_wait`

2. **Windows UIPI limitation**
   - What we know: Cannot inject into elevated windows from normal process
   - What's unclear: Whether users will encounter this often
   - Recommendation: Document limitation; running elevated creates other issues

3. **macOS Accessibility permission persistence**
   - What we know: Permission persists across launches
   - What's unclear: Behavior on app update/path change
   - Recommendation: Check at every startup, guide if not granted

## Sources

### Primary (HIGH confidence)
- [tray-icon docs](https://docs.rs/tray-icon/0.21.0/tray_icon/struct.TrayIcon.html) - `set_icon()` method verified
- [auto-launch docs](https://docs.rs/auto-launch/latest/auto_launch/) - API and platform modes
- [macos-accessibility-client GitHub](https://github.com/next-slide-please/macos-accessibility-client) - Usage pattern

### Secondary (MEDIUM confidence)
- [macOS code signing gist](https://gist.github.com/rsms/929c9c2fec231f0cf843a1a746a416f5) - Comprehensive signing guide
- [Apple notarization overview](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution) - Requirements
- [rcodesign docs](https://gregoryszorc.com/docs/apple-codesign/stable/apple_codesign_rcodesign_notarizing.html) - Rust-based signing

### Tertiary (LOW confidence - for context)
- Windows SmartScreen behavior changes - Multiple forum posts, may change
- smappservice-rs as alternative - Less documented, newer

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Using established crates already in project
- Auto-launch: HIGH - Well-documented crate with Tauri backing
- Icon flash: MEDIUM - API verified, implementation pattern is custom
- Accessibility UX: MEDIUM - API works, UX guidance is best practice
- Code signing: MEDIUM - Not implementing now, documented for reference

**Research date:** 2026-01-16
**Valid until:** 60 days (crate versions stable, signing requirements may update)
