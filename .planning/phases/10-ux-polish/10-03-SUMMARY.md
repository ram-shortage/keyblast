---
phase: 10-ux-polish
plan: 03
outcome: success
duration: ~2.5 min
completed: 2026-01-17

artifacts:
  created: []
  modified:
    - src/tray.rs
    - src/main.rs

verification:
  criteria_met: 6/6
  tests_passing: true
  build_status: success

dependencies:
  requires: []
  provides:
    - run-macro-submenu
    - click-to-run-functionality
  affects: []

tech-stack:
  added: []
  patterns:
    - flat-alphabetized-menu
    - uuid-based-lookup

decisions:
  - id: flat-list-over-grouped
    choice: Flat alphabetized list
    rationale: Research determined native tray menus cannot support search; flat list is easiest to browse

key-files:
  created: []
  modified:
    - path: src/tray.rs
      changes: Added run_macro_ids field and Run Macro submenu builder
    - path: src/main.rs
      changes: Added run_macro_ids event handler in about_to_wait
---

# Phase 10 Plan 03: Run Macro Submenu Summary

Run Macro submenu with click-to-run functionality for all macros.

## What Was Built

Added "Run Macro" submenu to the tray menu that allows users to run any macro by clicking on its name, without needing to remember hotkeys.

**Menu structure:**
```
[x] Enable
Stop Macro
---
Run Macro >
  Macro A (ctrl+a)
  Macro B (ctrl+b)
  ...
Macros >
  [management/delete menu]
```

## Key Implementation Details

### Tray Menu (src/tray.rs)

1. **New field in MenuIds:**
   - `run_macro_ids: HashMap<muda::MenuId, Uuid>` - maps menu item ID to macro UUID

2. **Run Macro submenu builder:**
   - Creates flat alphabetized list of all macros
   - Sorts case-insensitively by name
   - Label format: `{name} ({hotkey})`
   - Stores mapping for click handling

### Event Handler (src/main.rs)

1. **Run macro event processing:**
   - Checks run_macro_ids BEFORE delete_macro_ids
   - Looks up macro by UUID from config
   - Respects enabled state (ignores if disabled)
   - Respects running state (ignores if already executing)

2. **Execution path:**
   - Uses same fast/async logic as hotkey trigger
   - Fast path: <= 10 segments, no delay_ms, no {Delay} commands
   - Async path: longer macros or those with delays
   - Icon flashes on completion

## Verification Results

| Criteria | Status |
|----------|--------|
| Run Macro submenu appears in tray | Pass |
| Macros listed in alphabetical order | Pass |
| Format shows "Name (hotkey)" | Pass |
| Click executes macro immediately | Pass |
| Disabled state respected | Pass |
| Running state respected | Pass |

## Commits

1. `947bc7a` - feat(10-03): add Run Macro submenu to tray menu
2. `7d67ddf` - feat(10-03): handle run macro menu events

## Deviations from Plan

None - plan executed exactly as written.

## Success Criteria Met

- UX-01 satisfied: Flat alphabetized submenu for macro browsing
- UX-02 satisfied: Click-to-run works as alternative to hotkeys
- Same execution behavior as hotkey trigger (flash, async for long macros)
