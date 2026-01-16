/// Keystroke injection for KeyBlast.
///
/// Provides safe keystroke injection that properly handles modifier keys
/// held from hotkey activation and supports configurable typing delay.

use enigo::{Direction, Enigo, InputError, Key, Keyboard, NewConError, Settings};
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

impl From<NewConError> for InjectionError {
    fn from(e: NewConError) -> Self {
        InjectionError(format!("Failed to create Enigo: {:?}", e))
    }
}

impl From<InputError> for InjectionError {
    fn from(e: InputError) -> Self {
        InjectionError(format!("Input error: {:?}", e))
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

    /// Execute a parsed macro sequence with special keys and text.
    ///
    /// # Arguments
    ///
    /// * `segments` - The parsed macro segments to execute
    /// * `delay_ms` - Delay between keystrokes (0 for bulk typing)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut injector = KeystrokeInjector::new()?;
    /// let segments = parse_macro_sequence("Hello{Enter}World");
    /// injector.execute_sequence(&segments, 0)?;
    /// ```
    pub fn execute_sequence(
        &mut self,
        segments: &[MacroSegment],
        delay_ms: u64,
    ) -> Result<(), InjectionError> {
        // Release any modifiers held from hotkey activation
        self.release_modifiers()?;

        // Wait briefly to ensure modifiers are released
        thread::sleep(Duration::from_millis(10));

        for segment in segments {
            match segment {
                MacroSegment::Text(text) => {
                    if delay_ms == 0 {
                        self.enigo.text(text)?;
                    } else {
                        for c in text.chars() {
                            self.enigo.text(&c.to_string())?;
                            thread::sleep(Duration::from_millis(delay_ms));
                        }
                    }
                }
                MacroSegment::SpecialKey(key) => {
                    self.enigo.key(*key, Direction::Click)?;
                    if delay_ms > 0 {
                        thread::sleep(Duration::from_millis(delay_ms));
                    }
                }
            }
        }

        Ok(())
    }
}

/// A segment of a macro sequence - either plain text or a special key.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroSegment {
    /// Plain text to be typed.
    Text(String),
    /// A special key to be pressed (Enter, Tab, etc.).
    SpecialKey(Key),
}

/// Parse a macro string with escape sequences into segments.
///
/// Special keys are enclosed in braces: `{Enter}`, `{Tab}`, etc.
/// Unknown keys in braces are passed through as literal text.
///
/// # Supported Special Keys (case-insensitive)
///
/// - `{Enter}` or `{Return}` - Enter/Return key
/// - `{Tab}` - Tab key
/// - `{Escape}` or `{Esc}` - Escape key
/// - `{Backspace}` - Backspace key
/// - `{Delete}` or `{Del}` - Delete key
/// - `{Up}` - Up arrow
/// - `{Down}` - Down arrow
/// - `{Left}` - Left arrow
/// - `{Right}` - Right arrow
/// - `{Home}` - Home key
/// - `{End}` - End key
/// - `{PageUp}` or `{PgUp}` - Page Up
/// - `{PageDown}` or `{PgDn}` - Page Down
/// - `{Space}` - Space key
///
/// # Example
///
/// ```ignore
/// let segments = parse_macro_sequence("Hello{Enter}World{Tab}Next");
/// // Returns: [Text("Hello"), SpecialKey(Return), Text("World"), SpecialKey(Tab), Text("Next")]
/// ```
pub fn parse_macro_sequence(input: &str) -> Vec<MacroSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            // Start of potential escape sequence
            let mut key_name = String::new();
            let mut found_close = false;

            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next(); // consume the '}'
                    found_close = true;
                    break;
                }
                key_name.push(chars.next().unwrap());
            }

            if found_close {
                if let Some(key) = special_key_from_name(&key_name) {
                    // Valid special key - flush text buffer first
                    if !current_text.is_empty() {
                        segments.push(MacroSegment::Text(current_text.clone()));
                        current_text.clear();
                    }
                    segments.push(MacroSegment::SpecialKey(key));
                } else {
                    // Unknown key name - treat as literal text (don't crash)
                    current_text.push('{');
                    current_text.push_str(&key_name);
                    current_text.push('}');
                }
            } else {
                // Unclosed brace - treat as literal
                current_text.push('{');
                current_text.push_str(&key_name);
            }
        } else {
            current_text.push(c);
        }
    }

    // Flush any remaining text
    if !current_text.is_empty() {
        segments.push(MacroSegment::Text(current_text));
    }

    segments
}

