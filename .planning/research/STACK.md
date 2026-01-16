# Stack Research: Cross-Platform Rust System Tray App

**Project:** KeyBlast
**Domain:** Cross-platform keystroke automation (macOS + Windows)
**Researched:** 2026-01-16
**Overall Confidence:** HIGH

---

## Executive Summary

The Rust ecosystem has mature, well-maintained crates for all KeyBlast requirements. The Tauri team maintains the de facto standard crates (`tray-icon`, `global-hotkey`, `muda`) that work standalone without requiring the full Tauri framework. For keystroke injection, `enigo` is the clear leader with active development and cross-platform support.

**Key insight:** These crates share compatible event loop requirements - they all work with a standard event loop on the main thread, making integration straightforward.

---

## Recommended Stack

### Core Runtime

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **No async runtime needed** | N/A | Event-driven, not async | KeyBlast is event-loop-driven (hotkey press -> inject keystrokes). No network I/O, no async file operations. A simple synchronous event loop suffices. |

**Why no tokio/async-std:** This app responds to discrete events (hotkey pressed) with synchronous actions (inject keystrokes, update menu). Async adds complexity without benefit. The `tray-icon` and `global-hotkey` crates use their own event receivers that work with standard event loops.

---

### System Tray

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **tray-icon** | 0.21.3 | System tray icon and context menus | Maintained by Tauri team, 18.9k dependents, cross-platform (Win/Mac/Linux), works standalone without Tauri |
| **muda** | (re-exported) | Menu construction | Re-exported by tray-icon, provides native menu APIs |

**Why tray-icon:**
1. **Active maintenance:** v0.21.3 released January 2026, 57 releases total
2. **Production-proven:** Used by Tauri, which powers thousands of apps
3. **Standalone usage:** Does not require Tauri framework - can be used directly
4. **Native menus:** Uses muda internally for OS-native context menus
5. **Simple API:** `TrayIconBuilder::new().with_menu().with_tooltip().build()`

**Platform requirements:**
- **macOS:** Event loop must run on main thread, tray icon must be created on main thread
- **Windows:** Event loop must run on creation thread (can be any thread)

**Example usage:**
```rust
use tray_icon::{TrayIconBuilder, menu::Menu};
use tray_icon::menu::{MenuEvent, MenuItem};

let menu = Menu::new();
let quit_item = MenuItem::new("Quit", true, None);
menu.append(&quit_item)?;

let tray_icon = TrayIconBuilder::new()
    .with_menu(Box::new(menu))
    .with_tooltip("KeyBlast")
    .with_icon(icon)
    .build()?;
```

---

### Global Hotkeys

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **global-hotkey** | 0.7.0 | Register system-wide hotkey listeners | Maintained by Tauri team, cross-platform, simple API, same event loop pattern as tray-icon |

**Why global-hotkey:**
1. **Same maintainers as tray-icon:** Consistent APIs, compatible event loop patterns
2. **Simple registration:** Create manager, register hotkey, receive events
3. **Modifier support:** CTRL, ALT, SHIFT, META combinations
4. **Cross-platform:** Windows, macOS, Linux (X11)

**Platform requirements:**
- **macOS:** Event loop on main thread, manager created on main thread
- **Windows:** Win32 event loop on creation thread

**Example usage:**
```rust
use global_hotkey::{GlobalHotKeyManager, hotkey::{HotKey, Modifiers, Code}};
use global_hotkey::GlobalHotKeyEvent;

let manager = GlobalHotKeyManager::new()?;
let hotkey = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyK);
manager.register(hotkey)?;

// In event loop
if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
    if event.id == hotkey.id() {
        // Hotkey was pressed
    }
}
```

**Note on permissions:** On macOS, global hotkey registration may require Accessibility permissions depending on the application context. The app should handle permission prompts gracefully.

---

### Keystroke Injection

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **enigo** | 0.6.1 | Simulate keyboard input | Most mature Rust input simulation library, active development, cross-platform, handles permissions |

