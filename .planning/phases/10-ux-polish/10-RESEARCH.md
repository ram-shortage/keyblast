# Phase 10: UX Polish - Research

**Researched:** 2026-01-17
**Domain:** File logging, tray menu UX, state persistence, application icons
**Confidence:** MEDIUM-HIGH

## Summary

Phase 10 addresses five requirements: macro search/filter (UX-01), click-to-run (UX-02), file logging with "Open Logs" (UX-03), persist enabled/disabled state (UX-04), and custom app icon (UX-05).

Research reveals that:
1. **File logging**: Use `tracing` + `tracing-appender` for rolling log files with daily rotation. The `open` crate handles cross-platform "Open Logs" functionality.
2. **Search/filter**: Native tray menus (muda) don't support search input. Implement by dynamically rebuilding menu with filtered items, or accept menu-based browsing without search.
3. **Click-to-run**: Straightforward - add menu item handler that triggers macro execution (already have execution infrastructure from Phase 7).
4. **Persist state**: Add `enabled` field to config TOML and load/save on state change.
5. **Custom icon**: Need PNG icons at multiple resolutions (22x22 for tray, larger for app). Current blue square is placeholder.

**Primary recommendation:** Prioritize logging and persistence (low effort, high value). Search/filter may require scope discussion - native tray menus don't support text input, so "search" would need creative UX solution or scoping down to "show all macros in flat list for easy browsing."

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tracing | 0.1 | Structured logging/instrumentation | De facto standard for Rust logging, used by Tokio ecosystem |
| tracing-subscriber | 0.3 | Subscriber implementation with formatting | Official companion to tracing |
| tracing-appender | 0.2 | Rolling file appender, non-blocking writes | Official tracing extension for file logging |
| open | 5.3 | Open files with system default app | Cross-platform, widely used, 105M downloads |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| dirs | 5.0 | Platform-specific directories | Already in use - for log directory path |
| image | 0.25 | Icon loading/processing | Already in use |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| tracing | log + env_logger | `log` is simpler but `tracing` is more powerful and becoming standard |
| tracing-appender | custom file writer | tracing-appender handles rotation, non-blocking, and cleanup |
| open | std::process::Command | `open` crate handles platform detection automatically |

**Installation:**
```bash
cargo add tracing tracing-subscriber tracing-appender open
```

## Architecture Patterns

### Recommended Project Structure
```
src/
├── logging.rs       # NEW: tracing setup, log file management
├── app.rs           # Extended: add `enabled` to persisted state
├── config.rs        # Extended: add AppSettings struct
├── tray.rs          # Extended: add click-to-run, Open Logs menu item
├── main.rs          # Extended: init logging early in main()
└── ...
assets/
├── icon.png         # Tray icon - 44x44 (22pt @2x for macOS Retina)
├── icon-flash.png   # Flash variant
├── icon-16.png      # Windows small tray
├── icon-32.png      # Windows standard
└── icon-256.png     # App icon for Windows
```

### Pattern 1: Logging Initialization with Guard
**What:** Initialize tracing subscriber early in main(), keep guard alive for duration
**When to use:** Any application needing file logging
**Example:**
```rust
// Source: https://docs.rs/tracing-appender/latest/tracing_appender/
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("keyblast")
        .join("logs");

    // Create log directory if needed
    let _ = std::fs::create_dir_all(&log_dir);

    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("keyblast")
        .filename_suffix("log")
        .max_log_files(7)  // Keep 7 days of logs
        .build(&log_dir)
        .expect("failed to initialize rolling file appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)  // No ANSI colors in log files
        .init();

    guard  // MUST keep this alive!
}

fn main() {
    let _guard = init_logging();
    // ... rest of application
}
```

### Pattern 2: Cross-Platform Open File/Directory
**What:** Open log directory or specific log file with system default application
**When to use:** "Open Logs..." menu item
**Example:**
```rust
// Source: https://docs.rs/open/latest/open/
fn open_logs_directory() {
    let log_dir = dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("keyblast")
        .join("logs");

    if let Err(e) = open::that(&log_dir) {
        eprintln!("Failed to open logs directory: {}", e);
    }
}
```

### Pattern 3: Persist Enabled State in Config
**What:** Add app-level settings to config file, load/save on change
**When to use:** UX-04 - persist enabled/disabled state
**Example:**
```rust
// In config.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default)]
    pub macros: Vec<MacroDefinition>,
    /// Application settings (enabled state, etc.)
    #[serde(default)]
    pub settings: AppSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AppSettings {
    /// Whether macros are enabled (default: true)
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}
```

