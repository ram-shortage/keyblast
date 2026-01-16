---
phase: 5-configuration-ui
plan: 03
subsystem: ui
tags: [notify, file-watcher, hot-reload, rfd, file-dialog, tray-menu]

# Dependency graph
requires:
  - phase: 5-01
    provides: config export/import functions in config.rs
  - phase: 5-02
    provides: menu structure with edit/export/import/delete items
provides:
  - Config hot-reload via notify file watcher
  - Edit Config File opens system editor
  - Delete Macro with hotkey unregistration
  - Export Macros with native save dialog
  - Import Macros with merge and hotkey registration
affects: [6-testing]

# Tech tracking
tech-stack:
  added: [notify v6]
  patterns: [file-watcher-with-channel, non-blocking-event-polling]

key-files:
  created: []
  modified: [Cargo.toml, src/main.rs]

key-decisions:
  - "Used mpsc channel for file watcher events to avoid borrow issues"
  - "Import merges by name - existing macros skipped, new ones added"
  - "Hot-reload unregisters all hotkeys then re-registers from fresh config"

patterns-established:
  - "File watcher pattern: RecommendedWatcher -> mpsc channel -> non-blocking try_recv in event loop"
  - "Platform-specific commands via #[cfg] for opening files (open/xdg-open/cmd start)"

# Metrics
duration: 8min
completed: 2026-01-16
---

# Phase 5 Plan 3: Menu Action Handlers Summary

**Config hot-reload via notify, platform-specific edit command, delete with unregister, export/import with native file dialogs**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-16T10:05:00Z
- **Completed:** 2026-01-16T10:13:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments

- Config hot-reload: file watcher detects changes, unregisters old hotkeys, registers new ones automatically
- Edit Config File opens config.toml in system default editor (macOS: open, Windows: cmd start, Linux: xdg-open)
- Delete Macro removes from config, unregisters hotkey, saves, and rebuilds menu
- Export Macros shows native save dialog with .toml filter
- Import Macros shows native open dialog, merges new macros, registers hotkeys, saves config

## Task Commits

Each task was committed atomically:

1. **Task 1: Add notify crate and implement file watcher for config hot-reload** - `8cdae79` (feat)
2. **Task 2: Implement Edit Config File and Delete Macro actions** - `284dd91` (feat)
3. **Task 3: Implement Export and Import actions** - `d642a2c` (feat)

## Files Created/Modified

- `Cargo.toml` - Added notify = "6" dependency
- `src/main.rs` - Added file watcher fields, setup/check/reload methods, menu action handlers

## Decisions Made

- **mpsc channel for watcher events:** Avoids borrow checker issues by collecting events before mutation
- **Import merge strategy:** Adds new macros by name, skips duplicates, logs skipped names
- **Hot-reload approach:** Full unregister/re-register cycle ensures clean state after external edits
- **Platform-specific open commands:** Used #[cfg] blocks for clean cross-platform support

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed borrow checker issue in check_config_changes**
- **Found during:** Task 1 (file watcher implementation)
- **Issue:** Calling self.reload_config() while holding immutable borrow of self.config_change_rx
- **Fix:** Collected should_reload flag first, then called reload_config() outside borrow scope
- **Files modified:** src/main.rs
- **Verification:** cargo build succeeds
- **Committed in:** 8cdae79 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minor borrow checker fix, pattern from plan was sound but needed adjustment for Rust ownership rules.

## Issues Encountered

None - all functionality implemented as specified.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All CONF requirements complete (CONF-01 through CONF-06)
- Configuration UI phase complete
- Ready for Phase 6: Testing & Documentation

---
*Phase: 5-configuration-ui*
*Completed: 2026-01-16*
