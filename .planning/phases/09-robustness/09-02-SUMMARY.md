---
phase: 09-robustness
plan: 02
subsystem: config
tags: [uuid, validation, tray, serde]

# Dependency graph
requires:
  - phase: 09-01
    provides: ValidationWarning enum and validate_config function
provides:
  - Stable UUID identifiers for macros
  - Warnings submenu in tray showing config validation issues
  - UUID-based delete (works with duplicate names)
affects: [10-ux-polish]

# Tech tracking
tech-stack:
  added: [uuid]
  patterns: [UUID-based identity, warnings surfacing in UI]

key-files:
  created: []
  modified: [src/config.rs, src/tray.rs, src/main.rs, Cargo.toml]

key-decisions:
  - "serde default = Uuid::new_v4 for auto-generation on deserialize"
  - "Warnings displayed in disabled menu items (informational only)"

patterns-established:
  - "UUID identity: Macros identified by stable UUID, not name"
  - "Warnings surfacing: Validation warnings shown in tray submenu"

# Metrics
duration: 8min
completed: 2026-01-17
---

# Phase 9 Plan 2: UUID and Warnings UI Summary

**Stable UUID identifiers for macros with validation warnings surfaced in tray menu**

## Performance

- **Duration:** 8 min
- **Started:** 2026-01-17
- **Completed:** 2026-01-17
- **Tasks:** 3
- **Files modified:** 4

## Accomplishments
- MacroDefinition now has stable UUID that persists across saves and auto-generates on load
- Delete operations use UUID instead of name (works correctly with duplicate macro names)
- Validation warnings appear in tray menu as "Warnings (N)" submenu
- Warnings update automatically when config is reloaded or macros deleted

## Task Commits

Each task was committed atomically:

1. **Task 1: Add stable UUID to MacroDefinition** - `042b5b4` (feat)
2. **Task 2: Update tray menu to use UUIDs and show warnings** - `9190546` (feat)
3. **Task 3: Wire validation and UUID delete in main.rs** - `62d6f44` (feat)

## Files Created/Modified
- `Cargo.toml` - Added uuid dependency with v4 and serde features
- `src/config.rs` - Added id: Uuid field to MacroDefinition with serde default
- `src/tray.rs` - Changed delete_macro_ids to use Uuid, added Warnings submenu
- `src/main.rs` - Added config_warnings field, validation at load/reload, UUID-based delete

## Decisions Made
- Used `#[serde(default = "Uuid::new_v4")]` to auto-generate UUIDs for existing configs without id field
- Warning menu items are disabled (informational display, not clickable actions)
- Warnings re-validated after each delete operation to keep submenu current

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Robustness features (validation warnings, atomic save, UUID identity) complete
- Ready for UX polish phase
- No blockers

---
*Phase: 09-robustness*
*Completed: 2026-01-17*