/// Map a key name to an enigo Key variant.
///
/// Returns `None` for unknown key names.
fn special_key_from_name(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        "enter" | "return" => Some(Key::Return),
        "tab" => Some(Key::Tab),
        "escape" | "esc" => Some(Key::Escape),
        "backspace" => Some(Key::Backspace),
        "delete" | "del" => Some(Key::Delete),
        "up" => Some(Key::UpArrow),
        "down" => Some(Key::DownArrow),
        "left" => Some(Key::LeftArrow),
        "right" => Some(Key::RightArrow),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" | "pgup" => Some(Key::PageUp),
        "pagedown" | "pgdn" => Some(Key::PageDown),
        "space" => Some(Key::Space),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plain_text() {
        let segments = parse_macro_sequence("Hello World");
        assert_eq!(segments, vec![MacroSegment::Text("Hello World".to_string())]);
    }

    #[test]
    fn test_parse_special_keys() {
        let segments = parse_macro_sequence("Hello{Enter}World");
        assert_eq!(
            segments,
            vec![
                MacroSegment::Text("Hello".to_string()),
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::Text("World".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_multiple_special_keys() {
        let segments = parse_macro_sequence("{Tab}Hello{Enter}{Enter}World{Tab}");
        assert_eq!(
            segments,
            vec![
                MacroSegment::SpecialKey(Key::Tab),
                MacroSegment::Text("Hello".to_string()),
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::Text("World".to_string()),
                MacroSegment::SpecialKey(Key::Tab),
            ]
        );
    }

    #[test]
    fn test_parse_case_insensitive() {
        let segments = parse_macro_sequence("{ENTER}{enter}{Enter}");
        assert_eq!(
            segments,
            vec![
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::SpecialKey(Key::Return),
            ]
        );
    }

    #[test]
    fn test_parse_unknown_key_as_literal() {
        let segments = parse_macro_sequence("Hello{Unknown}World");
        assert_eq!(
            segments,
            vec![MacroSegment::Text("Hello{Unknown}World".to_string())]
        );
    }

    #[test]
    fn test_parse_unclosed_brace() {
        let segments = parse_macro_sequence("Hello{Enter");
        assert_eq!(
            segments,
            vec![MacroSegment::Text("Hello{Enter".to_string())]
        );
    }

    #[test]
    fn test_parse_all_special_keys() {
        // Test that all documented special keys are recognized
        let cases = vec![
            ("{enter}", Key::Return),
            ("{return}", Key::Return),
            ("{tab}", Key::Tab),
            ("{escape}", Key::Escape),
            ("{esc}", Key::Escape),
            ("{backspace}", Key::Backspace),
            ("{delete}", Key::Delete),
            ("{del}", Key::Delete),
            ("{up}", Key::UpArrow),
            ("{down}", Key::DownArrow),
            ("{left}", Key::LeftArrow),
            ("{right}", Key::RightArrow),
            ("{home}", Key::Home),
            ("{end}", Key::End),
            ("{pageup}", Key::PageUp),
            ("{pgup}", Key::PageUp),
            ("{pagedown}", Key::PageDown),
            ("{pgdn}", Key::PageDown),
            ("{space}", Key::Space),
        ];

        for (input, expected_key) in cases {
            let segments = parse_macro_sequence(input);
            assert_eq!(
                segments,
                vec![MacroSegment::SpecialKey(expected_key)],
                "Failed for input: {}",
                input
            );
        }
    }
}
