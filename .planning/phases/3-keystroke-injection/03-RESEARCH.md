# Phase 3: Keystroke Injection - Research

**Researched:** 2026-01-16
**Domain:** Cross-platform keystroke simulation and macOS Accessibility permissions
**Confidence:** HIGH

## Summary

Keystroke injection in Rust is well-served by the `enigo` crate (v0.6.1), the most mature cross-platform input simulation library in the Rust ecosystem. The library supports typing arbitrary text including Unicode, pressing special keys (Enter, Tab, Escape, arrows, function keys), and provides fine-grained control over key press/release timing.

The critical challenge for Phase 3 is **modifier key state handling**: when a hotkey like Ctrl+Shift+K triggers a macro, those modifier keys are still held down. If text is injected immediately, it will be affected by the held modifiers (e.g., text becomes CAPITALIZED with Shift held, or triggers shortcuts with Ctrl held). The solution is to release modifier keys before injecting, then optionally restore them.

macOS requires Accessibility permissions for keystroke injection. The `macos-accessibility-client` crate provides `application_is_trusted_with_prompt()` to check and prompt for permissions. Enigo handles this automatically by default (`open_prompt_to_get_permissions: true`), but for better UX, KeyBlast should check permissions proactively at startup.

**Primary recommendation:** Use `enigo` 0.6.1 with `Settings::default()`. Implement a `type_text()` function that: (1) releases held modifiers from the triggering hotkey, (2) waits a brief delay, (3) injects text character-by-character with configurable delay, (4) handles special keys in sequence. Check Accessibility permissions at startup on macOS with clear user guidance.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| enigo | 0.6.1 | Keyboard/mouse simulation | 1.6k GitHub stars, 417K+ downloads, actively maintained, cross-platform |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| macos-accessibility-client | 0.0.1 | Check/prompt for Accessibility permissions | macOS-only, proactive permission checking at startup |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| enigo | rdev | Can simulate input but less mature, last updated 2+ years ago |
| enigo | InputBot | No macOS support, Windows/Linux only |
| enigo | simulate | Windows-only |
| macos-accessibility-client | enigo's built-in prompt | Less control over UX, cannot guide user before attempting injection |

**Installation:**
```bash
cargo add enigo@0.6

# macOS-specific for permission checking
cargo add macos-accessibility-client --target 'cfg(target_os = "macos")'
```

**Cargo.toml:**
```toml
[dependencies]
enigo = "0.6"

[target.'cfg(target_os = "macos")'.dependencies]
macos-accessibility-client = "0.0.1"
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── app.rs           # AppState (existing)
├── hotkey.rs        # HotkeyManager (existing)
├── injection.rs     # NEW: KeystrokeInjector, MacroSequence
├── permission.rs    # NEW: macOS accessibility permission handling
├── tray.rs          # Tray icon (existing)
└── main.rs          # Event loop, wire hotkey -> injection
```

### Pattern 1: Modifier Release Before Injection
**What:** Release held modifier keys before typing text to prevent interference
**When to use:** Always - critical for correct macro expansion
**Example:**
```rust
use enigo::{Enigo, Keyboard, Key, Direction, Settings};

struct KeystrokeInjector {
    enigo: Enigo,
}

impl KeystrokeInjector {
    fn new() -> Result<Self, enigo::Error> {
        Ok(Self {
            enigo: Enigo::new(&Settings::default())?,
        })
    }

    /// Release common modifiers that might be held from hotkey activation
    fn release_modifiers(&mut self) -> Result<(), enigo::Error> {
        // Release all modifiers that could have triggered the hotkey
        self.enigo.key(Key::Control, Direction::Release)?;
        self.enigo.key(Key::Shift, Direction::Release)?;
        self.enigo.key(Key::Alt, Direction::Release)?;
        self.enigo.key(Key::Meta, Direction::Release)?;  // Cmd on macOS, Win on Windows
        Ok(())
    }

    /// Type text after releasing modifiers
    fn type_text(&mut self, text: &str) -> Result<(), enigo::Error> {
        self.release_modifiers()?;

        // Small delay to ensure modifiers are released
        std::thread::sleep(std::time::Duration::from_millis(10));

        self.enigo.text(text)?;
        Ok(())
    }
}
```

