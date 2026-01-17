# Phase 8: Expanded DSL - Research

**Researched:** 2026-01-17
**Domain:** Macro DSL parsing, clipboard access, modifier key state management
**Confidence:** HIGH

## Summary

Phase 8 extends KeyBlast's macro DSL with four new capabilities: inline delays (`{Delay 500}`), modifier key press/release (`{KeyDown Ctrl}`/`{KeyUp Ctrl}`), clipboard paste (`{Paste}`), and literal brace escapes (`{{`/`}}`). The research confirms that the existing stack (enigo 0.6) already supports everything needed for modifier keys and delays, while clipboard access requires adding a single new dependency (arboard 3.6).

The parser extension is straightforward: the existing `parse_macro_sequence` function already handles `{KeyName}` patterns. The extension adds parsing for parameterized commands (space-separated name and argument) and the `{{`/`}}` escape sequences. The `MacroSegment` enum expands from two variants (Text, SpecialKey) to five variants (adding Delay, KeyDown, KeyUp, Paste).

For `{Paste}`, the cross-platform approach is: read clipboard text with arboard, then type it using enigo's `text()` method. This is more reliable than simulating Ctrl+V/Cmd+V because it avoids modifier key state complications and works identically across platforms.

**Primary recommendation:** Extend `MacroSegment` enum with new variants, update parser to handle parameterized commands and brace escapes, add arboard for clipboard. No architectural changes needed.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| enigo | 0.6 | Keystroke injection, key press/release | Already in use, supports Direction::{Press, Release} |
| arboard | 3.6 | Clipboard read access | Maintained by 1Password, 16M+ downloads, cross-platform |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| std::thread::sleep | (std) | Delay implementation | For `{Delay N}` command |
| std::time::Duration | (std) | Duration handling | Millisecond delays |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| arboard | clipboard-rs | arboard more actively maintained, 1Password backing |
| arboard | copypasta (Alacritty) | arboard has simpler API, better Windows support |
| Type clipboard text | Simulate Ctrl+V/Cmd+V | Typing avoids modifier state issues, more reliable |

**Installation:**
```bash
cargo add arboard
```

**Cargo.toml addition:**
```toml
arboard = "3.6"
```

## Architecture Patterns

### Recommended Extension to MacroSegment

```rust
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
```

### Pattern 1: Parameterized Command Parsing
**What:** Parse `{Command Arg}` syntax (space-separated)
**When to use:** For `{Delay 500}`, `{KeyDown Ctrl}`, `{KeyUp Shift}`

```rust
// Inside parse_macro_sequence, after finding closing brace:
if found_close {
    // Check for parameterized commands (space-separated)
    let parts: Vec<&str> = key_name.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim());

    match command.as_str() {
        "delay" => {
            if let Some(ms_str) = arg {
                if let Ok(ms) = ms_str.parse::<u64>() {
                    flush_text(&mut current_text, &mut segments);
                    segments.push(MacroSegment::Delay(ms));
                    continue;
                }
            }
            // Invalid delay - treat as literal
            current_text.push('{');
            current_text.push_str(&key_name);
            current_text.push('}');
        }
        "keydown" => {
            if let Some(key) = arg.and_then(modifier_key_from_name) {
                flush_text(&mut current_text, &mut segments);
                segments.push(MacroSegment::KeyDown(key));
                continue;
            }
            // Invalid key - treat as literal
            // ...
        }
        "keyup" => {
            // Similar to keydown
        }
        "paste" => {
            flush_text(&mut current_text, &mut segments);
            segments.push(MacroSegment::Paste);
        }
        _ => {
            // Check for existing special keys (Enter, Tab, etc.)
            if let Some(key) = special_key_from_name(&key_name) {
                // ... existing logic
            }
        }
    }
}
```

### Pattern 2: Brace Escape Handling
**What:** Parse `{{` as literal `{` and `}}` as literal `}`
**When to use:** Always check for doubled braces before command parsing

