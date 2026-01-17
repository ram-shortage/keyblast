//! User-facing error notifications for KeyBlast.
//!
//! Provides cross-platform toast notifications for error conditions.
//! Uses notify-rust to abstract macOS/Windows/Linux differences.

use notify_rust::{Notification, Timeout};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimum interval between notifications to prevent spam (3 seconds)
const NOTIFICATION_DEBOUNCE_MS: u64 = 3000;

/// Last notification timestamp for debouncing
static LAST_NOTIFICATION: AtomicU64 = AtomicU64::new(0);

/// Severity levels for error notifications.
#[derive(Debug, Clone, Copy)]
pub enum NotificationSeverity {
    /// Permission issues - persistent notification, user action required
    Permission,
    /// Injection failed - transient notification, informational
    InjectionFailed,
}

impl NotificationSeverity {
    fn timeout(&self) -> Timeout {
        match self {
            // Note: macOS ignores timeout - system controls duration
            NotificationSeverity::Permission => Timeout::Never,
            NotificationSeverity::InjectionFailed => Timeout::Milliseconds(5000),
        }
    }
}

/// Show an error notification to the user.
///
/// Notifications are debounced to prevent spam when multiple failures occur rapidly.
/// Permission errors bypass debouncing since they are critical.
///
/// # Arguments
///
/// * `title` - Notification title (e.g., "KeyBlast")
/// * `message` - Error message to display
/// * `severity` - Determines notification timeout behavior
pub fn show_error(title: &str, message: &str, severity: NotificationSeverity) {
    // Permission errors always show (critical)
    // Other errors are debounced
    if !matches!(severity, NotificationSeverity::Permission) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let last = LAST_NOTIFICATION.load(Ordering::Relaxed);
        if now.saturating_sub(last) < NOTIFICATION_DEBOUNCE_MS {
            // Too soon since last notification, skip
            return;
        }
        LAST_NOTIFICATION.store(now, Ordering::Relaxed);
    }

    let result = Notification::new()
        .summary(title)
        .body(message)
        .appname("KeyBlast")
        .timeout(severity.timeout())
        .show();

    if let Err(e) = result {
        // Fallback to logging if notification fails
        tracing::error!("Notification failed: {} - {} - {}", title, message, e);
    }
}

/// Get platform-specific permission error message.
pub fn permission_error_message() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Accessibility permission required.\n\nGo to System Settings > Privacy & Security > Accessibility to enable KeyBlast."
    }
    #[cfg(target_os = "windows")]
    {
        "Injection may be blocked.\n\nTry running KeyBlast as Administrator for elevated applications."
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        "Permission denied for keystroke injection."
    }
}