### Pattern 2: Character-by-Character with Configurable Delay
**What:** Type text one character at a time with optional delay for slow applications
**When to use:** Requirement INJT-03 - per-macro keystroke delay
**Example:**
```rust
use std::time::Duration;

impl KeystrokeInjector {
    /// Type text with configurable delay between keystrokes
    fn type_text_with_delay(&mut self, text: &str, delay_ms: u64) -> Result<(), enigo::Error> {
        self.release_modifiers()?;
        std::thread::sleep(Duration::from_millis(10));

        if delay_ms == 0 {
            // Fast path: use bulk text() method
            self.enigo.text(text)?;
        } else {
            // Slow path: character-by-character with delay
            for c in text.chars() {
                self.enigo.text(&c.to_string())?;
                std::thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        Ok(())
    }
}
```

### Pattern 3: Special Key Sequences
**What:** Handle macros with embedded special keys like Enter, Tab, arrows
**When to use:** Requirement INJT-02 - special keys in sequences
**Example:**
```rust
/// Represents a segment of a macro sequence
enum MacroSegment {
    Text(String),
    SpecialKey(Key),
}

/// Parse a macro string with escape sequences into segments
/// Example: "Hello{Enter}World{Tab}Next" -> [Text("Hello"), SpecialKey(Return), Text("World"), ...]
fn parse_macro_sequence(input: &str) -> Vec<MacroSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            // Flush accumulated text
            if !current_text.is_empty() {
                segments.push(MacroSegment::Text(current_text.clone()));
                current_text.clear();
            }
            // Parse special key
            let mut key_name = String::new();
            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next();
                    break;
                }
                key_name.push(chars.next().unwrap());
            }
            if let Some(key) = special_key_from_name(&key_name) {
                segments.push(MacroSegment::SpecialKey(key));
            }
        } else {
            current_text.push(c);
        }
    }
    if !current_text.is_empty() {
        segments.push(MacroSegment::Text(current_text));
    }
    segments
}

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

impl KeystrokeInjector {
    fn execute_sequence(&mut self, segments: &[MacroSegment], delay_ms: u64) -> Result<(), enigo::Error> {
        self.release_modifiers()?;
        std::thread::sleep(std::time::Duration::from_millis(10));

        for segment in segments {
            match segment {
                MacroSegment::Text(text) => {
                    if delay_ms == 0 {
                        self.enigo.text(text)?;
                    } else {
                        for c in text.chars() {
                            self.enigo.text(&c.to_string())?;
                            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                        }
                    }
                }
                MacroSegment::SpecialKey(key) => {
                    self.enigo.key(*key, Direction::Click)?;
                    if delay_ms > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                    }
                }
            }
        }
        Ok(())
    }
}
```

### Pattern 4: macOS Accessibility Permission Checking
**What:** Proactively check and prompt for Accessibility permission
**When to use:** At application startup on macOS
**Example:**
```rust
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

// At startup:
fn init_injection() -> Result<KeystrokeInjector, String> {
    if !check_accessibility_permission() {
        return Err(
            "Accessibility permission required. Please grant permission in \
             System Preferences > Privacy & Security > Accessibility, then restart KeyBlast."
            .to_string()
        );
    }
    KeystrokeInjector::new().map_err(|e| format!("Failed to initialize injector: {}", e))
}
```

### Anti-Patterns to Avoid
- **Injecting immediately after hotkey:** Modifiers are still held, corrupting output
- **Using Enigo from multiple threads:** Create one Enigo instance, use from single thread
- **Ignoring the Result:** All enigo methods return Result - handle errors gracefully
- **Assuming permissions:** Always check Accessibility permission on macOS before first use
- **Blocking main thread with delays:** Keep delay_ms reasonable (0-50ms typical)

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Cross-platform key simulation | CGEventPost/SendInput wrappers | enigo crate | Platform APIs differ wildly, Unicode handling is complex |
| macOS permission checking | Manual CoreFoundation calls | macos-accessibility-client | AX API quirks, prompt handling |
| Unicode text input | Char-by-char virtual key mapping | enigo.text() | Unicode handling varies by platform, enigo handles it |
| Special key codes | Manual mapping table | enigo::Key enum | 112 variants already defined, platform-specific codes handled |

**Key insight:** Keystroke injection looks simple ("just send keystrokes") but involves platform-specific APIs (CGEventPost on macOS, SendInput on Windows, X11/libei on Linux), Unicode handling differences, modifier key state, and permission models. Enigo abstracts all of this correctly.