```rust
// In parse_macro_sequence, at start of main loop:
while let Some(c) = chars.next() {
    if c == '{' {
        // Check for escaped brace `{{`
        if chars.peek() == Some(&'{') {
            chars.next(); // consume second '{'
            current_text.push('{');
            continue;
        }
        // Otherwise, start of command...
    } else if c == '}' {
        // Check for escaped brace `}}`
        if chars.peek() == Some(&'}') {
            chars.next(); // consume second '}'
            current_text.push('}');
            continue;
        }
        // Lone '}' - treat as literal (shouldn't happen in valid input)
        current_text.push(c);
    } else {
        current_text.push(c);
    }
}
```

### Pattern 3: Clipboard Paste via Text Typing
**What:** Read clipboard and type contents (not simulate Ctrl+V)
**When to use:** For `{Paste}` command execution

```rust
// In execute_single_segment:
MacroSegment::Paste => {
    // Create clipboard instance (short-lived)
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| InjectionError(format!("Clipboard error: {}", e)))?;

    // Get text from clipboard
    let text = clipboard.get_text()
        .map_err(|e| InjectionError(format!("Failed to read clipboard: {}", e)))?;

    // Type the clipboard contents
    self.enigo.text(&text)?;
}
```

**Why type instead of Ctrl+V:**
1. Avoids modifier key state complications (what if Ctrl is already held?)
2. Works identically on all platforms (no Ctrl vs Cmd difference)
3. More predictable timing behavior
4. Respects the user's keystroke delay setting

### Pattern 4: Modifier Key Press/Release
**What:** Use enigo's `Direction::{Press, Release}` for held modifiers
**When to use:** For `{KeyDown Ctrl}` and `{KeyUp Ctrl}` commands

```rust
// In execute_single_segment:
MacroSegment::KeyDown(key) => {
    self.enigo.key(*key, Direction::Press)?;
}
MacroSegment::KeyUp(key) => {
    self.enigo.key(*key, Direction::Release)?;
}
```

**Supported modifier keys:**
| DSL Name | enigo Key |
|----------|-----------|
| Ctrl, Control | Key::Control |
| Shift | Key::Shift |
| Alt | Key::Alt |
| Meta, Win, Cmd, Command, Super | Key::Meta |
| LCtrl, LControl | Key::LControl |
| RCtrl, RControl | Key::RControl |
| LShift | Key::LShift |
| RShift | Key::RShift |

### Pattern 5: Delay Execution in Async Context
**What:** Delays work differently in sync vs async execution
**When to use:** Phase 7's async execution already handles this

```rust
// In execution worker (already exists):
// The worker thread handles delays between segments.
// A new Delay segment adds an ADDITIONAL pause.

// In sync execution (fast path):
MacroSegment::Delay(ms) => {
    std::thread::sleep(Duration::from_millis(*ms));
}

// In async execution, the worker just sends Delay to main thread,
// which sleeps before processing next command.
```

### Anti-Patterns to Avoid
- **Simulating Ctrl+V for paste:** Platform-dependent, modifier state issues
- **Allowing KeyDown without KeyUp:** Risk of stuck keys; consider tracking state
- **Parsing numbers with unwrap:** Always handle parse errors gracefully
- **Creating Clipboard per character:** Create once per Paste segment, not per character
- **Ignoring clipboard errors:** Surface errors clearly; clipboard can be locked

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Clipboard access | Manual platform FFI | arboard | Platform differences, error handling, image support |
| Key press/release | SendInput/CGEvent calls | enigo Direction::{Press,Release} | Already using enigo, tested cross-platform |
| Duration parsing | Manual string parsing | `str::parse::<u64>()` | Standard library, handles edge cases |
| Timeout sleeping | Busy loop | std::thread::sleep | OS-level, efficient |

**Key insight:** The existing enigo dependency already has everything needed for modifier key state. The only new dependency is arboard for clipboard, which is trivial to integrate.

## Common Pitfalls

### Pitfall 1: Stuck Modifier Keys
**What goes wrong:** User macro has `{KeyDown Ctrl}` but no matching `{KeyUp Ctrl}`, leaving Ctrl stuck
**Why it happens:** Macro text error or copy-paste mistake
**How to avoid:**
- Document the requirement for matched KeyDown/KeyUp
- Consider adding a "release all modifiers" at macro completion
- The existing `release_modifiers()` call at execution start provides partial safety
**Warning signs:** After macro runs, regular typing produces shortcuts (Ctrl+A, etc.)

