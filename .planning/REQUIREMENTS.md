# Requirements: KeyBlast

**Defined:** 2026-01-16
**Core Value:** Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### System Tray

- [x] **TRAY-01**: User can see app in system tray with menu
- [x] **TRAY-02**: User can enable/disable all macros via toggle
- [x] **TRAY-03**: User sees visual feedback when macro triggers (tray icon flash)

### Global Hotkeys

- [x] **HKEY-01**: User can register global hotkeys that work in any application
- [x] **HKEY-02**: User is warned when assigning a hotkey already in use
- [x] **HKEY-03**: User is suggested available hotkey combinations when creating macros

### Keystroke Injection

- [x] **INJT-01**: User's macro text is typed into the focused application
- [x] **INJT-02**: User can include special keys (Enter, Tab, Escape, arrows) in macros
- [x] **INJT-03**: User can set keystroke delay per macro (instant to slow)

### Configuration

- [x] **CONF-01**: User's macros persist across app restarts
- [x] **CONF-02**: User can create new macros via tray menu
- [x] **CONF-03**: User can edit existing macros via tray menu
- [x] **CONF-04**: User can delete macros via tray menu
- [x] **CONF-05**: User can export all macros to a file
- [x] **CONF-06**: User can import macros from a file

### Organization

- [x] **ORGN-01**: User can organize macros into groups/categories

### Platform

- [x] **PLAT-01**: App works on macOS
- [x] **PLAT-02**: App works on Windows
- [x] **PLAT-03**: User can enable auto-start at login
- [x] **PLAT-04**: macOS user is guided through Accessibility permission grant

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Configuration

- **CONF-07**: Config auto-reloads when edited externally

### Organization

- **ORGN-02**: User can search/filter macros by name

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| Abbreviation triggers | Different product category (keystroke monitoring vs hotkey listening) |
| Modifier combos as output (Ctrl+C, Cmd+V) | Platform edge cases, security concerns |
| Mouse automation | Keyboard-only focus |
| Macro recording | Adds complexity, manual config only |
| Scripting/conditionals | Simple sequence playback only |
| Cloud sync | Offline-only by design |
| Rich text/images | Keystroke injection is plain text |

## Traceability

Which phases cover which requirements. Updated by create-roadmap.

| Requirement | Phase | Status |
|-------------|-------|--------|
| TRAY-01 | Phase 1 | Complete |
| TRAY-02 | Phase 1 | Complete |
| TRAY-03 | Phase 6 | Complete |
| HKEY-01 | Phase 2 | Complete |
| HKEY-02 | Phase 2 | Complete |
| HKEY-03 | Phase 2 | Complete |
| INJT-01 | Phase 3 | Complete |
| INJT-02 | Phase 3 | Complete |
| INJT-03 | Phase 3 | Complete |
| CONF-01 | Phase 4 | Complete |
| CONF-02 | Phase 5 | Complete |
| CONF-03 | Phase 5 | Complete |
| CONF-04 | Phase 5 | Complete |
| CONF-05 | Phase 5 | Complete |
| CONF-06 | Phase 5 | Complete |
| ORGN-01 | Phase 5 | Complete |
| PLAT-01 | Phase 6 | Complete |
| PLAT-02 | Phase 6 | Complete |
| PLAT-03 | Phase 6 | Complete |
| PLAT-04 | Phase 6 | Complete |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 20 ✓
- Unmapped: 0

---
*Requirements defined: 2026-01-16*
*Last updated: 2026-01-16 after Phase 6 completion*
