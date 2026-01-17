# KeyBlast

## What This Is

A lightweight macro playback application that sits in the system tray and types pre-configured keystroke sequences when triggered by hotkeys. Built for quickly inserting code snippets, text templates, and multi-step input sequences wherever the cursor is focused.

## Core Value

Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.

## Current Milestone: v2.1 Windows Polish

**Goal:** Professional Windows experience with proper executable packaging and error feedback

**Target features:**
- Suppress Windows console window (run as GUI app)
- Error notifications via system tray
- Embedded executable icon for Windows
- Example macros in default config

## Requirements

### Validated (v1.0)

- ✓ System tray presence with menu — v1.0
- ✓ Global hotkey registration across all applications — v1.0
- ✓ Keystroke injection with special keys — v1.0
- ✓ Per-macro keystroke delay — v1.0
- ✓ TOML configuration persistence — v1.0
- ✓ Create/edit/delete macros via tray — v1.0
- ✓ Export/import macros — v1.0
- ✓ Macro grouping/categories — v1.0
- ✓ Hotkey conflict detection — v1.0
- ✓ Auto-start at login — v1.0
- ✓ macOS Accessibility permission UX — v1.0
- ✓ Tray icon flash feedback — v1.0
- ✓ Cross-platform (macOS/Windows) — v1.0

### Validated (v2.0)

- ✓ Async macro execution (non-blocking UI) — v2.0
- ✓ Stop running macro (Ctrl+Escape / menu) — v2.0
- ✓ Run macro by clicking menu item — v2.0
- ✓ File logging with "Open Logs..." menu — v2.0
- ✓ Persist enabled/disabled state — v2.0
- ✓ Expanded DSL: {Delay N}, {KeyDown/KeyUp}, {Paste}, {{/}} — v2.0
- ✓ Config validation (duplicate names, hotkeys, IDs) — v2.0
- ✓ Stable UUID-based macro deletion — v2.0
- ✓ Windows atomic config save fix — v2.0
- ✓ Import de-duplication — v2.0
- ✓ Custom tray icon (lightning bolt) — v2.0

### Active (v2.1)

**Windows Polish:**
- [ ] Suppress console window — `#![windows_subsystem = "windows"]`
- [ ] Embedded .exe icon — shows in Explorer, taskbar, Alt+Tab
- [ ] Error notifications — tray alerts for injection failures, permission issues

**Onboarding:**
- [ ] Example macros in default config — two templates to demonstrate usage

### Out of Scope

- Modifier key combos as output (Ctrl+C, Cmd+V) — plain text and special keys only
- Mouse automation — keyboard only
- Internet connectivity — fully offline
- Recording macros by watching input — manual configuration only
- Scripting/conditionals — simple sequence playback only
- GUI config editor — tray menu + TOML file editing sufficient
- Search/filter in tray menu — native menus don't support this well

## Context

v2.0 shipped with async execution, expanded DSL, and production hardening. v2.1 focuses on:
1. **Windows professionalism** — no console window, proper .exe icon
2. **Error visibility** — notifications instead of silent failures
3. **Onboarding** — example macros help new users understand the format

The codebase is ~1200 lines of Rust across 8 modules.

## Build Instructions

**macOS (native):**
```bash
cargo build --release
# Output: target/release/keyblast
```

**Windows (cross-compile from macOS):**
```bash
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/keyblast.exe
```

Requires: `rustup target add x86_64-pc-windows-gnu` and MinGW toolchain.

## Constraints

- **Tech stack**: Pure Rust — no web frontend, system tray menu only for UI
- **Distribution**: Standalone executable, no installer required
- **Connectivity**: Zero network calls, fully offline operation
- **Platforms**: macOS (primary), Windows (must work)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Pure Rust over Tauri/Electron | No web frontend needed — system tray suffices | ✓ Good |
| No modifier combos in output | Keeps scope simple, avoids platform edge cases | ✓ Good |
| System tray over full window | Macro tool should be invisible until needed | ✓ Good |
| Config file over database | Simple, portable, human-readable | ✓ Good |
| 50ms modifier release delay | macOS needs longer delay to prevent symbol bleed | ✓ Good |
| File watcher for hot-reload | Edit config externally, changes apply automatically | ✓ Good |
| auto-launch crate | Cross-platform login items, used by Tauri | ✓ Good |
| Flat menu for Run Macro | Native tray menus can't support search | ✓ Good |

---
*Last updated: 2026-01-17 after v2.1 milestone start*