### Pitfall 2: Clipboard Unavailable
**What goes wrong:** `{Paste}` fails because clipboard is empty or locked
**Why it happens:** Nothing copied, or another app has clipboard open (Windows)
**How to avoid:**
- Handle arboard errors gracefully (skip paste, continue macro)
- Log warning but don't abort entire macro
- Consider retry with short delay on Windows
**Warning signs:** Macro stops mid-execution on `{Paste}`

### Pitfall 3: Delay Blocking UI (Sync Path)
**What goes wrong:** `{Delay 5000}` blocks event loop for 5 seconds
**Why it happens:** Sync fast path doesn't yield to event loop
**How to avoid:**
- Macros with Delay segments should ALWAYS use async path
- Update fast-path condition: `delay_ms == 0 && segments.len() <= 10 && !has_delay_segment`
**Warning signs:** Tray menu unresponsive during macro with inline delays

### Pitfall 4: Escaped Braces in Existing Macros
**What goes wrong:** Existing macro `{Unknown}` was literal; now `{{` changes meaning
**Why it happens:** Adding escape sequences changes parser behavior
**How to avoid:**
- `{{` only produces `{` if FOLLOWED by another `{`
- Single `{Unknown}` still treated as literal (existing behavior)
- Document escape sequence behavior clearly
**Warning signs:** Existing macros change output after upgrade

### Pitfall 5: Case Sensitivity in Commands
**What goes wrong:** `{delay 500}` works but `{Delay 500}` doesn't (or vice versa)
**Why it happens:** Inconsistent case handling in parser
**How to avoid:** Normalize command names to lowercase before matching
**Warning signs:** User reports inconsistent behavior

### Pitfall 6: Whitespace in Arguments
**What goes wrong:** `{Delay  500}` (double space) fails to parse
**Why it happens:** splitn(2, ' ') leaves leading space on argument
**How to avoid:** Trim the argument: `parts.get(1).map(|s| s.trim())`
**Warning signs:** Delays work with some spacing but not others

## Code Examples

Verified patterns from official sources:

### Complete MacroSegment Enum
```rust
// Source: Extension of existing injection.rs
use enigo::Key;

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
```

### Modifier Key Name Mapping
```rust
// Source: enigo docs.rs - Key enum variants
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
```

### Clipboard Read with arboard
```rust
// Source: arboard docs.rs - Clipboard struct
use arboard::Clipboard;

fn read_clipboard_text() -> Result<String, String> {
    let mut clipboard = Clipboard::new()
        .map_err(|e| format!("Failed to access clipboard: {}", e))?;

    clipboard.get_text()
        .map_err(|e| format!("Failed to read clipboard text: {}", e))
}
```

### Enigo Key Press/Release
```rust
// Source: enigo GitHub examples/keyboard.rs
use enigo::{Direction::{Press, Release}, Enigo, Key, Keyboard};

fn press_modifier(enigo: &mut Enigo, key: Key) -> Result<(), enigo::InputError> {
    enigo.key(key, Press)
}

fn release_modifier(enigo: &mut Enigo, key: Key) -> Result<(), enigo::InputError> {
    enigo.key(key, Release)
}
```