## Common Pitfalls

### Pitfall 1: Modifier Keys Still Held After Hotkey
**What goes wrong:** User presses Ctrl+Shift+K, macro types "Hello" but it appears as "HELLO" or triggers shortcuts
**Why it happens:** Hotkey triggers while modifiers are physically held down, enigo sees the held state
**How to avoid:** Always release modifiers before injecting text (see Pattern 1)
**Warning signs:** Text appears capitalized, shortcuts trigger instead of text

### Pitfall 2: Missing macOS Accessibility Permission
**What goes wrong:** First keystroke injection silently fails or shows confusing system dialog
**Why it happens:** macOS requires explicit user permission for input simulation
**How to avoid:** Check permission at startup with `application_is_trusted_with_prompt()`, guide user clearly
**Warning signs:** Nothing happens when hotkey triggers, no error message

### Pitfall 3: Text Garbled in Slow Applications
**What goes wrong:** Some characters missing or out of order in target application
**Why it happens:** Application cannot process keystrokes as fast as enigo sends them
**How to avoid:** Implement configurable delay (INJT-03), default to 0, let users increase for problem apps
**Warning signs:** Terminal emulators, remote desktop, old apps show issues

### Pitfall 4: Special Keys in Text Not Working
**What goes wrong:** User types "{Enter}" in macro config, sees literal "{Enter}" in output
**Why it happens:** Text passed directly to enigo.text() without parsing
**How to avoid:** Parse escape sequences before injection (see Pattern 3)
**Warning signs:** Literal brace text appears instead of newline/tab

### Pitfall 5: Enigo Instance Created Multiple Times
**What goes wrong:** Memory leaks, performance degradation, potential resource conflicts
**Why it happens:** Creating new Enigo for each injection
**How to avoid:** Create single KeystrokeInjector at startup, reuse for all injections
**Warning signs:** Growing memory usage, slower injection over time

### Pitfall 6: Sandboxed macOS Apps Cannot Use Accessibility
**What goes wrong:** `AXIsProcessTrusted()` always returns false
**Why it happens:** Apple does not support Accessibility APIs in sandboxed apps
**How to avoid:** KeyBlast should NOT be sandboxed - distribute outside App Store or request exception entitlement
**Warning signs:** Permission prompt never appears, always returns false

## Code Examples

Verified patterns from official sources and documentation:

### Basic Text Input
```rust
// Source: https://github.com/enigo-rs/enigo README
use enigo::{Enigo, Keyboard, Settings};

let mut enigo = Enigo::new(&Settings::default()).unwrap();
enigo.text("Hello World!").unwrap();
```

### Special Key Press
```rust
// Source: https://docs.rs/enigo/0.6.1/enigo/
use enigo::{Enigo, Keyboard, Key, Direction, Settings};

let mut enigo = Enigo::new(&Settings::default()).unwrap();
enigo.key(Key::Return, Direction::Click).unwrap();
enigo.key(Key::Tab, Direction::Click).unwrap();
enigo.key(Key::Escape, Direction::Click).unwrap();
enigo.key(Key::UpArrow, Direction::Click).unwrap();
```

### Modifier Key Handling
```rust
// Source: https://docs.rs/enigo - Direction enum documentation
use enigo::{Enigo, Keyboard, Key, Direction, Settings};

let mut enigo = Enigo::new(&Settings::default()).unwrap();

// Press Ctrl+V (paste)
enigo.key(Key::Control, Direction::Press).unwrap();
enigo.key(Key::Unicode('v'), Direction::Click).unwrap();
enigo.key(Key::Control, Direction::Release).unwrap();
```

### Settings Configuration
```rust
// Source: https://docs.rs/enigo/0.6.1/enigo/struct.Settings.html
use enigo::{Enigo, Settings};

let settings = Settings {
    // macOS: Don't prompt automatically (we'll handle it ourselves)
    #[cfg(target_os = "macos")]
    open_prompt_to_get_permissions: false,
    // macOS: Simulated input independent of physical keyboard state
    #[cfg(target_os = "macos")]
    independent_of_keyboard_state: true,
    // Release held keys when Enigo is dropped (safety net)
    release_keys_when_dropped: true,
    ..Settings::default()
};

let enigo = Enigo::new(&settings).unwrap();
```

