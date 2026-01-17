/// Keystroke injection for KeyBlast.
///
/// Provides safe keystroke injection that properly handles modifier keys
/// held from hotkey activation and supports configurable typing delay.

use arboard::Clipboard;
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

        // Wait for modifiers to fully release (macOS needs longer)
        thread::sleep(Duration::from_millis(50));

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

        // Wait for modifiers to fully release (macOS needs longer)
        thread::sleep(Duration::from_millis(50));

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
                // New segment types - execution handled in Plan 08-02
                MacroSegment::Delay(ms) => {
                    thread::sleep(Duration::from_millis(*ms));
                }
                MacroSegment::KeyDown(key) => {
                    self.enigo.key(*key, Direction::Press)?;
                }
                MacroSegment::KeyUp(key) => {
                    self.enigo.key(*key, Direction::Release)?;
                }
                MacroSegment::Paste => {
                    // Read clipboard and type contents
                    let mut clipboard = Clipboard::new()
                        .map_err(|e| InjectionError(format!("Clipboard error: {}", e)))?;

                    match clipboard.get_text() {
                        Ok(text) => {
                            if delay_ms == 0 {
                                self.enigo.text(&text)?;
                            } else {
                                for c in text.chars() {
                                    self.enigo.text(&c.to_string())?;
                                    thread::sleep(Duration::from_millis(delay_ms));
                                }
                            }
                        }
                        Err(e) => {
                            // Log but don't fail - clipboard might be empty or inaccessible
                            eprintln!("Warning: Could not read clipboard: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a single macro segment.
    ///
    /// Unlike execute_sequence, this does NOT:
    /// - Release modifiers (caller must handle once at start)
    /// - Wait between segments (caller handles timing)
    ///
    /// Used by async execution where timing is managed by worker thread.
    ///
    /// # Arguments
    ///
    /// * `segment` - The single segment to execute
    ///
    /// # Example
    ///
    /// ```ignore
    /// // First, prepare for injection (releases modifiers)
    /// injector.prepare_for_injection()?;
    ///
    /// // Then execute segments one at a time
    /// for segment in segments {
    ///     injector.execute_single_segment(&segment)?;
    /// }
    /// ```
    pub fn execute_single_segment(&mut self, segment: &MacroSegment) -> Result<(), InjectionError> {
        match segment {
            MacroSegment::Text(text) => {
                self.enigo.text(text)?;
            }
            MacroSegment::SpecialKey(key) => {
                self.enigo.key(*key, Direction::Click)?;
            }
            // New segment types - execution handled in Plan 08-02
            MacroSegment::Delay(ms) => {
                thread::sleep(Duration::from_millis(*ms));
            }
            MacroSegment::KeyDown(key) => {
                self.enigo.key(*key, Direction::Press)?;
            }
            MacroSegment::KeyUp(key) => {
                self.enigo.key(*key, Direction::Release)?;
            }
            MacroSegment::Paste => {
                // Read clipboard and type contents
                let mut clipboard = Clipboard::new()
                    .map_err(|e| InjectionError(format!("Clipboard error: {}", e)))?;

                match clipboard.get_text() {
                    Ok(text) => {
                        self.enigo.text(&text)?;
                    }
                    Err(e) => {
                        // Log but don't fail - clipboard might be empty or inaccessible
                        eprintln!("Warning: Could not read clipboard: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    /// Release modifiers and wait for them to take effect.
    ///
    /// Call once at the start of async execution before processing segments.
    /// This handles the modifier release that execute_sequence does internally.
    ///
    /// # Why this is needed
    ///
    /// When a hotkey triggers macro playback, modifier keys (Ctrl, Shift, etc.)
    /// may still be physically held. We need to release them before typing.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // At start of macro execution
    /// injector.prepare_for_injection()?;
    ///
    /// // Now safe to execute segments
    /// for segment in segments {
    ///     injector.execute_single_segment(&segment)?;
    /// }
    /// ```
    pub fn prepare_for_injection(&mut self) -> Result<(), InjectionError> {
        self.release_modifiers()?;
        thread::sleep(Duration::from_millis(50));
        Ok(())
    }
}

/// A segment of a macro sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum MacroSegment {
    /// Plain text to be typed.
    Text(String),
    /// A special key to be pressed (Enter, Tab, etc.).
    SpecialKey(Key),
    /// Pause execution for N milliseconds.
    Delay(u64),
    /// Press and hold a modifier key.
    KeyDown(Key),
    /// Release a modifier key.
    KeyUp(Key),
    /// Paste current clipboard contents as text.
    Paste,
}

/// Parse a macro string with escape sequences into segments.
///
/// Special keys are enclosed in braces: `{Enter}`, `{Tab}`, etc.
/// Unknown keys in braces are passed through as literal text.
///
/// # Supported Commands (case-insensitive)
///
/// ## Special Keys
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
/// ## Extended Commands
/// - `{Delay N}` - Pause for N milliseconds
/// - `{KeyDown key}` - Press and hold a modifier key
/// - `{KeyUp key}` - Release a modifier key
/// - `{Paste}` - Paste clipboard contents
///
/// ## Escape Sequences
/// - `{{` - Literal `{` character
/// - `}}` - Literal `}` character
///
/// # Example
///
/// ```ignore
/// let segments = parse_macro_sequence("Hello{Enter}World{Tab}Next");
/// // Returns: [Text("Hello"), SpecialKey(Return), Text("World"), SpecialKey(Tab), Text("Next")]
///
/// let segments = parse_macro_sequence("Wait{Delay 500}Done");
/// // Returns: [Text("Wait"), Delay(500), Text("Done")]
///
/// let segments = parse_macro_sequence("{{braces}}");
/// // Returns: [Text("{braces}")]
/// ```
pub fn parse_macro_sequence(input: &str) -> Vec<MacroSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            // Check for escaped brace `{{`
            if chars.peek() == Some(&'{') {
                chars.next(); // consume second '{'
                current_text.push('{');
                continue;
            }

            // Start of command
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
                // Try to parse as command
                if let Some(segment) = parse_command(&key_name) {
                    flush_text(&mut current_text, &mut segments);
                    segments.push(segment);
                } else {
                    // Unknown command - treat as literal
                    current_text.push('{');
                    current_text.push_str(&key_name);
                    current_text.push('}');
                }
            } else {
                // Unclosed brace - treat as literal
                current_text.push('{');
                current_text.push_str(&key_name);
            }
        } else if c == '}' {
            // Check for escaped brace `}}`
            if chars.peek() == Some(&'}') {
                chars.next(); // consume second '}'
                current_text.push('}');
                continue;
            }
            // Lone '}' - treat as literal
            current_text.push(c);
        } else {
            current_text.push(c);
        }
    }

    // Flush any remaining text
    flush_text(&mut current_text, &mut segments);

    segments
}

/// Flush accumulated text to the segments vector.
fn flush_text(current_text: &mut String, segments: &mut Vec<MacroSegment>) {
    if !current_text.is_empty() {
        segments.push(MacroSegment::Text(current_text.clone()));
        current_text.clear();
    }
}

/// Parse a command string (contents between `{` and `}`) into a MacroSegment.
///
/// Returns `None` if the command is not recognized (will be treated as literal text).
fn parse_command(key_name: &str) -> Option<MacroSegment> {
    // Split on first space for parameterized commands
    let parts: Vec<&str> = key_name.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim());

    match command.as_str() {
        "delay" => {
            // {Delay N} - requires numeric argument
            arg.and_then(|s| s.parse::<u64>().ok())
                .map(MacroSegment::Delay)
        }
        "keydown" => {
            // {KeyDown key} - requires modifier key name
            arg.and_then(modifier_key_from_name)
                .map(MacroSegment::KeyDown)
        }
        "keyup" => {
            // {KeyUp key} - requires modifier key name
            arg.and_then(modifier_key_from_name)
                .map(MacroSegment::KeyUp)
        }
        "paste" => {
            // {Paste} - no argument needed
            Some(MacroSegment::Paste)
        }
        _ => {
            // Try as a special key (Enter, Tab, etc.)
            special_key_from_name(key_name).map(MacroSegment::SpecialKey)
        }
    }
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

/// Map a modifier key name to an enigo Key variant.
///
/// Returns `None` for unknown modifier key names.
/// Used for `{KeyDown key}` and `{KeyUp key}` commands.
fn modifier_key_from_name(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        "ctrl" | "control" => Some(Key::Control),
        "shift" => Some(Key::Shift),
        "alt" => Some(Key::Alt),
        "meta" | "win" | "cmd" | "command" | "super" => Some(Key::Meta),
        "lctrl" | "lcontrol" => Some(Key::LControl),
        "rctrl" | "rcontrol" => Some(Key::RControl),
        "lshift" => Some(Key::LShift),
        "rshift" => Some(Key::RShift),
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

    // === DSL Extension Tests (08-01) ===

    // Brace escape tests
    #[test]
    fn test_parse_escaped_open_brace() {
        let segments = parse_macro_sequence("{{");
        assert_eq!(segments, vec![MacroSegment::Text("{".to_string())]);
    }

    #[test]
    fn test_parse_escaped_close_brace() {
        let segments = parse_macro_sequence("}}");
        assert_eq!(segments, vec![MacroSegment::Text("}".to_string())]);
    }

    #[test]
    fn test_parse_escaped_braces_around_text() {
        let segments = parse_macro_sequence("{{foo}}");
        assert_eq!(segments, vec![MacroSegment::Text("{foo}".to_string())]);
    }

    #[test]
    fn test_parse_escaped_braces_mixed_with_text() {
        let segments = parse_macro_sequence("a{{b}}c");
        assert_eq!(segments, vec![MacroSegment::Text("a{b}c".to_string())]);
    }

    // Delay tests
    #[test]
    fn test_parse_delay() {
        let segments = parse_macro_sequence("{Delay 500}");
        assert_eq!(segments, vec![MacroSegment::Delay(500)]);
    }

    #[test]
    fn test_parse_delay_case_insensitive() {
        let segments = parse_macro_sequence("{delay 100}");
        assert_eq!(segments, vec![MacroSegment::Delay(100)]);
    }

    #[test]
    fn test_parse_delay_missing_arg_literal() {
        let segments = parse_macro_sequence("{Delay}");
        assert_eq!(segments, vec![MacroSegment::Text("{Delay}".to_string())]);
    }

    #[test]
    fn test_parse_delay_non_numeric_literal() {
        let segments = parse_macro_sequence("{Delay abc}");
        assert_eq!(
            segments,
            vec![MacroSegment::Text("{Delay abc}".to_string())]
        );
    }

    // KeyDown/KeyUp tests
    #[test]
    fn test_parse_keydown_ctrl() {
        let segments = parse_macro_sequence("{KeyDown Ctrl}");
        assert_eq!(segments, vec![MacroSegment::KeyDown(Key::Control)]);
    }

    #[test]
    fn test_parse_keydown_shift_case_insensitive() {
        let segments = parse_macro_sequence("{keydown shift}");
        assert_eq!(segments, vec![MacroSegment::KeyDown(Key::Shift)]);
    }

    #[test]
    fn test_parse_keyup_alt() {
        let segments = parse_macro_sequence("{KeyUp Alt}");
        assert_eq!(segments, vec![MacroSegment::KeyUp(Key::Alt)]);
    }

    #[test]
    fn test_parse_keydown_invalid_key_literal() {
        let segments = parse_macro_sequence("{KeyDown Unknown}");
        assert_eq!(
            segments,
            vec![MacroSegment::Text("{KeyDown Unknown}".to_string())]
        );
    }

    #[test]
    fn test_parse_keydown_meta_variants() {
        // All these should map to Key::Meta
        for name in ["Meta", "Win", "Cmd", "Command", "Super"] {
            let input = format!("{{KeyDown {}}}", name);
            let segments = parse_macro_sequence(&input);
            assert_eq!(
                segments,
                vec![MacroSegment::KeyDown(Key::Meta)],
                "Failed for: {}",
                name
            );
        }
    }

    #[test]
    fn test_parse_keydown_left_right_variants() {
        let segments = parse_macro_sequence("{KeyDown LCtrl}");
        assert_eq!(segments, vec![MacroSegment::KeyDown(Key::LControl)]);

        let segments = parse_macro_sequence("{KeyDown RShift}");
        assert_eq!(segments, vec![MacroSegment::KeyDown(Key::RShift)]);
    }

    // Paste tests
    #[test]
    fn test_parse_paste() {
        let segments = parse_macro_sequence("{Paste}");
        assert_eq!(segments, vec![MacroSegment::Paste]);
    }

    #[test]
    fn test_parse_paste_case_insensitive() {
        let segments = parse_macro_sequence("{paste}");
        assert_eq!(segments, vec![MacroSegment::Paste]);
    }

    // Mixed tests
    #[test]
    fn test_parse_mixed_commands() {
        let segments = parse_macro_sequence("Hello{Delay 100}{Enter}World");
        assert_eq!(
            segments,
            vec![
                MacroSegment::Text("Hello".to_string()),
                MacroSegment::Delay(100),
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::Text("World".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_escaped_braces_with_text() {
        // Escaped braces are merged into surrounding text
        let segments = parse_macro_sequence("Type {{braces}}");
        assert_eq!(
            segments,
            vec![MacroSegment::Text("Type {braces}".to_string())]
        );
    }

    #[test]
    fn test_parse_ctrl_c_combo() {
        // Common use case: Ctrl+C
        let segments = parse_macro_sequence("{KeyDown Ctrl}c{KeyUp Ctrl}");
        assert_eq!(
            segments,
            vec![
                MacroSegment::KeyDown(Key::Control),
                MacroSegment::Text("c".to_string()),
                MacroSegment::KeyUp(Key::Control),
            ]
        );
    }

    // === DSL Execution Tests (08-02) ===

    #[test]
    fn test_parse_delay_segment() {
        let segments = parse_macro_sequence("{Delay 500}");
        assert_eq!(segments, vec![MacroSegment::Delay(500)]);
    }

    #[test]
    fn test_parse_keydown_keyup() {
        let segments = parse_macro_sequence("{KeyDown Ctrl}c{KeyUp Ctrl}");
        assert_eq!(
            segments,
            vec![
                MacroSegment::KeyDown(Key::Control),
                MacroSegment::Text("c".to_string()),
                MacroSegment::KeyUp(Key::Control),
            ]
        );
    }

    #[test]
    fn test_parse_paste_in_context() {
        let segments = parse_macro_sequence("prefix{Paste}suffix");
        assert_eq!(
            segments,
            vec![
                MacroSegment::Text("prefix".to_string()),
                MacroSegment::Paste,
                MacroSegment::Text("suffix".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_brace_escapes_for_json() {
        // Template: Type JSON with escaped braces
        let segments = parse_macro_sequence(r#"{{"name": "test"}}"#);
        assert_eq!(
            segments,
            vec![MacroSegment::Text(r#"{"name": "test"}"#.to_string())]
        );
    }

    #[test]
    fn test_complex_macro_with_all_features() {
        // Realistic macro: Type text, delay, press Enter, escaped braces, paste clipboard
        let segments = parse_macro_sequence("Hello{Delay 100}{Enter}{{data}}: {Paste}");
        assert_eq!(
            segments,
            vec![
                MacroSegment::Text("Hello".to_string()),
                MacroSegment::Delay(100),
                MacroSegment::SpecialKey(Key::Return),
                MacroSegment::Text("{data}: ".to_string()),
                MacroSegment::Paste,
            ]
        );
    }

    #[test]
    fn test_shift_combo_for_uppercase() {
        // {KeyDown Shift}hello{KeyUp Shift} should hold shift while typing
        let segments = parse_macro_sequence("{KeyDown Shift}hello{KeyUp Shift}");
        assert_eq!(
            segments,
            vec![
                MacroSegment::KeyDown(Key::Shift),
                MacroSegment::Text("hello".to_string()),
                MacroSegment::KeyUp(Key::Shift),
            ]
        );
    }
}
