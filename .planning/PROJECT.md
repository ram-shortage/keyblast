# KeyBlast

## What This Is

A lightweight macro playback application that sits in the system tray and types pre-configured keystroke sequences when triggered by hotkeys. Built for quickly inserting code snippets, text templates, and multi-step input sequences wherever the cursor is focused.

## Core Value

Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.

## Current Milestone: v2.0 Quality & Power

**Goal:** Production hardening, async execution, expanded macro capabilities, and better UX

**Target features:**
- Search/filter macros by name
- Async macro execution (non-blocking)
- Stop running macro capability
- Expanded macro DSL ({Delay}, {Paste}, modifier keys)
- Run macro from menu (click to execute)
- File logging with "Open Logs" menu
- Config validation (duplicates, conflicts)
- Persist enabled/disabled state
- Bug fixes (Windows config save, import merge)

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

### Active (v2.0)

**UX Improvements:**
- [ ] Search/filter macros by name in tray menu
- [ ] Run macro by clicking menu item (not just hotkey)
- [ ] File logging with "Open Logs..." menu item
- [ ] Persist enabled/disabled state across restarts

**Async Execution:**
- [ ] Move macro execution off event loop thread
- [ ] Stop current macro hotkey/menu item
- [ ] Tray stays responsive during long macros

**Expanded DSL:**
- [ ] `{Delay 500}` — pause mid-macro (milliseconds)
- [ ] `{KeyDown Ctrl}` / `{KeyUp Ctrl}` — modifier press/release
- [ ] `{Paste}` — paste from clipboard
- [ ] `{{` / `}}` — literal braces escape

**Robustness:**
- [ ] Config validation: detect duplicate names/hotkeys
- [ ] Surface conflicts in tray menu (not just println)
- [ ] Use stable macro IDs for delete (not names)
- [ ] Fix import merge: de-dupe within imported file
- [ ] Fix Windows config save (fs::rename doesn't overwrite)

### Out of Scope

- Modifier key combos as output (Ctrl+C, Cmd+V) — plain text and special keys only
- Mouse automation — keyboard only
- Internet connectivity — fully offline
- Recording macros by watching input — manual configuration only
- Scripting/conditionals — simple sequence playback only
- GUI config editor — tray menu + TOML file editing sufficient for v2

## Context

v1.0 shipped and is functional. v2.0 focuses on:
1. **Responsiveness** — async execution prevents UI freeze during long macros
2. **Power user features** — expanded DSL, search, click-to-run
3. **Polish** — logging, validation, bug fixes

The codebase is ~600 lines of Rust across 7 modules. Well-structured for extension.

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

---
*Last updated: 2026-01-17 after v2.0 milestone start*