**Why enigo:**
1. **Mature:** 1.6k GitHub stars, 790 commits, actively maintained
2. **Cross-platform:** Windows, macOS, Linux (X11), BSD
3. **Permission-aware:** Checks macOS Accessibility permissions, prompts user if needed
4. **Full key support:** Plain text, special keys (Enter, Tab, Escape, arrows), Unicode
5. **Timing control:** Can configure delays between keystrokes

**Platform requirements:**
- **macOS:** Requires Accessibility permission (System Preferences > Privacy > Accessibility). Enigo checks and prompts automatically.
- **Windows:** UIPI limitations - cannot inject into higher-privilege windows without admin rights (e.g., Task Manager). Not a concern for typical use.
- **Linux:** No special permissions required

**Example usage:**
```rust
use enigo::{Enigo, Keyboard, Key, Settings};

let mut enigo = Enigo::new(&Settings::default())?;

// Type plain text
enigo.text("Hello, world!")?;

// Press special keys
enigo.key(Key::Return, enigo::Direction::Click)?;
enigo.key(Key::Tab, enigo::Direction::Click)?;
enigo.key(Key::Escape, enigo::Direction::Click)?;

// Arrow keys
enigo.key(Key::DownArrow, enigo::Direction::Click)?;
```

**Recent improvements (v0.4.0+):**
- macOS: Simulated input no longer affected by physical keyboard state
- macOS: No more sleeps during operation (only on struct drop)
- Better Unicode handling

---

### Configuration

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **serde** | 1.0 | Serialization framework | De facto standard, derive macros for easy implementation |
| **toml** | 0.9 | Config file format | Rust ecosystem standard (Cargo uses it), human-readable, good error messages |

**Why TOML over JSON:**
1. **Rust convention:** Cargo.toml established TOML as the Rust config standard
2. **Human-editable:** Users can manually edit config if needed
3. **Better error messages:** TOML parsers provide clear line/column errors
4. **Comments:** TOML supports comments, JSON does not

**Example config structure:**
```toml
[settings]
keystroke_delay_ms = 10
auto_start = false

[[macros]]
id = "greeting"
name = "Email Greeting"
hotkey = "Ctrl+Shift+G"
group = "Email"
sequence = "Hello,\n\nI hope this email finds you well.\n\n"

[[macros]]
id = "signature"
name = "Signature"
hotkey = "Ctrl+Shift+S"
group = "Email"
sequence = "Best regards,\nBrett"
```

---

### Build and Distribution

| Technology | Version | Purpose | Rationale |
|------------|---------|---------|-----------|
| **cargo** | (system) | Build system | Standard Rust toolchain |
| **cargo-bundle** | 0.6+ | macOS .app bundle | Creates proper .app with Info.plist, icon support |
| **cargo-packager** | latest | Windows installer | Creates .exe installer, supports DMG for macOS |

**Build strategy:**

1. **Development:** Standard `cargo build` / `cargo run`

2. **macOS release:**
   ```bash
   # Create .app bundle
   cargo bundle --release
   # Output: target/release/bundle/osx/KeyBlast.app
   ```

3. **Windows release:**
   ```bash
   # Create standalone .exe
   cargo build --release --target x86_64-pc-windows-msvc
   # Output: target/x86_64-pc-windows-msvc/release/keyblast.exe
   ```

4. **Cross-compilation:**
   - **macOS -> Windows:** Possible with `cargo-xwin` but has edge cases. Recommend GitHub Actions matrix builds instead.
   - **Windows -> macOS:** Not practical. Use GitHub Actions.

**Recommended CI approach:** GitHub Actions with matrix builds for each target platform. This avoids cross-compilation issues entirely.

---

## Alternatives Considered

### System Tray

