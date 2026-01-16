# Architecture Research

**Project:** KeyBlast
**Researched:** 2026-01-16
**Confidence:** HIGH (based on official documentation and established crates)

## Component Overview

KeyBlast requires five core components that work together:

| Component | Responsibility | Primary Crate |
|-----------|---------------|---------------|
| **Event Loop** | Central coordination, event dispatch | `tao` or `winit` |
| **Tray Manager** | System tray icon, context menu | `tray-icon` (re-exports `muda` for menus) |
| **Hotkey Listener** | Global hotkey registration and events | `global-hotkey` |
| **Keystroke Injector** | Simulate keyboard input | `enigo` |
| **Config Manager** | Load/save macros, persist settings | `serde` + `toml` |

### Why These Crates

All three UI-related crates (`tray-icon`, `global-hotkey`, `muda`) are maintained by the Tauri team and share the same event loop integration pattern. This makes them naturally compatible:

- **tray-icon** (v0.21.3): Cross-platform tray icons, re-exports muda for menus
- **global-hotkey** (v0.7.0): Cross-platform global hotkey registration
- **enigo** (v0.6.1): Cross-platform keyboard/mouse simulation

## Data Flow

```
                    +------------------+
                    |   Config File    |
                    |   (macros.toml)  |
                    +--------+---------+
                             |
                             | load on startup
                             v
+----------------+    +------+-------+    +------------------+
| Global Hotkey  |    |              |    |  Keystroke       |
| Listener       +--->+  Event Loop  +--->+  Injector        |
| (background)   |    |  (main)      |    |  (on-demand)     |
+----------------+    +------+-------+    +------------------+
                             ^
                             |
                    +--------+--------+
                    |   Tray Menu     |
                    |   (user input)  |
                    +-----------------+
```

### Event Flow

1. **Startup**: Config Manager loads `macros.toml`, populates in-memory macro registry
2. **Idle**: Event loop runs, Hotkey Listener monitors for registered hotkeys
3. **Trigger**: User presses hotkey -> GlobalHotKeyEvent fired -> Event loop receives
4. **Execution**: Event loop looks up macro -> Keystroke Injector types the sequence
5. **Menu Action**: User clicks tray menu -> MenuEvent fired -> Event loop handles (quit, reload, etc.)

## Threading Model

### Single Main Thread Architecture (Recommended)

All three Tauri crates require an event loop on the thread where they're created. On macOS, this **must be the main thread**. The simplest architecture:

```
Main Thread:
  - Event loop (tao/winit)
  - GlobalHotKeyManager
  - TrayIcon + Menu
  - Keystroke injection (enigo)
  - Config loading

No background threads needed for core functionality.
```

### Why Single-Threaded Works

- **global-hotkey**: Uses OS-level hooks, events delivered to the event loop thread
- **tray-icon**: Events forwarded via receiver or proxy
- **enigo**: Synchronous API, called on-demand when macro fires
- **Config**: Loaded at startup, reloaded on menu action (not continuous watching)

### Alternative: Worker Thread for Injection

If keystroke playback is slow (high delay between keys), you could spawn a worker:

```
Main Thread:                     Worker Thread:
  - Event loop                     - Receives macro to play
  - Hotkey/Menu events             - Calls enigo sequentially
  - Sends macro to worker          - Blocks during playback
```

**Recommendation:** Start with single-threaded. Only add worker if playback blocking becomes a problem (unlikely given typical macro lengths).

## Component Boundaries

### Event Loop (Core Coordinator)

```rust
// Main application state
struct App {
    tray_icon: TrayIcon,
    hotkey_manager: GlobalHotKeyManager,
    macros: HashMap<HotKeyId, Macro>,
    enigo: Enigo,
}

// Custom event type for unified handling
enum UserEvent {
    TrayIconEvent(TrayIconEvent),
    MenuEvent(MenuEvent),
    HotKeyEvent(GlobalHotKeyEvent),
}
```

The event loop:
- Owns all managers and state
- Routes events to appropriate handlers
- Never blocks (enigo calls are fast for short sequences)

### Tray Manager Interface

```rust
// What the tray provides to the app
trait TrayManager {
    fn set_tooltip(&self, text: &str);
    fn set_icon(&self, icon: Icon);
    fn update_menu(&self, macros: &[MacroGroup]);
}
```

The tray:
- Displays current macro count in tooltip
- Shows context menu with Quit, Reload Config, About
- Does NOT show macro list in menu (too many items expected)

### Hotkey Listener Interface

```rust
// What the hotkey system provides
trait HotkeyListener {
    fn register(&mut self, hotkey: HotKey) -> HotKeyId;
    fn unregister(&mut self, id: HotKeyId);
    fn register_all(&mut self, macros: &[Macro]) -> Vec<(HotKeyId, MacroId)>;
}
```

