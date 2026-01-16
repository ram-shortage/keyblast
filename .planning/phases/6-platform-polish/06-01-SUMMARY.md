---
phase: 06-platform-polish
plan: 01
subsystem: ui
tags: [auto-launch, tray-menu, login-item, macos, windows]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: System tray infrastructure and menu system
  - phase: 05-configuration-ui
    provides: Menu event handling patterns
provides:
  - Auto-start at login functionality
  - Cross-platform login item management (macOS LaunchAgent, Windows registry)
  - Start at Login toggle in tray menu
affects: [distribution, installer]

# Tech tracking
tech-stack:
  added: [auto-launch 0.6]
  patterns: [CheckMenuItem for toggle state, platform-specific cfg blocks]

key-files:
  created: [src/autostart.rs]
  modified: [Cargo.toml, src/main.rs, src/tray.rs]

key-decisions:
  - "Use auto-launch crate with MacOSLaunchMode::LaunchAgent for macOS"
  - "Query auto-start state at menu build time for accurate checkbox display"
  - "Use set_macos_launch_mode() API (not deprecated set_use_launch_agent())"

patterns-established:
  - "Platform-specific imports: #[cfg(target_os = \"macos\")] use ..."
  - "Auto-start toggle follows same CheckMenuItem pattern as Enable toggle"

# Metrics
duration: 7min
completed: 2026-01-16
---

# Phase 6 Plan 1: Auto-Start at Login Summary

**Cross-platform auto-start at login using auto-launch crate with LaunchAgent (macOS) and registry (Windows) backends**

## Performance

- **Duration:** 7 min
- **Started:** 2026-01-16T11:00:00Z
- **Completed:** 2026-01-16T11:07:00Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Auto-start at login functionality via auto-launch crate
- "Start at Login" checkbox in tray menu
- Cross-platform support (macOS LaunchAgent, Windows registry)
- Checkbox state accurately reflects actual system state

## Task Commits

Each task was committed atomically:

1. **Task 1: Add auto-launch crate and create autostart module** - `02c2ef1` (feat)
2. **Task 2: Add auto-start toggle to tray menu** - `f80a879` (feat)

## Files Created/Modified
- `src/autostart.rs` - Auto-launch management functions (create_auto_launch, is_auto_start_enabled, set_auto_start)
- `Cargo.toml` - Added auto-launch 0.6 dependency
- `src/tray.rs` - Added auto_start MenuId and "Start at Login" CheckMenuItem
- `src/main.rs` - Added autostart module and auto_start event handler

## Decisions Made
- Used `auto-launch` crate (0.6) - cross-platform, used by Tauri internally
- MacOSLaunchMode::LaunchAgent for macOS - creates plist in ~/Library/LaunchAgents/
- Windows uses registry key in HKCU\Software\Microsoft\Windows\CurrentVersion\Run
- Query actual auto-start state at menu build time for accurate initial checkbox state
- Used `set_macos_launch_mode()` instead of deprecated `set_use_launch_agent()`

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed deprecated API usage**
- **Found during:** Task 2 verification
- **Issue:** `set_use_launch_agent(true)` is deprecated, produces compiler warning
- **Fix:** Changed to `set_macos_launch_mode(MacOSLaunchMode::LaunchAgent)`
- **Files modified:** src/autostart.rs
- **Verification:** Build passes with no deprecation warnings
- **Committed in:** f80a879 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug - deprecated API)
**Impact on plan:** Minor API update for cleaner builds. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Auto-start functionality complete for PLAT-03
- Ready for additional platform polish features (visual feedback, permission UX)
- No blockers

---
*Phase: 06-platform-polish*
*Completed: 2026-01-16*
