# Requirements: KeyBlast

**Defined:** 2026-01-16
**Core Value:** Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### System Tray

- [ ] **TRAY-01**: User can see app in system tray with menu
- [ ] **TRAY-02**: User can enable/disable all macros via toggle
- [ ] **TRAY-03**: User sees visual feedback when macro triggers (tray icon flash)

### Global Hotkeys

- [ ] **HKEY-01**: User can register global hotkeys that work in any application
- [ ] **HKEY-02**: User is warned when assigning a hotkey already in use
- [ ] **HKEY-03**: User is suggested available hotkey combinations when creating macros

### Keystroke Injection

- [ ] **INJT-01**: User's macro text is typed into the focused application
- [ ] **INJT-02**: User can include special keys (Enter, Tab, Escape, arrows) in macros
- [ ] **INJT-03**: User can set keystroke delay per macro (instant to slow)

### Configuration

- [ ] **CONF-01**: User's macros persist across app restarts
- [ ] **CONF-02**: User can create new macros via tray menu
- [ ] **CONF-03**: User can edit existing macros via tray menu
- [ ] **CONF-04**: User can delete macros via tray menu
- [ ] **CONF-05**: User can export all macros to a file
- [ ] **CONF-06**: User can import macros from a file

### Organization

- [ ] **ORGN-01**: User can organize macros into groups/categories

### Platform

- [ ] **PLAT-01**: App works on macOS
- [ ] **PLAT-02**: App works on Windows
- [ ] **PLAT-03**: User can enable auto-start at login
- [ ] **PLAT-04**: macOS user is guided through Accessibility permission grant

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
| TRAY-01 | Phase 1 | Pending |
| TRAY-02 | Phase 1 | Pending |
| TRAY-03 | Phase 6 | Pending |
| HKEY-01 | Phase 2 | Pending |
| HKEY-02 | Phase 2 | Pending |
| HKEY-03 | Phase 2 | Pending |
| INJT-01 | Phase 3 | Pending |
| INJT-02 | Phase 3 | Pending |
| INJT-03 | Phase 3 | Pending |
| CONF-01 | Phase 4 | Pending |
| CONF-02 | Phase 5 | Pending |
| CONF-03 | Phase 5 | Pending |
| CONF-04 | Phase 5 | Pending |
| CONF-05 | Phase 5 | Pending |
| CONF-06 | Phase 5 | Pending |
| ORGN-01 | Phase 5 | Pending |
| PLAT-01 | Phase 6 | Pending |
| PLAT-02 | Phase 6 | Pending |
| PLAT-03 | Phase 6 | Pending |
| PLAT-04 | Phase 6 | Pending |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 20 ✓
- Unmapped: 0

---
*Requirements defined: 2026-01-16*
*Last updated: 2026-01-16 after roadmap creation*
