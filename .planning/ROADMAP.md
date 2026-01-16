# Roadmap: KeyBlast

## Overview

KeyBlast delivers a lightweight hotkey-triggered keystroke injector in 6 phases. Starting with system tray presence, we progressively add hotkey detection, keystroke injection, persistent configuration, a tray-based config UI, and finally cross-platform polish with accessibility handling. Each phase delivers a complete, testable capability.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Foundation** - System tray presence with enable/disable toggle
- [x] **Phase 2: Global Hotkeys** - Hotkey registration that works in any application
- [x] **Phase 3: Keystroke Injection** - Type macros into the focused application
- [x] **Phase 4: Configuration** - Persistent macro storage in TOML
- [x] **Phase 5: Configuration UI** - Create/edit/delete macros via tray menu
- [ ] **Phase 6: Platform Polish** - macOS/Windows support, auto-start, visual feedback

## Phase Details

### Phase 1: Foundation
**Goal**: System tray presence with enable/disable toggle and quit
**Depends on**: Nothing (first phase)
**Requirements**: TRAY-01, TRAY-02
**Success Criteria** (what must be TRUE):
  1. App icon visible in system tray on both macOS and Windows
  2. Right-click menu shows Enable/Disable toggle and Quit option
  3. Toggle state persists visually (checked/unchecked)
**Research**: Unlikely (tray-icon + muda established patterns)
**Plans**: 1 plan

Plans:
- [x] 01-01: System tray with enable/disable toggle and quit

### Phase 2: Global Hotkeys
**Goal**: Hotkey registration that works in any application
**Depends on**: Phase 1
**Requirements**: HKEY-01, HKEY-02, HKEY-03
**Success Criteria** (what must be TRUE):
  1. User can register a hotkey that triggers from any app
  2. User sees warning when assigning conflicting hotkey
  3. User is shown available hotkey suggestions when creating macro
**Research**: Unlikely (global-hotkey well-documented)
**Plans**: 2 plans

Plans:
- [x] 02-01: Core hotkey infrastructure with winit integration
- [x] 02-02: Conflict detection and hotkey suggestions

### Phase 3: Keystroke Injection
**Goal**: Type macros into the focused application
**Depends on**: Phase 2
**Requirements**: INJT-01, INJT-02, INJT-03
**Success Criteria** (what must be TRUE):
  1. Triggered macro types text into the currently focused input
  2. Special keys (Enter, Tab, Escape, arrows) work in sequences
  3. User can configure per-macro keystroke delay
**Research**: Complete (03-RESEARCH.md)
**Research topics**: AXIsProcessTrusted() checking, enigo cross-platform behavior, modifier key state handling
**Plans**: 2 plans

Plans:
- [x] 03-01: Core injection infrastructure (permission, injector, macro parsing)
- [x] 03-02: Integration with event loop and end-to-end testing

### Phase 4: Configuration
**Goal**: Persistent macro storage in TOML format
**Depends on**: Phase 3
**Requirements**: CONF-01
**Success Criteria** (what must be TRUE):
  1. Macros survive app restart
  2. Config file is human-readable (TOML)
  3. Config loads automatically at startup
**Research**: Unlikely (serde + toml standard patterns)
**Plans**: 2 plans

Plans:
- [x] 04-01: Config data model and TOML file handling
- [x] 04-02: Wire config loading into app startup

### Phase 5: Configuration UI
**Goal**: User-friendly macro management via tray menu
**Depends on**: Phase 4
**Requirements**: CONF-02, CONF-03, CONF-04, CONF-05, CONF-06, ORGN-01
**Success Criteria** (what must be TRUE):
  1. User can create new macros from tray menu
  2. User can edit existing macros from tray menu
  3. User can delete macros from tray menu
  4. User can export macros to file
  5. User can import macros from file
  6. User can organize macros into groups/categories
**Research**: Unlikely (internal UI patterns)
**Plans**: 3 plans

Plans:
- [x] 05-01: Config layer enhancements (groups, export/import)
- [x] 05-02: Tray menu restructure with dynamic macro listing
- [x] 05-03: Action handlers + file watcher for hot-reload

### Phase 6: Platform Polish
**Goal**: Production-ready cross-platform support
**Depends on**: Phase 5
**Requirements**: PLAT-01, PLAT-02, PLAT-03, PLAT-04, TRAY-03
**Success Criteria** (what must be TRUE):
  1. App works correctly on macOS
  2. App works correctly on Windows
  3. User can enable auto-start at login
  4. macOS user is guided through Accessibility permission
  5. Tray icon flashes when macro triggers (visual feedback)
**Research**: Likely (code signing, notarization, auto-launch mechanisms)
**Research topics**: auto-launch crate vs custom, Apple notarization requirements, Windows signing
**Plans**: TBD

Plans:
- [ ] 06-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4 → 5 → 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation | 1/1 | Complete | 2026-01-16 |
| 2. Global Hotkeys | 2/2 | Complete | 2026-01-16 |
| 3. Keystroke Injection | 2/2 | Complete | 2026-01-16 |
| 4. Configuration | 2/2 | Complete | 2026-01-16 |
| 5. Configuration UI | 3/3 | Complete | 2026-01-16 |
| 6. Platform Polish | 0/TBD | Not started | - |
