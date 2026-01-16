---
phase: 5-configuration-ui
plan: 02
subsystem: ui
tags: [tray-menu, muda, grouping, submenus]

# Dependency graph
requires:
  - phase: 5-01
    provides: config group field and export/import functions
  - phase: 4-02
    provides: config loading with MacroDefinition
provides:
  - Dynamic tray menu showing macros grouped by category
  - MenuIds with edit_config, export_macros, import_macros, delete_macro_ids fields
  - rebuild_menu method for config changes
affects: [05-03 action handlers, future macro management]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Grouped submenu structure with sorting
    - HashMap for delete menu item tracking
    - Menu rebuild pattern for dynamic updates

key-files:
  created: []
  modified:
    - src/tray.rs
    - src/main.rs

key-decisions:
  - "Groups sorted alphabetically with Ungrouped last"
  - "Each macro displayed as submenu with Delete action"
  - "Menu rebuilding via tray_icon.set_menu()"

patterns-established:
  - "Pattern: build_menu(enabled, macros) creates full dynamic menu"
  - "Pattern: rebuild_menu() refreshes tray after config changes"

# Metrics
duration: 7min
completed: 2026-01-16
---

# Phase 5 Plan 02: Menu Restructuring Summary

**Dynamic tray menu with grouped macros, management actions (Edit/Export/Import), and rebuild capability**

## Performance

- **Duration:** 7 min
- **Started:** 2026-01-16T12:00:00Z
- **Completed:** 2026-01-16T12:07:00Z
- **Tasks:** 3
- **Files modified:** 2

## Accomplishments
- Restructured MenuIds to include all management action IDs
- build_menu now accepts macros slice and creates grouped submenu structure
- Groups sorted alphabetically with "Ungrouped" always last
- Each macro shows name (hotkey) with Delete submenu action
- rebuild_menu method available for config changes after import/delete

## Task Commits

Each task was committed atomically:

1. **Task 1: Redesign MenuIds and build_menu for dynamic content** - `7670619` (feat)
2. **Task 2: Update main.rs to pass macros to build_menu** - `33f08c3` (feat)
3. **Task 3: Add menu rebuild capability** - `38e8667` (feat)

**Plan metadata:** [pending] (docs: complete plan)

## Files Created/Modified
- `src/tray.rs` - Dynamic menu building with grouped macros and management actions
- `src/main.rs` - Updated MenuIds initialization, reordered resumed() to load config before menu, added rebuild_menu()

## Decisions Made
- Groups sorted alphabetically with "Ungrouped" kept at the end for consistency
- Each macro uses a submenu with only "Delete" action (Edit is via config file)
- Menu rebuilt by calling set_menu() on existing TrayIcon

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Menu structure complete with all action placeholders
- Plan 05-03 can implement action handlers for:
  - Edit Config File (open in editor)
  - Export Macros (file picker)
  - Import Macros (file picker + merge)
  - Delete macro (remove from config + rebuild)
- rebuild_menu() ready to be called after config changes

---
*Phase: 5-configuration-ui*
*Completed: 2026-01-16*
