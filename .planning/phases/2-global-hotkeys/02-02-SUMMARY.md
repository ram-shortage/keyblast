---
phase: 02-global-hotkeys
plan: 02
subsystem: hotkeys
tags: [global-hotkey, conflict-detection, hotkey-suggestions, keyboard-shortcuts]

# Dependency graph
requires:
  - phase: 02-global-hotkeys
    plan: 01
    provides: HotkeyManager with register() method, global-hotkey crate integration
provides:
  - RegisterResult enum for typed conflict information
  - try_register() method with conflict detection
  - suggest_available() method for available hotkey discovery
  - hotkey_display_string() helper for formatting
affects: [03-macro-storage, 04-config-ui]

# Tech tracking
tech-stack:
  added: []
  patterns: [Error-based conflict detection, register/unregister probing for availability]

key-files:
  created: []
  modified: [src/hotkey.rs, src/main.rs]

key-decisions:
  - "RegisterResult enum over simple Result: Distinguishes internal vs external conflicts"
  - "Pre-check bindings before manager.register(): Catches internal conflicts without OS call"
  - "Tier 1/2 candidate hotkeys: Ctrl+Shift and Ctrl+Alt combinations rarely conflict"

patterns-established:
  - "Conflict classification: ConflictInternal (KeyBlast), ConflictExternal (OS/app)"
  - "Availability probing: Register then immediately unregister to test"
  - "Candidate ordering: Most available first (Ctrl+Shift+K,M,J,L...)"

# Metrics
duration: 6min
completed: 2026-01-16
---

# Phase 02 Plan 02: Conflict Detection and Suggestions Summary

**RegisterResult enum with internal/external conflict detection, and suggest_available() probing 16 candidate hotkeys to return available combinations**

## Performance

- **Duration:** 6 min
- **Started:** 2026-01-16T11:00:00Z
- **Completed:** 2026-01-16T11:06:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- RegisterResult enum distinguishes Success, ConflictInternal, ConflictExternal, and Error
- try_register() returns typed conflict information for user feedback (HKEY-02)
- suggest_available(n) returns n available hotkey combinations (HKEY-03)
- Suggestions correctly skip already-registered hotkeys (Ctrl+Shift+K excluded from suggestions)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add RegisterResult enum with conflict detection** - `6d916f4` (feat)
2. **Task 2: Implement hotkey suggestion generation** - `17bfcf4` (feat)
3. **Task 3: Test conflict detection and suggestions in main** - `4e6c486` (feat)

## Files Created/Modified
- `src/hotkey.rs` - Added RegisterResult enum, try_register(), unregister(), suggest_available(), candidate_hotkeys(), hotkey_display_string()
- `src/main.rs` - Added conflict detection test and suggestion generation demonstration

## Decisions Made
- **RegisterResult enum vs Result<T, E>:** Enum provides richer conflict classification (internal vs external) for better user messages
- **Pre-check internal bindings:** Check HashMap before calling manager.register() to classify internal conflicts without OS call
- **Candidate hotkey selection:** 16 candidates in Ctrl+Shift and Ctrl+Alt combinations chosen for low system conflict probability

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None - all tasks completed successfully.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- All Phase 2 requirements satisfied (HKEY-01, HKEY-02, HKEY-03)
- HotkeyManager ready for macro storage integration
- Conflict detection enables clear user feedback when hotkeys unavailable
- Suggestion system provides alternatives when user's preferred hotkey conflicts

---
*Phase: 02-global-hotkeys*
*Completed: 2026-01-16*
