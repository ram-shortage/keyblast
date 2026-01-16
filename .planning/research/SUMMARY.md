# KeyBlast Research Summary

**Project:** KeyBlast — Hotkey-triggered keystroke injector
**Researched:** 2026-01-16
**Overall Confidence:** HIGH

---

## Executive Summary

KeyBlast has a clear path to implementation. The Rust ecosystem provides mature, well-maintained crates for all core requirements. The stack is simpler than initially expected — no async runtime, no web framework, just a single-threaded event loop with native system tray and global hotkeys.

**Key insight:** The Tauri team maintains `tray-icon`, `global-hotkey`, and `muda` — all designed to work together with the same event loop pattern. This makes integration straightforward.

**Critical path:** macOS Accessibility permissions are the #1 implementation risk. Without explicit user permission, both hotkey detection and keystroke injection fail silently.

---

## Stack Decision

| Component | Choice | Confidence |
|-----------|--------|------------|
| System Tray | `tray-icon` v0.21 | HIGH |
| Global Hotkeys | `global-hotkey` v0.7 | HIGH |
| Keystroke Injection | `enigo` v0.6 | HIGH |
| Configuration | `serde` + `toml` | HIGH |
| Event Loop | Standard (no async) | HIGH |

**Why this works:** All three UI crates share the same main-thread event loop requirement, making them naturally compatible. No async runtime needed — KeyBlast responds to discrete events synchronously.

---

## Feature Scope Validation

Research confirms PROJECT.md scope is well-calibrated:

**Table Stakes (must have):**
- Global hotkey registration
- Keystroke injection (plain text + special keys)
- System tray presence
- Persistent configuration
- Hotkey conflict detection
- Enable/disable toggle

**Differentiated by design:**
- Hotkey-only triggers (vs abbreviation-based like Espanso)
- No clipboard involvement
- Offline-only, privacy-first

**Confirmed anti-features:**
- Abbreviation triggers require system-wide key monitoring — more invasive
- Scripting adds complexity without core value
- Cloud sync is a liability, not a feature

---

## Architecture Summary

```
Config File (TOML)
       ↓ load
   Event Loop ←── Hotkey Listener
       ↓
  Keystroke Injector
       ↓
   Target App
```

**Threading:** Single main thread for everything. macOS requires this anyway.

**Module structure:**
```
src/
  main.rs        // Entry, event loop
  app.rs         // State management
  config.rs      // Load/save
  hotkey.rs      // Registration wrapper
  inject.rs      // Keystroke wrapper
  tray.rs        // Tray + menu
  platform/      // OS-specific (permissions, auto-start)
```

---

## Critical Pitfalls

### CRITICAL: macOS Accessibility Permissions

Without Accessibility permission:
- `global-hotkey` events are silently ignored
- `enigo` injection fails silently

**Prevention:** Check `AXIsProcessTrusted()` at startup, guide user through permission grant if needed.

### CRITICAL: Windows UIPI

`SendInput` cannot inject into elevated (Run as Administrator) windows. Returns success but input is dropped.

**Prevention:** Document limitation. Do not run KeyBlast as admin by default.

### HIGH: Modifier Key State Corruption

If user holds modifier when hotkey fires, keys may get "stuck".

**Prevention:** Release all modifiers before and after macro playback.

### MEDIUM: Keystroke Timing

Fast injection overwhelms slow apps (Electron, IDEs).

**Prevention:** Configurable per-macro delay (already in requirements).

---

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation
**What:** System tray + basic menu + quit functionality
**Why:** Proves cross-platform foundation works
**Uses:** tray-icon, muda
**Avoids:** P-CROSS-2 (tray differences) by testing both platforms early

### Phase 2: Hotkey Detection
**What:** Global hotkey registration + event handling
**Why:** Core input mechanism, must work before output
**Uses:** global-hotkey
**Avoids:** P-CROSS-3 (hotkey conflicts) with graceful failure handling

### Phase 3: Keystroke Injection
**What:** Text and special key injection
**Why:** Core output mechanism, completes value loop
**Uses:** enigo
**Avoids:** P-MAC-1 (accessibility) with permission checking
**Avoids:** P-CROSS-5 (timing) with configurable delays

### Phase 4: Configuration
**What:** TOML config file, load/save, macro registry
**Why:** Makes macros user-configurable
**Uses:** serde, toml, dirs
**Avoids:** P-DIST-2 (config location) with dirs crate

### Phase 5: Configuration UI
**What:** Tray menu enhancements, "Open Config", "Reload"
**Why:** User-friendly config management without separate window
**Avoids:** Complexity of full GUI

### Phase 6: Platform Polish
**What:** Accessibility prompts, auto-start, error handling, packaging
**Why:** Production readiness
**Avoids:** P-MAC-3 (notarization), P-WIN-5 (signing) with proper distribution pipeline

**Phase ordering rationale:**
- Phases 1-3 build core value progressively (tray → input → output)
- Phase 4 makes it configurable (required for real use)
- Phase 5-6 are polish (can ship earlier if needed)

**Research flags:**
- Phase 3 (keystroke injection): Test extensively on both platforms early
- Phase 6 (packaging): Code signing requires certificates ($99-500/year)

---

## Open Questions for Planning

1. **Config UI approach:** "Open Config Folder" vs native dialog for adding macros?
2. **Auto-start mechanism:** Use `auto-launch` crate or custom per-platform?
3. **Hot-reload:** Auto-reload config on file change, or manual "Reload" menu item?
4. **Code signing:** Budget for Apple + Windows certificates for public distribution?

---

## Confidence Assessment

| Dimension | Confidence | Notes |
|-----------|------------|-------|
| Stack selection | HIGH | All crates verified current, well-maintained |
| Architecture | HIGH | Pattern proven in Tauri ecosystem |
| Feature scope | HIGH | Aligns with market, clear boundaries |
| Pitfall identification | HIGH | Platform-specific issues well-documented |
| Phase ordering | MEDIUM | Logical but not empirically tested |
| Time estimates | N/A | Not provided per guidelines |

---

## Sources Summary

- [tray-icon](https://github.com/tauri-apps/tray-icon) — System tray
- [global-hotkey](https://docs.rs/global-hotkey) — Hotkey registration
- [enigo](https://github.com/enigo-rs/enigo) — Keystroke injection
- [Apple Accessibility docs](https://developer.apple.com/documentation/accessibility/axisprocesstrusted())
- [Microsoft SendInput docs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [Espanso](https://github.com/espanso/espanso) — Reference architecture

---

*Research synthesized for KeyBlast roadmap planning. Ready for /gsd:define-requirements.*