| Alternative | Why Not Recommended |
|-------------|---------------------|
| **tray-item** | Simpler API but less maintained, fewer features |
| **systray-rs** | Maintainer explicitly discourages use ("this code was written in 2016") |
| **trayicon-rs** | Windows-focused, macOS support is community-contributed |
| **Full Tauri** | Overkill - adds WebView, JavaScript runtime when we just need tray |

### Global Hotkeys

| Alternative | Why Not Recommended |
|-------------|---------------------|
| **livesplit-hotkey** | Less documentation, smaller community |
| **hotkey-rs** | Less active maintenance than global-hotkey |
| **InputBot** | No macOS support, Windows/Linux only |
| **rdev** | Primarily for listening/recording, not just hotkey registration |

### Keystroke Injection

| Alternative | Why Not Recommended |
|-------------|---------------------|
| **rdev** | Can simulate input but less mature than enigo, last updated 2+ years ago |
| **InputBot** | No macOS support |
| **simulate** | Windows-only |
| **simulate_key.rs** | Thin wrapper around enigo - just use enigo directly |

### Configuration

| Alternative | Why Not Recommended |
|-------------|---------------------|
| **serde_json** | Works, but JSON lacks comments, less human-friendly |
| **serde_yaml** | Whitespace-sensitive, harder to hand-edit |
| **config crate** | Adds layer of abstraction we don't need |
| **confy** | Opinionated about file locations, less control |

---

## Platform-Specific Notes

### macOS

**Accessibility Permissions:**
- Keystroke injection requires Accessibility permission
- Enigo handles permission checking and prompting automatically
- Users will see: "KeyBlast would like to control this computer using accessibility features"
- Must be granted in System Preferences > Privacy & Security > Accessibility
- **First-run experience:** App should detect missing permission and guide user

**Code signing:**
- Unsigned apps will trigger Gatekeeper warnings
- For distribution: Apple Developer ID required for notarization
- For personal use: "Open Anyway" in Security preferences works

**App bundle requirements:**
- Must be a .app bundle to appear properly in system tray
- Raw binary can work but won't show proper app name in permission dialogs
- Use `cargo-bundle` to create proper bundle with Info.plist

**Event loop:**
- Must run on main thread
- Both tray-icon and global-hotkey require main thread for macOS

### Windows

**UIPI (User Interface Privilege Isolation):**
- Cannot inject keystrokes into admin/elevated windows without admin rights
- For normal apps (browser, editor, terminal): works fine
- If user runs KeyBlast as admin: can inject into admin windows

**No installer required:**
- Single .exe can be distributed and run directly
- For auto-start: app writes to registry (HKCU\Software\Microsoft\Windows\CurrentVersion\Run)

**Event loop:**
- Can run on any thread, but must be consistent (create and run on same thread)

### Linux (Future)

**Not in current scope but for reference:**
- tray-icon requires GTK dependencies: `libgtk-3-dev`, `libappindicator3-dev`
- global-hotkey requires X11 (no Wayland support yet)
- enigo supports X11 natively, Wayland is experimental

---

## Installation / Dependencies

### Cargo.toml

```toml
[package]
name = "keyblast"
version = "0.1.0"
edition = "2021"

[dependencies]
# System tray with native menus
tray-icon = "0.21"

# Global hotkey registration
global-hotkey = "0.7"

# Keystroke injection
enigo = "0.6"

# Configuration
serde = { version = "1.0", features = ["derive"] }
toml = "0.9"

# Error handling
anyhow = "1.0"

# Logging (optional but recommended)
log = "0.4"
env_logger = "0.11"

# macOS: Check accessibility permissions
[target.'cfg(target_os = "macos")'.dependencies]
macos-accessibility-client = "0.0.1"

[build-dependencies]
# For embedding icons
embed-resource = "2.4"  # Windows
```

### Platform Dependencies

**macOS:**
- Xcode Command Line Tools: `xcode-select --install`
- No additional libraries needed

**Windows:**
- Visual Studio Build Tools with C++ workload
- No additional libraries needed

---

## Confidence Assessment