### Pattern 4: Click-to-Run Menu Items
**What:** Add run action to macro menu items
**When to use:** UX-02 - run macro by clicking
**Example:**
```rust
// In tray.rs - modify macro submenu to include "Run" action
let macro_submenu = Submenu::new(&label, true);

let run_item = MenuItem::new("Run", true, None::<Accelerator>);
let run_id = run_item.id().clone();
run_macro_ids.insert(run_id, macro_def.id);

let delete_item = MenuItem::new("Delete", true, None::<Accelerator>);
// ...

macro_submenu.append(&run_item).expect("Failed to add run item");
macro_submenu.append(&delete_item).expect("Failed to add delete item");
```

### Anti-Patterns to Avoid
- **Calling init() multiple times:** Use `try_init()` if there's any chance of double initialization
- **Dropping the WorkerGuard early:** Logs may be lost if guard is dropped before program exits
- **Blocking file operations on event loop:** Use tracing's non_blocking writer
- **Complex search UX in tray menu:** Native menus don't support text input - keep it simple

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Log file rotation | Custom rotation logic | tracing-appender RollingFileAppender | Handles date boundaries, cleanup, atomic writes |
| Log file cleanup | Manual rm old files | tracing-appender max_log_files | Built-in, race-condition free |
| Non-blocking logging | Spawn thread, channel | tracing-appender non_blocking | Handles backpressure, graceful shutdown |
| Open file with default app | Platform #[cfg] blocks | open crate | Handles WSL, xdg-open fallbacks, edge cases |
| Icon format conversion | Manual PNG to ICO | iconutil (macOS), image crate | Complex format requirements per-platform |

**Key insight:** Logging and file operations have many edge cases (permissions, atomic writes, platform differences). The tracing ecosystem handles these robustly.

## Common Pitfalls

### Pitfall 1: WorkerGuard Dropped Early
**What goes wrong:** Log messages lost, especially at program shutdown
**Why it happens:** Guard goes out of scope before all logs flushed
**How to avoid:** Keep guard alive in main() for entire program lifetime
**Warning signs:** Missing logs at end of execution, especially panic messages

### Pitfall 2: init() vs try_init()
**What goes wrong:** Panic "global subscriber already installed"
**Why it happens:** Calling init() twice (e.g., in tests, or if library also initializes)
**How to avoid:** Use try_init() which returns Result instead of panicking
**Warning signs:** Panic during startup or in test suite

### Pitfall 3: Tray Menu "Search" Expectations
**What goes wrong:** User expects Alfred/Spotlight-style search popup
**Why it happens:** Native tray menus don't support text input fields
**How to avoid:** Set clear expectations - "filter" via submenu categories, not free-text search
**Warning signs:** UX requirement asking for "search box" in tray menu

### Pitfall 4: Icon Resolution Mismatch
**What goes wrong:** Blurry or pixelated tray icons
**Why it happens:** Using wrong resolution for platform DPI
**How to avoid:** Provide 22x22 @1x and 44x44 @2x for macOS, 16x16 for Windows tray
**Warning signs:** Icon looks fuzzy on Retina displays

### Pitfall 5: Log Directory Doesn't Exist
**What goes wrong:** RollingFileAppender panics or fails silently
**Why it happens:** First run, or user deleted directory
**How to avoid:** Create log directory with create_dir_all before initializing appender
**Warning signs:** No log files created, panic on startup

## Code Examples

Verified patterns from official sources:

### Complete Logging Setup
```rust
// Source: https://docs.rs/tracing-appender/latest/tracing_appender/
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt;

pub fn log_directory() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("keyblast")
        .join("logs")
}

pub fn init_file_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    let log_dir = log_directory();

    // Ensure directory exists
    if std::fs::create_dir_all(&log_dir).is_err() {
        eprintln!("Warning: Could not create log directory: {}", log_dir.display());
        return None;
    }

    let file_appender = match RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("keyblast")
        .filename_suffix("log")
        .max_log_files(7)
        .build(&log_dir)
    {
        Ok(appender) => appender,
        Err(e) => {
            eprintln!("Warning: Could not create log appender: {}", e);
            return None;
        }
    };

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    if tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .try_init()
        .is_err()
    {
        eprintln!("Warning: Logging already initialized");
    }

    Some(guard)
}
```

### Open Logs Menu Handler
```rust
// Source: https://docs.rs/open/latest/open/
fn handle_open_logs() {
    let log_dir = log_directory();

    if !log_dir.exists() {
        eprintln!("Log directory does not exist: {}", log_dir.display());
        return;
    }

    // Open directory in system file browser (Finder on macOS, Explorer on Windows)
    if let Err(e) = open::that(&log_dir) {
        eprintln!("Failed to open logs directory: {}", e);
    }
}
```

