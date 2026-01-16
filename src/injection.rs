/// Keystroke injection for KeyBlast.
///
/// Provides safe keystroke injection that properly handles modifier keys
/// held from hotkey activation and supports configurable typing delay.

use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::thread;
use std::time::Duration;

/// Error type for injection operations.
#[derive(Debug)]
pub struct InjectionError(pub String);

impl std::fmt::Display for InjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for InjectionError {}

impl From<enigo::Error> for InjectionError {
    fn from(e: enigo::Error) -> Self {
        InjectionError(format!("Enigo error: {}", e))
    }
}

/// Handles keystroke injection with proper modifier key handling.
///
/// # Important
///
/// When a hotkey triggers macro playback, modifier keys (Ctrl, Shift, Alt, Meta)
/// may still be physically held. The injector releases these modifiers before
/// typing to prevent interference (e.g., text being capitalized or triggering
/// shortcuts).
pub struct KeystrokeInjector {
    enigo: Enigo,
}

impl KeystrokeInjector {
    /// Create a new KeystrokeInjector.
    ///
    /// # Platform Configuration
    ///
    /// - macOS: Disables automatic permission prompt (we handle it separately)
    /// - macOS: Sets independent_of_keyboard_state for reliable simulation
    /// - All platforms: Releases held keys when dropped (safety net)
    pub fn new() -> Result<Self, InjectionError> {
        #[cfg(target_os = "macos")]
        let settings = Settings {
            // We check permissions ourselves with macos-accessibility-client
            open_prompt_to_get_permissions: false,
            // Simulated input should be independent of physical keyboard state
            independent_of_keyboard_state: true,
            // Release any held keys when the injector is dropped
            release_keys_when_dropped: true,
            ..Settings::default()
        };

        #[cfg(not(target_os = "macos"))]
        let settings = Settings {
            release_keys_when_dropped: true,
            ..Settings::default()
        };

        let enigo = Enigo::new(&settings)?;
        Ok(Self { enigo })
    }

    /// Release common modifier keys that might be held from hotkey activation.
    ///
    /// This is critical for correct macro expansion. Without releasing modifiers:
    /// - Shift held: text becomes CAPITALIZED
    /// - Ctrl held: may trigger shortcuts instead of typing
    /// - Alt/Meta held: may produce alternate characters
    pub fn release_modifiers(&mut self) -> Result<(), InjectionError> {
        self.enigo.key(Key::Control, Direction::Release)?;
        self.enigo.key(Key::Shift, Direction::Release)?;
        self.enigo.key(Key::Alt, Direction::Release)?;
        self.enigo.key(Key::Meta, Direction::Release)?;
        Ok(())
    }

    /// Type text after releasing modifiers.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to type
    /// * `delay_ms` - Delay between keystrokes in milliseconds.
    ///   - `0`: Use bulk text() method for maximum speed
    ///   - `>0`: Type character-by-character with delay (for slow applications)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut injector = KeystrokeInjector::new()?;
    /// // Fast typing
    /// injector.type_text_with_delay("Hello, World!", 0)?;
    /// // Slow typing for problematic applications
    /// injector.type_text_with_delay("Hello, World!", 20)?;
    /// ```
    pub fn type_text_with_delay(&mut self, text: &str, delay_ms: u64) -> Result<(), InjectionError> {
        // Release any modifiers held from hotkey activation
        self.release_modifiers()?;

        // Wait briefly to ensure modifiers are released
        thread::sleep(Duration::from_millis(10));

        if delay_ms == 0 {
            // Fast path: use bulk text() method
            self.enigo.text(text)?;
        } else {
            // Slow path: character-by-character with delay
            for c in text.chars() {
                self.enigo.text(&c.to_string())?;
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }

        Ok(())
    }
}
