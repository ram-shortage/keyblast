# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-01-17)

**Core value:** Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.
**Current focus:** v2.1 Windows Polish

## Current Position

Phase: 11 of 13 (Windows Executable)
Plan: Not started
Status: Ready to plan
Last activity: 2026-01-17 — v2.1 roadmap created

Progress: ██████████░░░░░░░░░░ 77% (10/13 phases complete)

## Performance Metrics

**v1.0 Velocity:**
- Total plans completed: 13
- Average duration: ~7.5 min
- Total execution time: ~1.6 hours

**v2.0 Velocity:**
- Total plans completed: 9
- Phases: 4
- All phases complete

## Accumulated Context

### Decisions

| Decision | Phase | Rationale |
|----------|-------|-----------|
| Unbounded crossbeam channel | 07-01 | Avoid deadlock scenarios with bounded channels |
| 50ms stop flag check interval | 07-01 | Balance responsiveness and efficiency during delays |
| ExecutionCommand enum pattern | 07-01 | Clean worker-to-main communication (Inject/Complete/Cancelled) |
| Sync fast path for short macros | 07-02 | Avoid async overhead for <= 10 segments with no delay |
| Collect-then-process pattern | 07-02 | Satisfy Rust borrow checker when processing commands |
| Placeholder execution handlers | 08-01 | Allow compilation before 08-02 implements full execution |
| Brace escapes merge into text | 08-01 | Efficiency - single Text segment vs multiple |
| Paste graceful degradation | 08-02 | Log warning but don't fail if clipboard inaccessible |
| has_delay fast-path check | 08-02 | Ensure macros with {Delay N} use async execution for responsive tray |
| First occurrence wins dedupe | 09-01 | Simple deterministic rule for duplicate macro names |
| Windows remove-then-rename | 09-01 | Preserve atomic write intent on Windows platform |
| validate_config returns warnings | 09-01 | Caller decides action on validation warnings |
| serde default = Uuid::new_v4 | 09-02 | Auto-generate UUIDs for existing configs on deserialize |
| Warnings as disabled menu items | 09-02 | Informational display in tray, not clickable actions |
| Flat list over grouped for Run Macro | 10-03 | Native tray menus cannot support search; flat alphabetized is easiest to browse |
| AppSettings separate struct | 10-02 | Application-wide preferences separate from macros, backward-compatible via serde defaults |
| Immediate save on toggle | 10-02 | Prevents data loss on crash, no wait-for-quit semantics |
| 7-day log retention | 10-01 | Reasonable default for troubleshooting without accumulating excessive files |
| Daily log rotation | 10-01 | Matches common patterns, easy to find logs by date |
| Graceful logging fallback | 10-01 | If logging setup fails, app continues without file logging |
| Lightning bolt icon design | 10-04 | Suggests "blast"/speed, recognizable at small tray size |
| Color inversion for flash state | 10-04 | Yellow-on-dark vs dark-on-yellow provides clear visual feedback |

### Pending Todos

None.

### Blockers/Concerns

None.

## Session Continuity

Last session: 2026-01-17
Stopped at: Milestone v2.1 initialized
Resume file: None
