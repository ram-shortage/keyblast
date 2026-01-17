---
phase: 12-error-notifications
plan: 01
subsystem: ui
tags: [notify-rust, toast-notifications, error-handling, cross-platform]

# Dependency graph
requires:
  - phase: 10-platform-polish
    provides: logging infrastructure, tray icon system
provides:
  - User-visible error notifications via OS toast system
  - Cross-platform notification abstraction (macOS/Windows/Linux)
  - Debounced notification system to prevent spam
  - Platform-specific permission error messages
affects: [13-diagnostics, 14-macos-bundle]

# Tech tracking
tech-stack:
  added: [notify-rust v4]
  patterns: [debounced notifications, severity-based timeout]

key-files:
  created: [src/notification.rs]
  modified: [Cargo.toml, src/main.rs, src/permission.rs]

key-decisions:
  - "3s debounce interval prevents notification spam on rapid failures"
  - "Permission errors bypass debouncing (always critical)"
  - "5s auto-dismiss for injection failures, persistent for permission issues"

patterns-established:
  - "Notification severity enum controls timeout behavior"
  - "show_error supplements logging, does not replace it"

# Metrics
duration: 4min
completed: 2026-01-17
---

# Phase 12 Plan 01: Error Notifications Summary

**Cross-platform toast notifications for injection failures and permission issues using notify-rust with 3s debouncing**

## Performance

- **Duration:** 4 min
- **Started:** 2026-01-17T12:00:00Z
- **Completed:** 2026-01-17T12:04:00Z
- **Tasks:** 3 (2 auto, 1 checkpoint skipped)
- **Files modified:** 4

## Accomplishments
- Created notification module with show_error() and NotificationSeverity enum
- Integrated notifications at all 6 injection failure points across main.rs and permission.rs
- Implemented debouncing (3s interval) to prevent notification spam on rapid failures
- Platform-specific permission error messages guide users to correct settings

## Task Commits

Each task was committed atomically:

1. **Task 1: Create notification module with notify-rust** - `cd05247` (feat)
2. **Task 2: Integrate notifications at error sites** - `751bb78` (feat)
3. **Task 3: Human verification checkpoint** - skipped (skip_checkpoints: true)

**Plan metadata:** (pending)

## Files Created/Modified
- `Cargo.toml` - Added notify-rust v4 dependency
- `src/notification.rs` - New notification abstraction module with show_error() and NotificationSeverity
- `src/main.rs` - Added notification calls at 5 injection failure sites
- `src/permission.rs` - Added notification call on macOS accessibility permission denial

## Decisions Made
- 3-second debounce interval for non-permission errors prevents notification spam when multiple failures occur rapidly
- Permission errors bypass debouncing since they are critical and require user action
- 5-second auto-dismiss for injection failures (informational), persistent for permission issues (action required)
- Notifications supplement existing eprintln!/error! logging rather than replacing it

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- Error notification system complete and integrated
- Ready for Phase 13 (Diagnostics) which may use notifications for health check results
- Ready for Phase 14 (macOS App Bundle) which will need proper notification entitlements

---
*Phase: 12-error-notifications*
*Completed: 2026-01-17*