The listener:
- Registers hotkeys from config at startup
- Re-registers on config reload
- Returns mapping of HotKeyId -> MacroId for lookup

### Keystroke Injector Interface

```rust
// What the injector provides
trait KeystrokeInjector {
    fn type_text(&mut self, text: &str) -> Result<()>;
    fn type_key(&mut self, key: Key) -> Result<()>;
    fn type_sequence(&mut self, sequence: &[KeystrokeAction], delay_ms: u64) -> Result<()>;
}

enum KeystrokeAction {
    Text(String),
    Key(SpecialKey),
    Delay(u64), // milliseconds
}

enum SpecialKey {
    Enter,
    Tab,
    Escape,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Backspace,
    Delete,
    // etc.
}
```

The injector:
- Wraps enigo
- Handles delay between keystrokes (configurable per-macro)
- Maps KeyBlast's key types to enigo's Key enum

### Config Manager Interface

```rust
// What the config system provides
trait ConfigManager {
    fn load() -> Result<Config>;
    fn save(&self, config: &Config) -> Result<()>;
    fn config_path() -> PathBuf;
}

struct Config {
    version: u32,
    settings: Settings,
    groups: Vec<MacroGroup>,
}

struct Settings {
    default_delay_ms: u64,
    start_at_login: bool,
}

struct MacroGroup {
    name: String,
    macros: Vec<Macro>,
}

struct Macro {
    id: MacroId,
    name: String,
    hotkey: HotKeyDef,
    sequence: Vec<KeystrokeAction>,
    delay_ms: Option<u64>, // overrides default
}
```

The config:
- TOML file in platform-appropriate config directory
- Human-readable and editable
- Versioned for future migrations

## Suggested Build Order

Based on dependencies between components:

### Phase 1: Skeleton with Event Loop

Build the event loop foundation that everything hangs off of.

**Components:**
1. Basic Cargo project with dependencies
2. Event loop (tao) that runs without crashing
3. Tray icon that shows up with placeholder icon
4. Simple menu with just "Quit"

**Why first:** Everything else plugs into this. If the event loop works, you can iterate on everything else.

**Deliverable:** App that shows tray icon, clicking Quit exits.

### Phase 2: Global Hotkey Registration

Add hotkey detection (input side).

**Components:**
1. GlobalHotKeyManager creation
2. Register a hardcoded test hotkey (e.g., Ctrl+Shift+K)
3. Event forwarding to main loop
4. Log when hotkey is pressed

**Why second:** This is the input mechanism. Need to prove hotkeys work before wiring up output.

**Deliverable:** App logs "Hotkey pressed!" when you hit Ctrl+Shift+K.

### Phase 3: Keystroke Injection

Add keystroke output (output side).

**Components:**
1. Enigo initialization
2. Type hardcoded text when hotkey fires
3. Handle special keys (Enter, Tab)
4. Configurable delay between keystrokes

**Why third:** Now you have input -> output working. This is the core value loop.

**Deliverable:** Press Ctrl+Shift+K, app types "Hello, World!\n" into focused app.

### Phase 4: Config File

Make macros configurable.

**Components:**
1. Define Config structs with serde
2. Create default config on first run
3. Load config at startup
4. Register hotkeys from config
5. Look up macro on hotkey press

**Why fourth:** Core functionality works. Now make it configurable.

**Deliverable:** Edit macros.toml, restart app, new macros work.

### Phase 5: Configuration UI

Add ability to modify config without editing files.

**Components:**
1. Extend tray menu with "Open Config Folder"
2. Add "Reload Config" menu item
3. Add hotkey conflict detection on load
4. Consider simple native dialog for adding macros (optional)

**Why fifth:** Power users can edit TOML. This makes it accessible to everyone.

**Deliverable:** Full working app with config management.

### Phase 6: Polish and Platform Testing

Cross-platform validation and edge cases.

**Components:**
1. Windows testing and fixes
2. macOS accessibility permission handling
3. Auto-start at login (platform-specific)
4. Error handling and user feedback
5. App icon and branding

## Platform Abstraction

### What's Abstracted by Libraries

| Concern | Abstracted By | Notes |
|---------|--------------|-------|
| Tray icon | tray-icon | Unified API, platform details hidden |
| Menus | muda (via tray-icon) | Same menu structure works everywhere |
| Hotkeys | global-hotkey | Same registration API everywhere |
| Key simulation | enigo | Same Keyboard trait everywhere |

### What KeyBlast Must Handle

