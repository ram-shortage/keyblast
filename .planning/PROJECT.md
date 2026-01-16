# KeyBlast

## What This Is

A lightweight macro playback application that sits in the system tray and types pre-configured keystroke sequences when triggered by hotkeys. Built for quickly inserting code snippets, text templates, and multi-step input sequences wherever the cursor is focused.

## Core Value

Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] System tray presence with menu for accessing configuration
- [ ] Global hotkey registration that works across all applications
- [ ] Keystroke injection supporting plain text and special keys (Enter, Tab, Escape, arrows)
- [ ] Configurable delay between keystrokes (instant to slow, per-macro)
- [ ] Configuration UI to create, edit, and delete macros
- [ ] Macro grouping/categories for organization (15-50 macros expected)
- [ ] Hotkey assignment per macro with conflict detection
- [ ] Persistent config file (load on startup, save on change)
- [ ] Optional auto-start at login (user-configurable)
- [ ] Cross-platform support: macOS and Windows

### Out of Scope

- Modifier key combos as output (Ctrl+C, Cmd+V) — plain text and special keys only
- Mouse automation — keyboard only
- Internet connectivity — fully offline
- Recording macros by watching input — manual configuration only
- Scripting/conditionals — simple sequence playback only

## Context

This is a personal productivity tool for repetitive text entry. Primary use cases are code snippets and text templates that need to be inserted frequently across different applications.

The "keyblast" name reflects the core action: blasting keystrokes into the focused application.

## Constraints

- **Tech stack**: Tauri (Rust backend, web frontend) — chosen for small binary size (~5-10MB) and cross-platform support
- **Distribution**: Standalone executable, no installer required if possible
- **Connectivity**: Zero network calls, fully offline operation
- **Platforms**: macOS (primary development), Windows (must work)

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri over Electron | Small binary (~5-10MB vs ~100MB+), Rust gives solid low-level keyboard access | — Pending |
| No modifier combos in output | Keeps scope simple, avoids platform-specific edge cases | — Pending |
| System tray over full window | Macro tool should be invisible until needed | — Pending |
| Config file over database | Simple, portable, human-readable | — Pending |

---
*Last updated: 2026-01-16 after initialization*
