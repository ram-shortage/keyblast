# Roadmap: KeyBlast

## Milestones

- ✅ **v1.0 MVP** - Phases 1-6 (shipped 2026-01-16)
- ✅ **v2.0 Quality & Power** - Phases 7-10 (shipped 2026-01-17)
- 🚧 **v2.1 Platform Polish** - Phases 11-14 (in progress)
- 📋 **v3.0 Security** - Encrypted macros with PIN/passkey (planned)

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

<details>
<summary>✅ v1.0 MVP (Phases 1-6) - SHIPPED 2026-01-16</summary>

- [x] **Phase 1: Foundation** - System tray presence with enable/disable toggle
- [x] **Phase 2: Global Hotkeys** - Hotkey registration that works in any application
- [x] **Phase 3: Keystroke Injection** - Type macros into the focused application
- [x] **Phase 4: Configuration** - Persistent macro storage in TOML
- [x] **Phase 5: Configuration UI** - Create/edit/delete macros via tray menu
- [x] **Phase 6: Platform Polish** - macOS/Windows support, auto-start, visual feedback

</details>

<details>
<summary>✅ v2.0 Quality & Power (Phases 7-10) - SHIPPED 2026-01-17</summary>

- [x] **Phase 7: Async Execution** - Non-blocking macro execution with stop capability
- [x] **Phase 8: Expanded DSL** - New macro syntax: Delay, KeyDown/KeyUp, Paste, brace escapes
- [x] **Phase 9: Robustness** - Config validation, conflict UI, bug fixes
- [x] **Phase 10: UX Polish** - Search, click-to-run, logging, persist state, custom icon

</details>

### 🚧 v2.1 Platform Polish (In Progress)

- [x] **Phase 11: Windows Executable** - Console suppression and embedded .exe icon
- [ ] **Phase 12: Error Notifications** - Tray alerts for injection failures and permission issues
- [ ] **Phase 13: Onboarding Defaults** - Example macros in default config
- [ ] **Phase 14: macOS App Bundle** - .app bundle with custom icon in Finder/Dock

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
**Research**: Complete (06-RESEARCH.md)
**Research topics**: auto-launch crate, tray icon flash mechanism, Accessibility permission UX
**Plans**: 2 plans

Plans:
- [x] 06-01: Auto-start at login functionality (PLAT-03)
- [x] 06-02: Accessibility permission UX and tray icon flash (PLAT-04, TRAY-03)

</details>

---

## v2.0 Phase Details

### Phase 7: Async Execution
**Goal**: Non-blocking macro execution with stop capability
**Depends on**: Phase 6
**Requirements**: ASYNC-01, ASYNC-02, ASYNC-03
**Success Criteria** (what must be TRUE):
  1. Long macros don't freeze the tray menu
  2. User can stop a running macro mid-execution
  3. Macro execution happens in background thread
**Research**: Complete (07-RESEARCH.md)
**Research topics**: Rust async vs threads for keystroke injection, cancellation tokens, cross-thread communication
**Plans**: 2 plans

Plans:
- [x] 07-01: Async execution infrastructure (worker thread, channels, stop flag)
- [x] 07-02: Wire async execution into application (event loop, stop hotkey, stop menu)

### Phase 8: Expanded DSL
**Goal**: New macro syntax features for advanced sequences
**Depends on**: Phase 7
**Requirements**: DSL-01, DSL-02, DSL-03, DSL-04
**Success Criteria** (what must be TRUE):
  1. User can pause mid-macro with `{Delay 500}`
  2. User can press/release modifiers with `{KeyDown Ctrl}` / `{KeyUp Ctrl}`
  3. User can paste clipboard with `{Paste}`
  4. User can type literal braces with `{{` and `}}`
**Research**: Complete (08-RESEARCH.md)
**Research topics**: Cross-platform clipboard crates, modifier key state management, DSL parser extension
**Plans**: 2 plans

Plans:
- [x] 08-01: Parser extension (Delay, KeyDown/KeyUp, Paste, brace escapes)
- [x] 08-02: Execution integration (clipboard, fast-path update)

### Phase 9: Robustness
**Goal**: Config validation, conflict surfacing, and bug fixes
**Depends on**: Phase 7 (needs stable IDs before async)
**Requirements**: ROBUST-01, ROBUST-02, ROBUST-03, ROBUST-04, ROBUST-05
**Success Criteria** (what must be TRUE):
  1. App warns on duplicate macro names at config load
  2. Hotkey conflicts shown in tray menu (not just console)
  3. Macro delete works reliably via stable IDs
  4. Importing macros doesn't create duplicates
  5. Config saves correctly on Windows
**Research**: Skipped (internal patterns, bug fixes)
**Plans**: 2 plans

