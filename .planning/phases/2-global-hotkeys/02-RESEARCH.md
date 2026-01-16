# Phase 2: Global Hotkeys - Research

**Researched:** 2026-01-16
**Domain:** Global keyboard shortcuts for desktop applications
**Confidence:** HIGH

## Summary

Global hotkey registration in Rust is well-served by the `global-hotkey` crate from the Tauri project. This crate provides cross-platform support (Windows, macOS, Linux X11) with a clean API for registering, unregistering, and listening to global keyboard shortcuts.

The crate integrates naturally with the existing KeyBlast architecture: it requires the same event loop pattern already established with winit in Phase 1. The `GlobalHotKeyManager` must be created on the same thread as the event loop (main thread on macOS), which aligns with the existing `KeyBlastApp` structure.

Conflict detection is built into the API - the `register()` method returns `Error::AlreadyRegistered(HotKey)` when attempting to register a hotkey already in use by the application, and `Error::FailedToRegister(String)` when a hotkey is claimed by the OS or another application.

**Primary recommendation:** Use `global-hotkey` 0.7.0 with the existing winit event loop. Create `GlobalHotKeyManager` in `resumed()` alongside tray icon creation, and process hotkey events in `about_to_wait()` via `GlobalHotKeyEvent::receiver().try_recv()`.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| global-hotkey | 0.7.0 | Global keyboard shortcut registration | Tauri-maintained, 111K+ downloads/month, same author as tray-icon |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| keyboard-types | (dep of global-hotkey) | Key code definitions | Automatically included, provides `Code` enum |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| global-hotkey | livesplit-hotkey | Supports Wayland/WASM but less integrated with Tauri ecosystem |
| global-hotkey | win-hotkeys | Windows-only, requires separate event loop thread |
| global-hotkey | InputBot | No macOS support |

**Installation:**
```bash
cargo add global-hotkey@0.7
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── app.rs           # AppState - add hotkey registry
├── hotkey.rs        # NEW: HotkeyManager, HotkeyRegistry
├── tray.rs          # TrayIcon setup (existing)
└── main.rs          # Event loop integration
```

### Pattern 1: Event Loop Integration with winit
**What:** Use `GlobalHotKeyEvent::set_event_handler` to forward events to the winit event loop via `EventLoopProxy`
**When to use:** Always - required for proper cross-platform behavior
**Example:**
```rust
// Source: https://github.com/tauri-apps/global-hotkey/blob/dev/examples/winit.rs
use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent, hotkey::{HotKey, Modifiers, Code}};
use winit::event_loop::{EventLoop, EventLoopProxy};

#[derive(Debug)]
enum AppEvent {
    HotKey(GlobalHotKeyEvent),
    Menu(muda::MenuEvent),
}

fn main() {
    let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    // Forward global hotkey events to the winit event loop
    GlobalHotKeyEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(AppEvent::HotKey(event));
    }));

    // ...
}
```

### Pattern 2: Hotkey Registry with ID Tracking
**What:** Maintain a HashMap of registered hotkeys by their ID for lookup during event handling
**When to use:** When you need to map hotkey events back to their associated actions/macros
**Example:**
```rust
use std::collections::HashMap;
use global_hotkey::hotkey::HotKey;

struct HotkeyRegistry {
    manager: GlobalHotKeyManager,
    hotkeys: HashMap<u32, HotkeyBinding>,  // id -> binding
}

struct HotkeyBinding {
    hotkey: HotKey,
    macro_id: String,
}

impl HotkeyRegistry {
    fn register(&mut self, hotkey: HotKey, macro_id: String) -> Result<(), global_hotkey::Error> {
        self.manager.register(hotkey)?;
        self.hotkeys.insert(hotkey.id(), HotkeyBinding { hotkey, macro_id });
        Ok(())
    }

    fn get_macro_id(&self, hotkey_id: u32) -> Option<&str> {
        self.hotkeys.get(&hotkey_id).map(|b| b.macro_id.as_str())
    }
}
```

### Pattern 3: Error-Based Conflict Detection
**What:** Use the `Error::AlreadyRegistered` and `Error::FailedToRegister` variants to detect conflicts
**When to use:** Requirement HKEY-02 - warn user when assigning conflicting hotkey
**Example:**
```rust
use global_hotkey::{GlobalHotKeyManager, Error, hotkey::HotKey};

fn try_register(manager: &GlobalHotKeyManager, hotkey: HotKey) -> RegisterResult {
    match manager.register(hotkey) {
        Ok(()) => RegisterResult::Success,
        Err(Error::AlreadyRegistered(hk)) => {
            RegisterResult::ConflictInternal(hk) // Already registered by this app
        }
        Err(Error::FailedToRegister(msg)) => {
            RegisterResult::ConflictExternal(msg) // OS or other app has it
        }
        Err(e) => RegisterResult::Error(e),
    }
}

enum RegisterResult {
    Success,
    ConflictInternal(HotKey),  // Hotkey already registered by KeyBlast
    ConflictExternal(String),  // Hotkey taken by OS or another app
    Error(global_hotkey::Error),
}
```

