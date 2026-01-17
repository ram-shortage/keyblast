# Milestones: KeyBlast

## Completed Milestones

### v2.0 — Quality & Power (2026-01-17)

**Goal:** Production hardening, async execution, expanded macro capabilities, and better UX

**Shipped:**
- Async macro execution (non-blocking UI)
- Stop running macro (Ctrl+Escape / menu item)
- Run macro by clicking menu item
- File logging with "Open Logs..." menu
- Persist enabled/disabled state across restarts
- Expanded DSL: {Delay N}, {KeyDown/KeyUp}, {Paste}, {{/}}
- Config validation (duplicate names, hotkeys, IDs)
- Stable UUID-based macro deletion
- Windows atomic config save fix
- Import de-duplication
- Custom tray icon (lightning bolt)
- Code review fixes (v2.1, v2.2)

**Requirements:** 11/11 complete
**Phases:** 4
**Plans:** 9

---

### v1.0 — Foundation Release (2026-01-16)

**Goal:** Functional hotkey-triggered keystroke injector with cross-platform support

**Shipped:**
- System tray presence with enable/disable toggle
- Global hotkey registration across all applications
- Keystroke injection with special keys (Enter, Tab, arrows, etc.)
- Per-macro keystroke delay configuration
- TOML-based persistent configuration
- Create/edit/delete macros via tray menu
- Export/import macros
- Macro grouping/categories
- Hotkey conflict detection and suggestions
- Auto-start at login
- macOS Accessibility permission guidance
- Tray icon flash feedback
- Cross-platform: macOS and Windows

**Requirements:** 13/13 complete
**Phases:** 6
**Plans:** 13
**Duration:** ~1.6 hours execution time

---

## Current Milestone

See PROJECT.md for v2.1 scope.