Plans:
- [x] 09-01: Config validation, Windows save fix, import de-dupe
- [x] 09-02: Stable UUIDs for macros, warnings UI in tray menu

### Phase 10: UX Polish
**Goal**: User-facing improvements for power users
**Depends on**: Phase 9
**Requirements**: UX-01, UX-02, UX-03, UX-04, UX-05
**Success Criteria** (what must be TRUE):
  1. User can search/filter macros by name in tray menu
  2. User can execute macro by clicking menu item
  3. User can open log files from tray menu
  4. Enabled/disabled state survives app restart
  5. App has distinctive custom icon
**Research**: Complete (10-RESEARCH.md)
**Research topics**: tracing/log crate setup, file rotation, icon design/format requirements
**Plans**: 4 plans

Plans:
- [x] 10-01: File logging with "Open Logs..." menu
- [x] 10-02: Persist enabled/disabled state across restarts
- [x] 10-03: Click-to-run macros via "Run Macro" submenu
- [x] 10-04: Custom icon design (lightning bolt)

</details>

---

## v2.1 Phase Details

### Phase 11: Windows Executable
**Goal**: Professional Windows executable presentation
**Depends on**: Phase 10
**Requirements**: WIN-01, WIN-02
**Success Criteria** (what must be TRUE):
  1. Windows executable runs without spawning a console window
  2. Windows executable shows custom icon in Explorer file listing
  3. Windows executable shows custom icon in taskbar when running
  4. Windows executable shows custom icon in Alt+Tab switcher
**Research**: Complete (11-RESEARCH.md)
**Research topics**: winres vs embed-resource crate, icon format requirements (.ico), build.rs setup for Windows-only resource embedding
**Plans**: 1 plan

Plans:
- [x] 11-01: Windows executable polish (console suppression + embedded icon)

### Phase 12: Error Notifications
**Goal**: Users see failures instead of silent errors
**Depends on**: Phase 11
**Requirements**: ERR-01, ERR-02
**Success Criteria** (what must be TRUE):
  1. User sees tray notification when keystroke injection fails
  2. User sees tray notification when Accessibility permission is missing (macOS)
  3. User sees tray notification when injection is blocked (Windows UIPI)
**Research**: Likely (cross-platform notification API)
**Research topics**: notify-rust vs native-dialog, tray-icon notification support, platform-specific error conditions
**Plans**: TBD

Plans:
- [ ] 12-01: TBD

### Phase 13: Onboarding Defaults
**Goal**: New users have example macros to understand the format
**Depends on**: Phase 12
**Requirements**: ONBOARD-01
**Success Criteria** (what must be TRUE):
  1. Fresh config includes example "Hello World" macro
  2. Fresh config includes example macro demonstrating special keys and DSL features
  3. Example macros have reasonable non-conflicting hotkeys
**Research**: Unlikely (internal patterns)
**Plans**: TBD

Plans:
- [ ] 13-01: TBD

### Phase 14: macOS App Bundle
**Goal**: Professional macOS app distribution with custom icon
**Depends on**: Phase 13
**Requirements**: MAC-01
**Success Criteria** (what must be TRUE):
  1. macOS app distributed as KeyBlast.app bundle
  2. App shows lightning bolt icon in Finder
  3. App shows lightning bolt icon in Dock when running
  4. App bundle structure follows Apple conventions (Info.plist, .icns)
**Research**: Likely (cargo-bundle or manual .app structure)
**Research topics**: cargo-bundle vs manual packaging, .icns format generation, Info.plist requirements, LSUIElement for menu bar apps
**Plans**: TBD

Plans:
- [ ] 14-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → ... → 14

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Foundation | v1.0 | 1/1 | Complete | 2026-01-16 |
| 2. Global Hotkeys | v1.0 | 2/2 | Complete | 2026-01-16 |
| 3. Keystroke Injection | v1.0 | 2/2 | Complete | 2026-01-16 |
| 4. Configuration | v1.0 | 2/2 | Complete | 2026-01-16 |
| 5. Configuration UI | v1.0 | 3/3 | Complete | 2026-01-16 |
| 6. Platform Polish | v1.0 | 2/2 | Complete | 2026-01-16 |
| 7. Async Execution | v2.0 | 2/2 | Complete | 2026-01-17 |
| 8. Expanded DSL | v2.0 | 2/2 | Complete | 2026-01-17 |
| 9. Robustness | v2.0 | 2/2 | Complete | 2026-01-17 |
| 10. UX Polish | v2.0 | 4/4 | Complete | 2026-01-17 |
| 11. Windows Executable | v2.1 | 1/1 | Complete | 2026-01-17 |
| 12. Error Notifications | v2.1 | 0/TBD | Not started | - |
| 13. Onboarding Defaults | v2.1 | 0/TBD | Not started | - |
| 14. macOS App Bundle | v2.1 | 0/TBD | Not started | - |
