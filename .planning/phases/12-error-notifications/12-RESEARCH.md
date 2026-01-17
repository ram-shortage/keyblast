# Phase 12: Error Notifications - Research

**Researched:** 2026-01-17
**Domain:** Cross-platform desktop notifications for error handling
**Confidence:** HIGH

## Summary

KeyBlast needs to show tray notifications when keystroke injection fails, when macOS Accessibility permissions are missing, and when Windows UIPI blocks injection. The standard approach is to use the `notify-rust` crate (v4.11.7), which provides cross-platform desktop notification support for macOS, Windows, and Linux through a unified API.

The codebase already has well-defined error types (`InjectionError`) and error handling points throughout `main.rs`. The implementation will add a notification module that wraps `notify-rust` and integrate it at the existing `eprintln!` error sites. Error detection is straightforward for macOS (enigo's `NewConError::NoPermission`), but Windows UIPI failures are undetectable at the API level - we can only notify on general injection failures.

**Primary recommendation:** Use `notify-rust` crate with persistent notifications for permission errors and short-duration toasts for injection failures.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| notify-rust | 4.11.7 | Cross-platform desktop notifications | De facto standard with 19k dependents, supports macOS/Windows/Linux, used by Tauri |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| (none needed) | - | - | notify-rust is self-contained for basic notifications |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| notify-rust | winrt-notification (Windows only) | More Windows features but no cross-platform |
| notify-rust | mac-notification-sys (macOS only) | Direct NSNotification access but no cross-platform |
| notify-rust | native-dialog | Modal dialogs, not toast notifications |

**Installation:**
```bash
cargo add notify-rust
```

## Architecture Patterns

### Recommended Module Structure
```
src/
├── notification.rs  # NEW: Notification abstraction
├── injection.rs     # Existing: Error types already defined
├── permission.rs    # Existing: macOS permission checking
└── main.rs          # Integrate notifications at error sites
```

### Pattern 1: Notification Module Abstraction
**What:** Create a thin wrapper around notify-rust to standardize error notifications
**When to use:** All error notification scenarios
**Example:**
```rust
// src/notification.rs
// Source: https://docs.rs/notify-rust/latest/notify_rust/

use notify_rust::{Notification, Timeout};

/// Show an error notification to the user.
/// For critical errors (permissions), use longer timeout.
/// For transient errors (injection failed), use shorter timeout.
pub fn show_error(title: &str, message: &str, is_critical: bool) {
    let timeout = if is_critical {
        Timeout::Never  // User must dismiss permission errors
    } else {
        Timeout::Milliseconds(5000)  // 5 seconds for transient errors
    };

    let result = Notification::new()
        .summary(title)
        .body(message)
        .timeout(timeout)
        .show();

    if let Err(e) = result {
        // Fallback to eprintln if notification fails
        eprintln!("Notification error: {} - {} - {}", title, message, e);
    }
}
```

### Pattern 2: Error Classification
**What:** Classify errors by severity to determine notification behavior
**When to use:** Deciding notification duration and persistence
**Example:**
```rust
pub enum ErrorSeverity {
    /// Permission issues - persistent notification, user action required
    Permission,
    /// Injection failed - transient notification, informational
    InjectionFailed,
}

impl ErrorSeverity {
    pub fn timeout(&self) -> Timeout {
        match self {
            ErrorSeverity::Permission => Timeout::Never,
            ErrorSeverity::InjectionFailed => Timeout::Milliseconds(5000),
        }
    }
}
```

### Pattern 3: Conditional Compilation for Platform-Specific Messages
**What:** Use `#[cfg]` attributes for platform-specific error details
**When to use:** When error message or behavior differs by platform
**Example:**
```rust
pub fn permission_error_message() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Accessibility permission required. Go to System Settings > Privacy & Security > Accessibility to enable."
    }
    #[cfg(target_os = "windows")]
    {
        "Injection blocked. Try running as Administrator for elevated applications."
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "Permission denied for keystroke injection."
    }
}
```

### Anti-Patterns to Avoid
- **Notification spam:** Don't show a notification for every keystroke failure in a macro - batch or debounce
- **Auto-dismissing critical errors:** Permission errors should persist until dismissed
- **Blocking the event loop:** Use non-blocking `show()`, not `show_and_wait_for_action()`

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Desktop notifications | Platform-specific FFI | notify-rust | Handles macOS NSNotification, Windows Toast, Linux DBus |
| Permission detection | Custom system calls | enigo's NewConError::NoPermission + macos-accessibility-client | Already in codebase |
| Notification timeout | Manual timer threads | notify-rust Timeout enum | Built-in, works cross-platform |

**Key insight:** notify-rust abstracts significant platform complexity. macOS uses NSUserNotificationCenter (deprecated) or UNUserNotificationCenter, Windows uses WinRT Toast API, Linux uses D-Bus. Hand-rolling this would require separate implementations per platform.

## Common Pitfalls

### Pitfall 1: macOS Notification Location During Development
**What goes wrong:** Notifications don't appear during `cargo run`
**Why it happens:** macOS usernoted daemon restricts notifications from binaries in certain locations (like target/debug)
**How to avoid:**
- Run from `/tmp`, `/usr/local/bin`, or `/Applications`
- Use `cargo install` then run the installed binary
- This is development-only; distributed apps work fine
**Warning signs:** `show()` returns Ok but no notification appears

### Pitfall 2: Windows UIPI Detection is Impossible
**What goes wrong:** Cannot detect if injection was blocked by UIPI vs other failure
**Why it happens:** Windows SendInput silently fails for UIPI - GetLastError returns success
**How to avoid:**
- Cannot be avoided at API level
- Show generic "injection failed" notification
- Add guidance about running as Administrator
**Warning signs:** Injection returns Ok(0) but nothing was typed

### Pitfall 3: Notification Timeout on macOS is Ignored
**What goes wrong:** Setting timeout has no effect on macOS
**Why it happens:** macOS NSNotification doesn't support programmatic timeout
**How to avoid:**
- Accept that macOS controls notification duration
- Use Timeout::Default on macOS
- Document this limitation
**Warning signs:** Notifications dismiss faster/slower than expected on macOS

### Pitfall 4: Auto-Dismissing Permission Errors
**What goes wrong:** User doesn't notice critical permission error
**Why it happens:** UX research shows error toasts that auto-dismiss are often missed
**How to avoid:**
- Use `Timeout::Never` for permission errors
- Let user dismiss manually
**Warning signs:** Users report "nothing happened" when permissions are denied

### Pitfall 5: Bundle Identifier Issues on macOS
**What goes wrong:** Notification settings don't persist between runs
**Why it happens:** Without stable bundle ID, macOS treats each run as new app
**How to avoid:**
- Set `.appname()` consistently
- For distributed app, ensure Info.plist has CFBundleIdentifier
**Warning signs:** User notification preferences reset

## Code Examples

Verified patterns from official sources:

### Basic Notification (Cross-Platform)
```rust
// Source: https://docs.rs/notify-rust/latest/notify_rust/
use notify_rust::Notification;

Notification::new()
    .summary("KeyBlast Error")
    .body("Keystroke injection failed")
    .show()?;
```

### Notification with Timeout
```rust
// Source: https://docs.rs/notify-rust/latest/notify_rust/enum.Timeout.html
use notify_rust::{Notification, Timeout};

Notification::new()
    .summary("KeyBlast")
    .body("Macro execution failed")
    .timeout(Timeout::Milliseconds(5000))  // 5 seconds
    .show()?;
```

### macOS-Specific Example
```rust
// Source: https://github.com/hoodie/notify-rust/blob/main/examples/mac.rs
#[cfg(target_os = "macos")]
fn show_macos_notification() -> Result<(), Box<dyn std::error::Error>> {
    use notify_rust::Notification;

    Notification::new()
        .summary("KeyBlast")
        .body("Accessibility permission required")
        .appname("KeyBlast")
        .show()?;

    Ok(())
}
```

### Detecting Permission Errors (from existing enigo usage)
```rust
// Source: https://docs.rs/enigo/latest/enigo/enum.NewConError.html
// Already in codebase: src/injection.rs

use enigo::{NewConError, Enigo, Settings};

match Enigo::new(&settings) {
    Ok(enigo) => { /* success */ }
    Err(NewConError::NoPermission) => {
        // Show permission notification
        show_error("Permission Denied", "...", ErrorSeverity::Permission);
    }
    Err(e) => {
        // Other initialization error
        show_error("Initialization Failed", &format!("{:?}", e), ErrorSeverity::InjectionFailed);
    }
}
```

### Integration at Error Sites (pattern for main.rs)
```rust
// Pattern for existing error handling in main.rs
// Replace eprintln! with notification + logging

// Before:
eprintln!("Injection failed: {}", e);

// After:
error!("Injection failed: {}", e);  // Keep logging
notification::show_error(
    "KeyBlast",
    &format!("Macro injection failed: {}", e),
    false,  // not critical
);
```

## Error Detection Strategy

### macOS Accessibility Permission
| Detection Point | Method | Confidence |
|-----------------|--------|------------|
| App startup | `macos-accessibility-client::application_is_trusted_with_prompt()` | HIGH - already in codebase |
| Enigo creation | `NewConError::NoPermission` | HIGH - explicit error variant |
| Injection failure | `InputError::Simulate` | MEDIUM - generic failure, could be permission |

### Windows UIPI
| Detection Point | Method | Confidence |
|-----------------|--------|------------|
| SendInput blocked | Cannot detect | LOW - Windows API provides no indication |
| Injection failure | `InputError::Simulate` | LOW - same error for all failures |

**Recommendation:** On Windows, show generic "injection failed" message with suggestion to run as Administrator. Cannot distinguish UIPI from other failures.

### General Injection Failures
| Detection Point | Method | Confidence |
|-----------------|--------|------------|
| Text injection | `InputError::Simulate` | HIGH - clear failure |
| Key injection | `InputError::Simulate` | HIGH - clear failure |
| Mapping errors | `InputError::Mapping` | HIGH - keymap issue |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| NSUserNotificationCenter (macOS) | UNUserNotificationCenter | macOS 10.14+ | notify-rust abstracts this |
| Custom Win32 balloon tips | WinRT Toast API | Windows 8+ | notify-rust uses winrt-notification |

**Deprecated/outdated:**
- NSUserNotificationCenter: Deprecated in macOS 10.14, but still works. notify-rust handles this.
- Win32 balloon notifications: Legacy, but still functional. Modern apps use Toast API.

## Notification UX Recommendations

Based on UX research for error notifications:

| Error Type | Timeout | Positioning | User Action |
|------------|---------|-------------|-------------|
| Permission missing | Never (persistent) | System default | Dismiss manually |
| Injection failed | 5000ms | System default | Optional dismiss |
| Clipboard error | 3000ms | System default | Auto-dismiss |

**Key UX principles:**
1. Error notifications should NOT auto-dismiss for critical issues
2. Keep messages concise - title + 1-2 sentences
3. Include actionable guidance (e.g., "Try running as Administrator")
4. Don't spam - batch multiple failures or debounce

## Open Questions

Things that couldn't be fully resolved:

1. **Notification rate limiting**
   - What we know: Multiple rapid injection failures could spam notifications
   - What's unclear: Optimal debounce interval
   - Recommendation: Track last notification time, suppress if <3 seconds

2. **macOS notification appearance during development**
   - What we know: Binaries in target/debug may not show notifications
   - What's unclear: Exact usernoted rules for path restrictions
   - Recommendation: Document this quirk, test with `cargo install`

3. **Windows UIPI detection**
   - What we know: Cannot detect UIPI-specific failures
   - What's unclear: If there's any workaround
   - Recommendation: Accept limitation, show generic error with admin hint

## Sources

### Primary (HIGH confidence)
- [notify-rust docs.rs](https://docs.rs/notify-rust/latest/notify_rust/) - API documentation
- [notify-rust GitHub](https://github.com/hoodie/notify-rust) - Platform support, examples
- [enigo NewConError](https://docs.rs/enigo/latest/enigo/enum.NewConError.html) - Error types
- [enigo InputError](https://docs.rs/enigo/latest/enigo/enum.InputError.html) - Error types

### Secondary (MEDIUM confidence)
- [Windows UIPI Microsoft Docs](https://learn.microsoft.com/en-us/archive/blogs/luisdem/uipi-user-interface-privilege-isolation) - UIPI behavior
- [mac-notification-sys GitHub](https://github.com/h4llow3En/mac-notification-sys) - macOS notification backend
- [Smashing Magazine Notification UX](https://www.smashingmagazine.com/2025/07/design-guidelines-better-notifications-ux/) - UX best practices

### Tertiary (LOW confidence)
- [notify-rust macOS issue #132](https://github.com/hoodie/notify-rust/issues/132) - Development path issues

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - notify-rust is clearly the standard, 19k dependents
- Architecture: HIGH - Pattern is straightforward wrapper + integration
- Pitfalls: MEDIUM - macOS dev issues from GitHub issues, UIPI from MS docs

**Research date:** 2026-01-17
**Valid until:** 2026-03-17 (60 days - notify-rust is stable, slow-moving)