### Pattern 4: Hotkey String Parsing
**What:** Parse hotkeys from string representation for configuration storage
**When to use:** Loading/saving hotkey configurations
**Example:**
```rust
use global_hotkey::hotkey::HotKey;

// Parse from string - modifiers must come before key
let hotkey: HotKey = "shift+alt+KeyQ".parse().unwrap();
let hotkey2: HotKey = "ctrl+KeyS".parse().unwrap();
let hotkey3: HotKey = "super+shift+F1".parse().unwrap();

// Convert back to string for storage
let stored = hotkey.into_string();
```

### Anti-Patterns to Avoid
- **Creating GlobalHotKeyManager on wrong thread:** On macOS, MUST be main thread. On Windows, must be same thread as event loop.
- **Polling receiver without event loop:** The `try_recv()` approach only works inside an active event loop. Don't poll in a busy loop.
- **Ignoring registration errors:** Always handle `register()` results - silent failures lead to confused users.
- **Using process::exit for cleanup:** Unregister hotkeys properly before exit to avoid OS-level orphaned registrations.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Global hotkey capture | Raw CGEventTap/SetWindowsHookEx | global-hotkey crate | Platform differences are subtle, security/permissions vary |
| Key code handling | Manual key mapping | keyboard-types::Code enum | Comprehensive W3C-compliant key codes |
| Modifier combination parsing | String parsing | HotKey::from_str() | Edge cases in modifier ordering, validation |
| Conflict detection | Check system shortcuts manually | register() error handling | OS reports conflicts, no need to guess |

**Key insight:** Global hotkey registration involves deep platform-specific APIs (Carbon on macOS, Win32 on Windows, X11 on Linux). The `global-hotkey` crate abstracts these correctly; hand-rolling would require maintaining three separate implementations.

## Common Pitfalls

### Pitfall 1: Thread Mismatch on macOS
**What goes wrong:** GlobalHotKeyManager created on background thread, hotkeys never fire
**Why it happens:** macOS requires main thread for event loop integration
**How to avoid:** Create manager in `resumed()` callback which runs on main thread
**Warning signs:** No errors but hotkey events never arrive

### Pitfall 2: Not Forwarding Events to winit
**What goes wrong:** Using `try_recv()` but winit event loop is in `Wait` mode, events delayed
**Why it happens:** `ControlFlow::Wait` doesn't wake up for external events
**How to avoid:** Use `GlobalHotKeyEvent::set_event_handler` with `EventLoopProxy` to wake the loop
**Warning signs:** Hotkeys work but with noticeable delay

### Pitfall 3: Assuming All Hotkeys Are Available
**What goes wrong:** User can't register expected shortcuts like Cmd+C, Cmd+V
**Why it happens:** System shortcuts are globally reserved by OS
**How to avoid:** Handle `FailedToRegister` gracefully, suggest alternatives
**Warning signs:** `Error::FailedToRegister` for common modifier+key combinations

### Pitfall 4: No Wayland Support
**What goes wrong:** App doesn't work on modern Linux desktops using Wayland
**Why it happens:** global-hotkey only supports X11 on Linux
**How to avoid:** Document limitation, or detect Wayland and show warning
**Warning signs:** Works in X11 session but not Wayland

### Pitfall 5: Orphaned Hotkey Registrations
**What goes wrong:** After crash, OS still has hotkeys registered to dead process
**Why it happens:** No cleanup on abnormal exit
**How to avoid:** OS typically cleans up on process death, but test this behavior
**Warning signs:** "Hotkey already registered" errors after crash

## Code Examples

Verified patterns from official sources:

### Complete winit Integration
```rust
// Source: https://github.com/tauri-apps/global-hotkey/blob/dev/examples/winit.rs
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

#[derive(Debug)]
enum AppEvent {
    HotKey(GlobalHotKeyEvent),
}

struct App {
    hotkeys_manager: GlobalHotKeyManager,
}

impl ApplicationHandler<AppEvent> for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        _event: WindowEvent,
    ) {}

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::HotKey(event) => {
                println!("Hotkey pressed: {:?}", event);
                if event.state == HotKeyState::Pressed {
                    // Handle hotkey press
                }
            }
        }
    }
}

fn main() {
    let hotkeys_manager = GlobalHotKeyManager::new().unwrap();

    let hotkey = HotKey::new(Some(Modifiers::SHIFT), Code::KeyD);
    hotkeys_manager.register(hotkey).unwrap();

    let event_loop = EventLoop::<AppEvent>::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    GlobalHotKeyEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(AppEvent::HotKey(event));
    }));

    let mut app = App { hotkeys_manager };
    event_loop.run_app(&mut app).unwrap();
}
```