### Persist Enabled State
```rust
// When toggle is clicked in main.rs:
self.state.toggle();

// Also update config and save
if let Some(ref mut cfg) = self.config {
    cfg.settings.enabled = self.state.enabled;
    if let Err(e) = config::save_config(cfg) {
        eprintln!("Failed to save enabled state: {}", e);
    }
}

// On startup, load enabled state from config:
if let Some(ref cfg) = self.config {
    self.state.enabled = cfg.settings.enabled;
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| log crate + env_logger | tracing + tracing-subscriber | 2020+ | tracing is now de facto standard for new Rust projects |
| println! debugging | tracing macros (info!, debug!, error!) | - | Structured, filterable, can route to files |
| Manual platform detection for open | open crate | - | Handles edge cases automatically |

**Deprecated/outdated:**
- `log` crate is still maintained but `tracing` is preferred for new projects
- Manual `#[cfg(target_os)]` blocks for opening files - use `open` crate instead

## UX-01: Search/Filter Analysis

Native tray menus (muda on Rust, NSMenu on macOS, Win32 menu on Windows) do **not** support text input fields. Options:

### Option A: Scoped Down - Flat Macro List (Recommended)
Instead of "search", show all macros in a flat, alphabetically-sorted list under a "Run Macro" submenu. Users can visually scan. This is simple and works with native menus.

**Pros:** Simple, no new UI paradigm, works with existing muda
**Cons:** Doesn't scale well past 20-30 macros

### Option B: Category-Based Filtering
Show macros organized by group (already implemented). Users filter by clicking group submenus.

**Pros:** Already partially implemented
**Cons:** Not "search" in the traditional sense

### Option C: External Search Window (Complex)
Create a small floating window with a text input that appears on a hotkey (like Spotlight). This is significantly more work and may be out of scope for v2.0.

**Pros:** True search experience
**Cons:** Requires GUI framework, significant new code, cross-platform complexity

**Recommendation:** Implement Option A (flat list under "Run Macro" submenu) for v2.0. This satisfies "user can find and run macro by name" without over-engineering. True search could be a v3.0 feature.

## UX-05: Icon Requirements

### macOS Menu Bar Icon
- Format: PNG
- Size: 22x22 points (44x44 pixels for @2x Retina)
- Style: Template image (monochrome, adapts to light/dark mode) OR colored
- Current: 64x64 blue square (needs resize and design)

### Windows System Tray Icon
- Format: PNG (tray-icon converts internally)
- Size: 16x16 pixels primary, include 32x32 for high DPI
- Style: Colored icons are standard on Windows

### Recommendation
Create a simple, recognizable icon:
- Lightning bolt (suggests "blast" / speed)
- Keyboard key with lightning
- Simple "KB" monogram

Use an online tool or design app to create:
1. 44x44 PNG for macOS tray (will work as template)
2. 16x16 PNG for Windows tray
3. Optionally: 256x256 or 512x512 for app icon

## Open Questions

Things that couldn't be fully resolved:

1. **Search UX Scope**
   - What we know: Native tray menus don't support text input
   - What's unclear: Does stakeholder expect true search or is flat list acceptable?
   - Recommendation: Clarify with user; implement flat list for v2.0

2. **Icon Design**
   - What we know: Current icon is placeholder blue square
   - What's unclear: Design requirements, branding, who creates the icon
   - Recommendation: Create simple lightning bolt icon or defer to user

3. **Log Retention Policy**
   - What we know: tracing-appender supports max_log_files
   - What's unclear: How many days of logs to keep?
   - Recommendation: Default to 7 days, configurable in future

## Sources

### Primary (HIGH confidence)
- [tracing-appender docs](https://docs.rs/tracing-appender/latest/tracing_appender/) - Rolling file appender API, non-blocking setup
- [tracing-subscriber docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) - fmt subscriber configuration
- [open crate docs](https://docs.rs/open/latest/open/) - Cross-platform file opening

### Secondary (MEDIUM confidence)
- [muda crate GitHub](https://github.com/tauri-apps/muda) - Menu structure, no search support confirmed
- [ToDesktop tray icon docs](https://www.todesktop.com/docs/trays/tray-icons) - Icon size recommendations
- [Microsoft icon guidelines](https://learn.microsoft.com/en-us/windows/apps/design/style/iconography/app-icon-construction) - Windows icon sizes

### Tertiary (LOW confidence)
- WebSearch results on Alfred/Spotlight patterns - for UX inspiration only
- Various icon design articles - general guidance

## Metadata

**Confidence breakdown:**
- File logging (tracing-appender): HIGH - well-documented official crate
- Open files (open crate): HIGH - well-documented, widely used
- Persist state: HIGH - simple config extension pattern
- Click-to-run: HIGH - straightforward menu handler
- Search/filter UX: MEDIUM - technical options clear, UX scope unclear
- Icon requirements: MEDIUM - platform guidelines found, design TBD

**Research date:** 2026-01-17
**Valid until:** 60 days (stable domain, mature crates)
