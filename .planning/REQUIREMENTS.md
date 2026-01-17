# Requirements: KeyBlast

**Defined:** 2026-01-16
**Updated:** 2026-01-17 (v2.0 requirements added)
**Core Value:** Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.

## v1 Requirements (Complete)

All v1.0 requirements delivered and shipped.

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

## v2 Requirements (Complete)

All v2.0 requirements delivered and shipped.

### UX Improvements

- [x] **UX-01**: User can search/filter macros by name in tray menu
- [x] **UX-02**: User can run macro by clicking menu item (not just hotkey)
- [x] **UX-03**: User can access file logs via "Open Logs..." menu item
- [x] **UX-04**: User's enabled/disabled state persists across restarts
- [x] **UX-05**: App has a unique, recognizable icon (tray and app)

### Async Execution

- [x] **ASYNC-01**: Macro execution runs off event loop thread (non-blocking)
- [x] **ASYNC-02**: User can stop a running macro via hotkey or menu
- [x] **ASYNC-03**: Tray menu stays responsive during long macro execution

### Expanded DSL

- [x] **DSL-01**: User can insert `{Delay 500}` to pause mid-macro (milliseconds)
- [x] **DSL-02**: User can use `{KeyDown Ctrl}` / `{KeyUp Ctrl}` for modifier combos
- [x] **DSL-03**: User can use `{Paste}` to paste clipboard contents
- [x] **DSL-04**: User can use `{{` / `}}` for literal brace characters

### Robustness

- [x] **ROBUST-01**: App validates config and detects duplicate names/hotkeys
- [x] **ROBUST-02**: Conflicts are surfaced in tray menu (not just println)
- [x] **ROBUST-03**: Macro delete uses stable IDs instead of names
- [x] **ROBUST-04**: Import merge correctly de-dupes within imported file
- [x] **ROBUST-05**: Windows config save works (fix fs::rename overwrite)

## v2.1 Requirements (Active)

### Windows Polish

- [x] **WIN-01**: Windows executable runs without console window
- [x] **WIN-02**: Windows executable displays embedded icon in Explorer/taskbar/Alt+Tab

### Error Visibility

- [ ] **ERR-01**: User receives tray notification when keystroke injection fails
- [ ] **ERR-02**: User receives tray notification when permission issue occurs

### Onboarding

- [ ] **ONBOARD-01**: Fresh config includes example macros demonstrating usage

### macOS Polish

- [ ] **MAC-01**: macOS app distributed as .app bundle with custom icon in Finder/Dock

## v3.0 Requirements (Planned)

*Specifics to be defined when starting v3.0 milestone.*

### Security

- [ ] **SEC-01**: User can encrypt sensitive macros with PIN/passkey access control

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
| GUI config editor | Tray menu + TOML file editing sufficient for v2 |

## Traceability

Which phases cover which requirements. Updated by create-roadmap.

### v1.0 (Complete)

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

### v2.0 (Complete)

| Requirement | Phase | Status |
|-------------|-------|--------|
| ASYNC-01 | Phase 7 | Complete |
| ASYNC-02 | Phase 7 | Complete |
| ASYNC-03 | Phase 7 | Complete |
| DSL-01 | Phase 8 | Complete |
| DSL-02 | Phase 8 | Complete |
| DSL-03 | Phase 8 | Complete |
| DSL-04 | Phase 8 | Complete |
| ROBUST-01 | Phase 9 | Complete |
| ROBUST-02 | Phase 9 | Complete |
| ROBUST-03 | Phase 9 | Complete |
| ROBUST-04 | Phase 9 | Complete |
| ROBUST-05 | Phase 9 | Complete |
| UX-01 | Phase 10 | Complete |
| UX-02 | Phase 10 | Complete |
| UX-03 | Phase 10 | Complete |
| UX-04 | Phase 10 | Complete |
| UX-05 | Phase 10 | Complete |

### v2.1 (Active)

| Requirement | Phase | Status |
|-------------|-------|--------|
| WIN-01 | Phase 11 | Complete |
| WIN-02 | Phase 11 | Complete |
| ERR-01 | Phase 12 | Pending |
| ERR-02 | Phase 12 | Pending |
| ONBOARD-01 | Phase 13 | Pending |
| MAC-01 | Phase 14 | Pending |

**Coverage:**
- v1 requirements: 20 total (Complete)
- v2 requirements: 17 total (Complete)
- v2.1 requirements: 6 total (2 complete, 4 pending)
- Unmapped: 0 ✓

---
*Requirements defined: 2026-01-16*
*Last updated: 2026-01-17 after adding MAC-01*