### Updated Parser (Brace Escapes + Parameterized Commands)
```rust
pub fn parse_macro_sequence(input: &str) -> Vec<MacroSegment> {
    let mut segments = Vec::new();
    let mut current_text = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            // Check for escaped brace `{{`
            if chars.peek() == Some(&'{') {
                chars.next();
                current_text.push('{');
                continue;
            }

            // Start of command
            let mut key_name = String::new();
            let mut found_close = false;

            while let Some(&next) = chars.peek() {
                if next == '}' {
                    chars.next();
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
                chars.next();
                current_text.push('}');
                continue;
            }
            // Lone '}' - treat as literal
            current_text.push(c);
        } else {
            current_text.push(c);
        }
    }

    flush_text(&mut current_text, &mut segments);
    segments
}

fn flush_text(current_text: &mut String, segments: &mut Vec<MacroSegment>) {
    if !current_text.is_empty() {
        segments.push(MacroSegment::Text(current_text.clone()));
        current_text.clear();
    }
}

fn parse_command(key_name: &str) -> Option<MacroSegment> {
    let parts: Vec<&str> = key_name.splitn(2, ' ').collect();
    let command = parts[0].to_lowercase();
    let arg = parts.get(1).map(|s| s.trim());

    match command.as_str() {
        "delay" => {
            arg.and_then(|s| s.parse::<u64>().ok())
                .map(MacroSegment::Delay)
        }
        "keydown" => {
            arg.and_then(modifier_key_from_name)
                .map(MacroSegment::KeyDown)
        }
        "keyup" => {
            arg.and_then(modifier_key_from_name)
                .map(MacroSegment::KeyUp)
        }
        "paste" => Some(MacroSegment::Paste),
        _ => special_key_from_name(key_name).map(MacroSegment::SpecialKey),
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| clipboard crate | arboard | 2021+ | arboard actively maintained, clipboard deprecated |
| enigo KeyboardControllable trait | enigo Keyboard trait | enigo 0.3+ | Cleaner API, better cross-platform |
| Manual platform clipboard FFI | arboard abstraction | Ongoing | Simplified code, handles edge cases |

**Deprecated/outdated:**
- `clipboard` crate: No longer maintained, use arboard instead
- `Key::Command`, `Key::Super`, `Key::Windows`: Deprecated in enigo, use `Key::Meta`

## Open Questions

Things that couldn't be fully resolved:

1. **Should KeyDown require KeyUp?**
   - What we know: Unmatched KeyDown leaves modifier stuck
   - What's unclear: Whether to track state and auto-release, or trust user
   - Recommendation: Document requirement; add release_modifiers() at macro end as safety net

2. **Clipboard image support**
   - What we know: arboard supports images, could add `{PasteImage}` in future
   - What's unclear: Whether image paste is useful for this tool
   - Recommendation: Text-only for v2.0; consider images for v3 if requested

3. **Delay precision**
   - What we know: std::thread::sleep has OS-dependent precision (~15ms on Windows)
   - What's unclear: Whether sub-15ms delays matter for this use case
   - Recommendation: Accept OS limitations; document that delays are approximate

## Sources

### Primary (HIGH confidence)
- [enigo docs.rs - Keyboard trait](https://docs.rs/enigo/latest/enigo/trait.Keyboard.html) - Key press/release API
- [enigo docs.rs - Key enum](https://docs.rs/enigo/latest/enigo/enum.Key.html) - Modifier key variants
- [enigo docs.rs - Direction enum](https://docs.rs/enigo/latest/enigo/enum.Direction.html) - Press/Release/Click
- [arboard docs.rs - Clipboard struct](https://docs.rs/arboard/latest/arboard/struct.Clipboard.html) - get_text(), set_text() API
- [enigo GitHub examples/keyboard.rs](https://github.com/enigo-rs/enigo/blob/main/examples/keyboard.rs) - Modifier key usage pattern

### Secondary (MEDIUM confidence)
- [arboard GitHub README](https://github.com/1Password/arboard) - Platform-specific notes (Windows global clipboard, Linux selection ownership)
- [enigo GitHub README](https://github.com/enigo-rs/enigo) - Cross-platform behavior notes

### Tertiary (LOW confidence)
- AutoHotkey syntax references - Informed DSL syntax design (not Rust-specific)

## Metadata

**Confidence breakdown:**
- Parser extension: HIGH - Extending existing working parser with clear patterns
- Modifier keys: HIGH - enigo Direction::{Press,Release} is documented and tested
- Clipboard: HIGH - arboard is mature (16M+ downloads, 1Password maintained)
- Brace escapes: HIGH - Standard pattern (`{{` -> `{`), simple to implement

**Research date:** 2026-01-17
**Valid until:** 90 days (stable crates, well-documented APIs)
