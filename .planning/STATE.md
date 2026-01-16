# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-16)

**Core value:** Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.
**Current focus:** Phase 6 — Platform Polish

## Current Position

Phase: 6 of 6 (Platform Polish)
Plan: 1 of 2 complete
Status: In progress
Last activity: 2026-01-16 — Completed 06-01-PLAN.md (Auto-Start at Login)

Progress: █████████░ 92%

## Performance Metrics

**Velocity:**
- Total plans completed: 12
- Average duration: ~8 min
- Total execution time: ~1.5 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 1. Foundation | 1 | ~15 min | ~15 min |
| 2. Global Hotkeys | 2 | ~14 min | ~7 min |
| 3. Keystroke Injection | 2 | ~23 min | ~11 min |
| 4. Configuration | 2 | ~5 min | ~2.5 min |
| 5. Configuration UI | 3 | ~21 min | ~7 min |
| 6. Platform Polish | 1 | ~7 min | ~7 min |

**Recent Trend:**
- Last 5 plans: 05-01 (~6 min), 05-02 (~7 min), 05-03 (~8 min), 06-01 (~7 min)
- Trend: Consistent ~7 min per plan in recent phases

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

| Decision | Phase | Rationale |
|----------|-------|-----------|
| Added winit event loop | 01-01 | macOS requires active event loop for tray icon visibility |
| Used CheckMenuItem for toggle | 01-01 | Native checkmark display follows platform conventions |
| Module separation (app/tray/main) | 01-01 | Prepares codebase for complexity in phases 2-6 |
| EventLoopProxy pattern for hotkeys | 02-01 | Forward GlobalHotKeyEvent to winit loop, avoids polling delays |
| HotkeyManager in resumed() | 02-01 | macOS requires main thread for GlobalHotKeyManager |
| Test hotkey Ctrl+Shift+K | 02-01 | Low conflict probability with system shortcuts |
| RegisterResult enum over Result | 02-02 | Distinguishes internal vs external conflicts for better UX |
| Tier 1/2 candidate hotkeys | 02-02 | Ctrl+Shift and Ctrl+Alt combinations rarely conflict |
| enigo 0.6 for injection | 03-01 | Most mature cross-platform keystroke simulation library |
| Release modifiers before typing | 03-01 | Prevents Ctrl/Shift/Alt interference from hotkey trigger |
| 50ms delay after modifier release | 03-02 | macOS needs longer delay; 10ms caused Apple symbol bleed-through |
| Bulk vs char-by-char delay modes | 03-01 | 0ms uses fast text(), >0ms for slow applications |
| dirs crate for config paths | 04-01 | Cross-platform config directory detection |
| Atomic config writes | 04-01 | Temp file + rename prevents corruption on crash |
| delay_ms defaults to 0 | 04-01 | Instant/bulk typing is the common case |
| HashMap<u32, MacroDefinition> for lookup | 04-02 | O(1) macro lookup when hotkey triggers |
| Default example macro creation | 04-02 | Provides template for users to edit |
| Group field with skip_serializing_if | 05-01 | Keeps TOML clean when group is unused |
| Export/import with arbitrary paths | 05-01 | User chooses backup/share locations |
| Groups sorted alphabetically, Ungrouped last | 05-02 | Consistent ordering for users |
| Each macro as submenu with Delete action | 05-02 | Delete via menu, Edit via config file |
| Menu rebuild via set_menu() | 05-02 | Refresh tray dynamically after config changes |
| File watcher mpsc channel pattern | 05-03 | Avoids borrow issues; collect events then process |
| Import merge strategy | 05-03 | Adds new macros by name, skips duplicates |
| Hot-reload full re-register | 05-03 | Unregisters all then registers fresh for clean state |
| auto-launch crate for login items | 06-01 | Cross-platform, used by Tauri internally |
| MacOSLaunchMode::LaunchAgent | 06-01 | Creates plist in ~/Library/LaunchAgents/ |
| Use set_macos_launch_mode() API | 06-01 | Avoids deprecated set_use_launch_agent() |

### Pending Todos

None yet.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-01-16
Stopped at: Completed 06-01-PLAN.md
Resume file: None