| Concern | macOS | Windows |
|---------|-------|---------|
| Config directory | `~/Library/Application Support/keyblast/` | `%APPDATA%\keyblast\` |
| Auto-start | LaunchAgent plist | Registry or Startup folder |
| Accessibility | Must prompt for permission | Usually not needed |
| Icon format | PNG works | ICO preferred but PNG works |

### Accessibility Permission (macOS Critical)

macOS requires Accessibility permission for both:
- **global-hotkey**: To receive hotkey events
- **enigo**: To inject keystrokes

Without this permission:
- global-hotkey events are silently ignored
- enigo calls fail silently or error

**Detection approach:**
```rust
// Use macos-accessibility-client crate
#[cfg(target_os = "macos")]
fn check_accessibility() -> bool {
    macos_accessibility_client::accessibility::application_is_trusted_with_prompt()
}
```

**User experience:**
- On first launch, check if trusted
- If not, show dialog explaining why permission is needed
- Open System Preferences to correct pane
- Re-check after user grants permission

### Config Directory Resolution

Use the `dirs` crate for cross-platform config paths:

```rust
use dirs::config_dir;

fn config_path() -> PathBuf {
    let mut path = config_dir().expect("Could not find config directory");
    path.push("keyblast");
    path.push("macros.toml");
    path
}
```

### Auto-Start Implementation

**macOS (LaunchAgent):**
```rust
// Create ~/Library/LaunchAgents/com.keyblast.app.plist
// Set RunAtLoad = true
```

**Windows (Registry):**
```rust
// Add to HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Run
```

Consider using the `auto-launch` crate which handles both platforms.

## Architecture Anti-Patterns to Avoid

### 1. Blocking the Event Loop

**Bad:** Running enigo.text() with long delays in the event loop thread
**Why bad:** UI becomes unresponsive, hotkeys queue up
**Solution:** Keep delays reasonable (< 50ms typical), or spawn worker thread for slow macros

### 2. Separate Threads for Each Component

**Bad:** Tray in one thread, hotkeys in another, coordination via channels
**Why bad:** macOS requires main thread for both, over-complicated
**Solution:** Single main thread with event-driven design

### 3. Polling for Events

**Bad:** Busy-loop calling try_recv() repeatedly
**Why bad:** Wastes CPU, drains battery
**Solution:** Use set_event_handler with EventLoopProxy to wake the loop

### 4. Global Mutable State

**Bad:** static mut MACROS: Vec<Macro> or lazy_static for app state
**Why bad:** Thread safety issues, hard to test, hard to reason about
**Solution:** App struct owns all state, passed through event loop

### 5. Ignoring Platform Differences

**Bad:** Assuming accessibility "just works"
**Why bad:** macOS will silently fail without permission
**Solution:** Detect and guide user through permission setup

## Reference: Espanso Architecture

Espanso (a production Rust text expander) uses a modular crate architecture:

| Crate | Purpose |
|-------|---------|
| espanso-detect | Keyboard input detection |
| espanso-inject | Text injection |
| espanso-match | Trigger matching |
| espanso-config | Configuration parsing |
| espanso-engine | Core coordination |

**Lessons for KeyBlast:**
- Separate concerns into modules (not separate crates for a smaller app)
- Platform-specific code isolated behind traits
- Config is its own subsystem

KeyBlast is simpler (hotkeys not typing detection), so a single crate with modules suffices:

```
src/
  main.rs        // Entry point, event loop
  app.rs         // App struct, state management
  config.rs      // Config loading/saving
  hotkey.rs      // Hotkey registration wrapper
  inject.rs      // Keystroke injection wrapper
  tray.rs        // Tray icon and menu setup
  platform/
    mod.rs
    macos.rs     // Accessibility, auto-start
    windows.rs   // Auto-start
```

## Sources

- [tray-icon GitHub](https://github.com/tauri-apps/tray-icon) - Official repository
- [tray-icon docs.rs](https://docs.rs/tray-icon/latest/tray_icon/) - API documentation
- [global-hotkey GitHub](https://github.com/tauri-apps/global-hotkey) - Official repository
- [global-hotkey docs.rs](https://docs.rs/global-hotkey/latest/global_hotkey/) - API documentation
- [enigo GitHub](https://github.com/enigo-rs/enigo) - Official repository
- [enigo docs.rs](https://docs.rs/enigo/latest/enigo/) - API documentation
- [rdev GitHub](https://github.com/Narsil/rdev) - macOS accessibility notes
- [espanso GitHub](https://github.com/espanso/espanso) - Reference architecture
- [muda crates.io](https://crates.io/crates/muda) - Menu crate info
- [tao GitHub](https://github.com/tauri-apps/tao) - Event loop library