| Component | Confidence | Rationale |
|-----------|------------|-----------|
| **tray-icon** | HIGH | Verified v0.21.3 (Jan 2026), 18.9k dependents, Tauri team maintained |
| **global-hotkey** | HIGH | Verified v0.7.0 (May 2025), same maintainers as tray-icon, clear docs |
| **enigo** | HIGH | Verified v0.6.1, 1.6k stars, active development, permission handling documented |
| **serde + toml** | HIGH | De facto Rust standards, millions of downloads |
| **cargo-bundle** | MEDIUM | Works but marked "early alpha", may have edge cases |
| **Event loop compatibility** | HIGH | All crates use same pattern (main thread event loop), tested together in Tauri |

---

## Key Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| macOS permission prompt UX is confusing | Medium | Medium | Add first-run permission check with clear guidance |
| Keystroke injection fails in specific apps | Low | Medium | Test in target apps (VS Code, terminal, browser); enigo is battle-tested |
| Hotkey conflicts with system/other apps | Medium | Low | Implement conflict detection, warn user, allow reassignment |
| App not signed for macOS distribution | N/A for personal use | High for distribution | Document signing process for future public release |

---

## Sources

### Official Documentation
- [tray-icon crate (crates.io)](https://crates.io/crates/tray-icon)
- [tray-icon GitHub](https://github.com/tauri-apps/tray-icon) - v0.21.3, January 2026
- [global-hotkey crate (crates.io)](https://crates.io/crates/global-hotkey)
- [global-hotkey docs.rs](https://docs.rs/global-hotkey/latest/global_hotkey/) - v0.7.0
- [enigo GitHub](https://github.com/enigo-rs/enigo) - v0.6.1, 1.6k stars
- [enigo docs.rs](https://docs.rs/enigo/latest/enigo/)
- [enigo Permissions.md](https://github.com/enigo-rs/enigo/blob/main/Permissions.md)
- [muda crate (crates.io)](https://crates.io/crates/muda)
- [Serde documentation](https://serde.rs/)
- [toml crate docs.rs](https://docs.rs/toml)

### Build/Distribution
- [cargo-bundle GitHub](https://github.com/burtonageo/cargo-bundle)
- [cargo-packager GitHub](https://github.com/crabnebula-dev/cargo-packager)
- [Tauri bundler docs](https://lib.rs/crates/tauri-bundler)

### Platform References
- [macOS Accessibility permissions (Apple)](https://support.apple.com/guide/mac-help/allow-accessibility-apps-to-access-your-mac-mh43185/mac)
- [macos-accessibility-client crate](https://github.com/next-slide-please/macos-accessibility-client)
- [Windows UIPI documentation](https://docs.microsoft.com/en-us/windows/win32/winauto/uiauto-securityoverview)

### Community Resources
- [Tauri System Tray docs](https://v2.tauri.app/learn/system-tray/)
- [Building a Native macOS Menu Bar App in Rust (Medium)](https://medium.com/@ekfqlwcjswl/building-a-native-macos-menu-bar-app-in-rust-0d55786db083)
- [winit Tray Icons Discussion](https://github.com/rust-windowing/winit/discussions/3835)

---

## Summary for Roadmap

**Stack is ready.** All components are well-maintained, cross-platform, and compatible with each other.

**Recommended phase order:**
1. **Phase 1:** System tray + basic menu (tray-icon) - proves cross-platform foundation
2. **Phase 2:** Global hotkey registration (global-hotkey) - adds core trigger mechanism
3. **Phase 3:** Keystroke injection (enigo) - completes core functionality
4. **Phase 4:** Configuration persistence (serde + toml) - adds user customization
5. **Phase 5:** Polish (permissions handling, auto-start, packaging)

**No blockers identified.** All crates have recent releases and active maintenance.

---

*Research conducted for KeyBlast roadmap planning. All versions verified via crates.io and GitHub as of 2026-01-16.*