### macOS Permission Check
```rust
// Source: https://docs.rs/macos-accessibility-client
#[cfg(target_os = "macos")]
fn query_accessibility_permissions() -> bool {
    use macos_accessibility_client::accessibility::application_is_trusted_with_prompt;

    let trusted = application_is_trusted_with_prompt();
    if trusted {
        println!("Application is trusted for accessibility");
    } else {
        println!("Accessibility permission required - please grant in System Preferences");
    }
    trusted
}
```

## Key Enum Reference

Enigo's `Key` enum supports 112 variants. Key ones for INJT-02:

| Category | Keys |
|----------|------|
| Navigation | UpArrow, DownArrow, LeftArrow, RightArrow, Home, End, PageUp, PageDown |
| Text Entry | Return, Tab, Space, Backspace, Delete |
| Escape | Escape |
| Function | F1 through F35 |
| Modifiers | Shift, Control, Alt, Meta/Command |
| Numpad | Numpad0-9, Add, Subtract, Multiply, Divide, Decimal |
| Media | MediaPlayPause, MediaNextTrack, MediaPrevTrack, VolumeUp, VolumeDown |
| Unicode | Unicode(char) - any Unicode character |

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| enigo DSL macros | Agent trait (serde feature) | v0.2.0 | Cleaner API, serializable |
| Physical keyboard state affects output (macOS) | independent_of_keyboard_state | v0.3.0 | Simulated input no longer affected by held keys |
| libxdotool dependency (Linux) | x11rb native Rust | v0.5.0 | No runtime dependency, more reliable |
| xdotool | libei (experimental) | v0.5.0+ | Wayland support coming |

**Deprecated/outdated:**
- enigo DSL: Removed in v0.2.0, replaced with Agent trait + serde
- `delay()` function: Removed in development branch - use `std::thread::sleep` instead
- `linux_delay` setting: Being removed - delays now platform-specific

## Open Questions

Things that couldn't be fully resolved:

1. **Optimal default delay value**
   - What we know: 0ms works for most apps, some need 10-50ms
   - What's unclear: Best default that balances speed vs compatibility
   - Recommendation: Default to 0, expose in per-macro config, document troubleshooting

2. **Modifier key restoration**
   - What we know: We release modifiers before injection
   - What's unclear: Should we re-press them after? User may have released
   - Recommendation: Don't restore - releasing is safer, user controls physical state

3. **Thread safety of Enigo**
   - What we know: Enigo uses platform APIs that may not be thread-safe
   - What's unclear: Exact thread requirements per platform
   - Recommendation: Keep KeystrokeInjector on main thread with event loop

4. **Wayland support timeline**
   - What we know: libei backend is experimental, has bugs
   - What's unclear: When it will be production-ready
   - Recommendation: Linux support via X11, document Wayland as known limitation

## Sources

### Primary (HIGH confidence)
- [enigo docs.rs](https://docs.rs/enigo/0.6.1/enigo/) - Official API documentation
- [enigo GitHub](https://github.com/enigo-rs/enigo) - v0.6.1, 1.6k stars
- [enigo CHANGES.md](https://github.com/enigo-rs/enigo/blob/main/CHANGES.md) - Recent changes, breaking changes
- [enigo Permissions.md](https://github.com/enigo-rs/enigo/blob/main/Permissions.md) - Platform permission requirements
- [macos-accessibility-client docs.rs](https://docs.rs/macos-accessibility-client/latest/macos_accessibility_client/) - Permission checking API

### Secondary (MEDIUM confidence)
- [Espanso configuration options](https://espanso.org/docs/configuration/options/) - Delay patterns from production text expander
- [Apple AXIsProcessTrusted documentation](https://developer.apple.com/documentation/applicationservices/1460720-axisprocesstrusted) - Official macOS API

### Tertiary (LOW confidence)
- WebSearch results on modifier key handling patterns - General guidance
- AutoHotkey/Espanso forum discussions - Cross-validated with official docs

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - enigo is clearly dominant, actively maintained
- Architecture patterns: HIGH - Well-documented APIs, clear examples
- Modifier handling: MEDIUM - Pattern derived from text expander behavior, not official enigo docs
- Pitfalls: HIGH - Well-documented in issues, changelog, permissions doc

**Research date:** 2026-01-16
**Valid until:** 2026-02-16 (30 days - stable crate, v0.6.1 from ~3 months ago)