### Registering Hotkey with Conflict Handling
```rust
// Source: Derived from API documentation
use global_hotkey::{GlobalHotKeyManager, Error, hotkey::{HotKey, Modifiers, Code}};

fn register_with_feedback(manager: &GlobalHotKeyManager, hotkey: HotKey) -> Result<(), String> {
    match manager.register(hotkey) {
        Ok(()) => Ok(()),
        Err(Error::AlreadyRegistered(hk)) => {
            Err(format!("Hotkey {} is already registered by KeyBlast", hk))
        }
        Err(Error::FailedToRegister(msg)) => {
            Err(format!("Hotkey unavailable (may be used by system or another app): {}", msg))
        }
        Err(e) => Err(format!("Registration error: {}", e)),
    }
}
```

### Parsing Hotkeys from Config
```rust
// Source: https://docs.rs/global-hotkey/latest/global_hotkey/hotkey/struct.HotKey.html
use global_hotkey::hotkey::HotKey;

fn parse_hotkey(s: &str) -> Result<HotKey, String> {
    s.parse::<HotKey>()
        .map_err(|e| format!("Invalid hotkey '{}': {}", s, e))
}

// Valid formats (modifiers before key):
// "shift+KeyD"
// "ctrl+alt+KeyQ"
// "super+shift+F1"
// "ctrl+shift+alt+super+KeyA" (all modifiers)
// "KeyF" (no modifiers)
```

## Hotkey Suggestions for HKEY-03

For suggesting available hotkey combinations, use these generally-safe patterns:

### Tier 1: Very Likely Available
```rust
// Ctrl+Shift+<letter> - rarely conflicts
HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD)

// Ctrl+Alt+<letter> - rarely conflicts
HotKey::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyM)

// F13-F19 keys (if keyboard has them)
HotKey::new(None, Code::F13)
```

### Tier 2: Usually Available
```rust
// Ctrl+Shift+<number>
HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Digit1)

// Super/Cmd+Shift+<letter> - some conflicts on macOS
HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::KeyK)
```

### Tier 3: Check Before Suggesting
```rust
// Ctrl+<letter> - often taken (Ctrl+C, Ctrl+V, etc.)
// Super/Cmd+<letter> - system shortcuts on macOS
// Alt+<letter> - menu accelerators on Windows
```

**Implementation strategy:** Maintain a list of candidate hotkeys in Tier 1/2, try registering in sequence, unregister immediately, suggest the first N that succeed.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| hotkey-rs crate | global-hotkey | 2022 | Tauri-backed, actively maintained |
| Manual CGEventTap | global-hotkey | N/A | Abstracts macOS Carbon API |
| Separate thread event loops | Single winit integration | v0.5+ | Cleaner architecture |

**Deprecated/outdated:**
- `tauri-hotkey` crate: Marked as "minimal maintenance", use `global-hotkey` directly
- Raw X11 bindings: `global-hotkey` v0.7.0 uses `x11rb` internally

## Known Limitations

1. **No Wayland support** (Linux): Only X11 supported. Issue #28 open.
2. **Cannot use Globe/Fn key on macOS**: Issue #111 - hardware limitation
3. **Cannot override system defaults on Windows**: Issue #161
4. **Security advisories on dependencies**: Multiple RUSTSEC notices for gtk-rs and other deps

## Open Questions

Things that couldn't be fully resolved:

1. **Wayland support timeline**
   - What we know: Issue #28 open since 2023, marked "help wanted"
   - What's unclear: Whether/when this will be addressed
   - Recommendation: Document as known limitation, most Linux users still have X11 fallback

2. **Hotkey persistence across app restarts**
   - What we know: OS cleans up registrations when process exits
   - What's unclear: Behavior on crash vs clean shutdown
   - Recommendation: Re-register all hotkeys on startup, don't assume persistence

3. **Accessibility permissions on macOS**
   - What we know: Some global hotkey apps require Accessibility permissions
   - What's unclear: Whether global-hotkey requires this (likely not for basic usage)
   - Recommendation: Test on clean macOS install, document if permissions needed

## Sources

### Primary (HIGH confidence)
- [global-hotkey crates.io](https://crates.io/crates/global-hotkey) - Version 0.7.0 confirmed
- [global-hotkey GitHub](https://github.com/tauri-apps/global-hotkey) - Examples, README, issues
- [docs.rs/global-hotkey](https://docs.rs/global-hotkey/latest/global_hotkey/) - API documentation

### Secondary (MEDIUM confidence)
- [tray-icon README](https://github.com/tauri-apps/tray-icon/blob/dev/README.md) - Event loop requirements
- [Tauri Global Shortcut Plugin](https://v2.tauri.app/plugin/global-shortcut/) - Pattern validation

### Tertiary (LOW confidence)
- WebSearch results on macOS shortcut conflicts - General guidance only
- Rust forum posts - Specific to different crates (win-hotkeys)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - global-hotkey is clearly dominant, Tauri-maintained
- Architecture: HIGH - Official examples show winit integration pattern
- Pitfalls: HIGH - Well-documented platform requirements, error types clear
- Conflict detection: MEDIUM - API supports it, real-world edge cases less documented

**Research date:** 2026-01-16
**Valid until:** 2026-02-16 (30 days - stable crate, v0.7.0 since May 2025)
