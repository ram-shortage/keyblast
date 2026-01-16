# Pitfalls Research: Cross-Platform Keystroke Injection

**Project:** KeyBlast
**Researched:** 2026-01-16
**Focus:** macOS and Windows keystroke injection, global hotkeys, system tray

---

## macOS-Specific Pitfalls

### P-MAC-1: TCC Accessibility Permission Dance (CRITICAL)

**What goes wrong:** App silently fails to inject keystrokes because it lacks Accessibility permission. Users see no error - keystrokes just don't arrive.

**Why it happens:** macOS TCC (Transparency, Consent, and Control) framework requires explicit user permission for any app to use `CGEventPost` to send synthetic keyboard events. The permission must be granted in System Preferences > Privacy & Security > Accessibility.

**Warning signs:**
- Keystrokes work in development but fail in release builds
- App works after fresh install, then stops working after macOS update
- Each new app signature (dev builds) requires re-granting permission

**Prevention:**
- Call `AXIsProcessTrusted()` at startup to check permission status
- Use `AXIsProcessTrustedWithOptions()` with `kAXTrustedCheckOptionPrompt` to show the system dialog
- Display clear UI state showing whether permissions are granted
- Include user-facing documentation for granting permissions
- Test with a fresh user account to validate permission flow

**Phase:** Phase 1 (core functionality depends on this)

**Confidence:** HIGH - verified via [Apple documentation](https://developer.apple.com/documentation/accessibility/axisprocesstrusted()) and [jano.dev](https://jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html)

---

### P-MAC-2: App Sandbox Incompatibility with Accessibility APIs

**What goes wrong:** When App Sandbox is enabled, Accessibility permission prompt never appears, `AXIsProcessTrusted()` always returns false, and the app cannot be manually added to Accessibility preferences.

**Why it happens:** App Sandbox and Accessibility APIs have fundamental conflicts. Sandboxed apps cannot reliably use `CGEventPost` or related APIs. Window management utilities and keystroke injection apps typically must ship without App Sandbox enabled.

**Warning signs:**
- Works in development, fails when packaged for distribution
- Accessibility prompt never appears
- App cannot be added to Accessibility preferences manually

**Prevention:**
- Do NOT enable App Sandbox for keystroke injection functionality
- This means KeyBlast cannot be distributed via Mac App Store
- Plan for direct distribution (DMG, Homebrew) from the start
- Document this limitation in architecture decisions

**Phase:** Phase 1 (distribution planning)

**Confidence:** HIGH - verified via [Apple Developer Forums](https://developer.apple.com/forums/tags/app-sandbox) and [electron/osx-sign wiki](https://github.com/electron/osx-sign/wiki/3.-App-Sandbox-and-Entitlements)

---

### P-MAC-3: Notarization and Hardened Runtime Requirements

**What goes wrong:** Users cannot run the app - macOS Gatekeeper blocks it with "cannot be opened because the developer cannot be verified" or requires visiting System Settings to allow it.

**Why it happens:** macOS Sequoia (26+) no longer allows Control-click bypass of Gatekeeper. Users must explicitly allow unsigned/un-notarized apps in System Settings > Privacy & Security, which many won't do.

**Warning signs:**
- App works on development machine, users report they cannot open it
- SmartScreen-like warnings on first launch
- Users in corporate environments completely blocked

**Prevention:**
- Obtain Apple Developer ID certificate ($99/year)
- Enable Hardened Runtime in build process
- Submit for notarization before distribution
- Staple notarization ticket to the app bundle
- Use `xcrun notarytool` in CI/CD pipeline
- Test the download-and-run flow from a fresh machine

**Phase:** Distribution phase (before public release)

**Confidence:** HIGH - verified via [Apple Developer Documentation](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)

---

### P-MAC-4: Keyboard Layout Fallback Surprises

**What goes wrong:** On non-US keyboard layouts, injected characters are wrong or unexpected characters appear.

**Why it happens:** enigo (the Rust library) has a fallback to ASCII-capable keyboard layout for handling non-standard input sources. This means your injected keystrokes may not match what the user expects based on their active layout.

**Warning signs:**
- Reports from international users about wrong characters
- Works in testing (US layout), fails for users with different layouts
- German users report Y/Z swapped

**Prevention:**
- Test with multiple keyboard layouts (German QWERTZ, French AZERTY minimum)
- Consider using Unicode character injection rather than virtual key codes where possible
- Document known layout limitations
- Add layout-specific testing to QA checklist

**Phase:** Phase 2 (after core functionality)

**Confidence:** MEDIUM - reported in [enigo issues](https://github.com/enigo-rs/enigo/issues) (#429)

---

### P-MAC-5: Modifier Key State Corruption

**What goes wrong:** After macro playback, modifier keys (Cmd, Shift, Option) appear "stuck" - subsequent user typing behaves as if modifiers are held down.

**Why it happens:** If the user is holding a modifier key when the macro fires, or if the macro is interrupted, the modifier key state can become desynchronized between what the OS thinks and what's actually pressed.

**Warning signs:**
- Users report "Shift stuck" after macro
- Subsequent typing is all caps or triggers shortcuts unexpectedly
- Inconsistent behavior based on how fast user releases hotkey

**Prevention:**
- Release all modifier keys at the start of macro playback
- Release all modifier keys at the end of macro playback
- Add small delay after hotkey detection before injecting
- Consider using `CGEventSourceFlagsState` to read current modifier state

**Phase:** Phase 1 (core injection logic)

**Confidence:** MEDIUM - common pattern from [AutoHotkey forums](https://www.autohotkey.com/boards/viewtopic.php?t=127078) and general keystroke injection experience

---

## Windows-Specific Pitfalls

### P-WIN-1: UIPI Blocks Injection to Elevated Windows (CRITICAL)

**What goes wrong:** Keystrokes silently fail to reach applications running as Administrator. `SendInput` returns success but the target never receives the input.

**Why it happens:** User Interface Privilege Isolation (UIPI) prevents lower-integrity processes from sending input to higher-integrity processes. A normal user-mode app cannot inject keystrokes into an elevated (Run as Administrator) application.

**Warning signs:**
- Works for most apps, fails for specific apps (usually dev tools, installers, Task Manager)
- `SendInput` reports success but target doesn't respond
- No error message or indication of failure

**Prevention:**
- Document this limitation clearly to users
- Do NOT run KeyBlast as Administrator by default (this would require UAC prompt every launch)
- Consider optional "Run as Administrator" mode for users who need it
- Detect elevated target windows and warn user that injection won't work

**Phase:** Phase 1 (core documentation), Phase 3 (elevated mode option)

**Confidence:** HIGH - verified via [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

---

### P-WIN-2: Silent SendInput Failures

**What goes wrong:** `SendInput` appears to work (returns non-zero) but keystrokes are silently dropped.

**Why it happens:** UIPI blocking does not set an error code. The function returns success even when input is blocked. There's no API to detect this condition after the fact.

**Warning signs:**
- Intermittent "macro didn't work" reports
- Works sometimes, not other times (depends on which app has focus)
- Debugging shows SendInput returning correct count

**Prevention:**
- Cannot be fully prevented - this is Windows design
- Document the limitation
- Consider heuristics to detect elevated windows (check process integrity level)
- Provide "test macro" feature so users can verify it works in their context

**Phase:** Phase 2 (user experience improvements)

**Confidence:** HIGH - verified via [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

---

### P-WIN-3: F12 Key Reserved by Debugger

**What goes wrong:** User configures F12 as hotkey but it never triggers (or triggers debugger instead).

**Why it happens:** F12 is reserved for the debugger at all times on Windows. Even when not debugging, if a kernel-mode debugger or just-in-time debugger is installed, F12 is unavailable.

**Warning signs:**
- F12 hotkey works on some machines, not others
- Works in development, fails on user machines with Visual Studio installed

**Prevention:**
- Block F12 as a configurable hotkey with clear explanation
- Add to hotkey conflict detection: "F12 is reserved by Windows for debugging"
- Suggest alternatives in UI

**Phase:** Phase 1 (hotkey configuration)

**Confidence:** HIGH - verified via [Microsoft Learn RegisterHotKey](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)

---

### P-WIN-4: Windows Key Combinations Reserved

**What goes wrong:** User wants Win+X hotkey but registration fails or Windows intercepts it.

**Why it happens:** Keyboard shortcuts involving the Windows key are reserved for the OS. Applications cannot reliably register global hotkeys using the Windows modifier.

**Warning signs:**
- Hotkey registration fails for Win+ combinations
- Hotkey triggers Windows feature instead of macro

**Prevention:**
- Block or warn against Win+ combinations in hotkey configuration
- Document supported modifier combinations (Ctrl, Alt, Shift, and combinations)
- Provide clear error message when reserved combination is attempted

**Phase:** Phase 1 (hotkey configuration)

**Confidence:** HIGH - verified via [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)

---

### P-WIN-5: SmartScreen Warnings for Unsigned Executables

**What goes wrong:** Users see "Windows protected your PC" warning, many will not click "Run anyway" or won't know how.

**Why it happens:** Windows Defender SmartScreen flags executables from "Unknown publisher" as potentially dangerous. Without code signing, every user sees this warning.

**Warning signs:**
- User reports saying "Windows blocked it"
- Low adoption despite interest
- Users asking "is this safe?"

**Prevention:**
- Obtain Windows code signing certificate (OV or EV)
- EV certificate provides immediate SmartScreen reputation
- OV certificate requires building reputation over time
- Use Azure Key Vault or similar for CI/CD signing
- Budget $200-500/year for certificate

**Phase:** Distribution phase (before public release)

**Confidence:** HIGH - verified via [Tauri Windows Code Signing Guide](https://tauri.app/distribute/sign/windows/)

---

### P-WIN-6: Interfering Modifier Key State

**What goes wrong:** Injected keystrokes include unintended modifiers because user was holding a key when macro triggered.

**Why it happens:** `SendInput` does not reset the keyboard's current state. If user has any keys pressed when macro fires, those keys may interfere with injected events.

**Warning signs:**
- Macro outputs UPPERCASE when it should be lowercase
- Macro triggers shortcuts instead of typing text
- Inconsistent results based on how user triggered hotkey

**Prevention:**
- Use `GetAsyncKeyState` to check modifier state before injection
- Release all modifiers at start of macro playback
- Add brief delay after hotkey to allow user to release keys
- Check bit 15 (0x8000) of GetAsyncKeyState result, not other bits

**Phase:** Phase 1 (core injection logic)

**Confidence:** HIGH - verified via [Microsoft Learn GetAsyncKeyState](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate)

---

## Cross-Platform Pitfalls

### P-CROSS-1: enigo Library Platform Inconsistencies

**What goes wrong:** Code works on one platform but behaves differently on another, or crashes on one platform.

**Why it happens:** enigo is a cross-platform abstraction, but platforms have fundamentally different input models. The library papers over differences but cannot eliminate them.

**Documented issues:**
- macOS: Unicode emoji injection produces wrong characters (#343)
- macOS: Right modifier keys simulate as both left AND right (#416)
- Windows: Respects language of current window for scancodes
- General: Keyboard layout handling differs by platform

**Warning signs:**
- "Works on my machine" during cross-platform testing
- Different behavior on same platform with different settings
- Timing-sensitive bugs that appear intermittently

**Prevention:**
- Test on both macOS and Windows from day one
- Create automated test matrix for different layouts/locales
- Pin enigo version and test thoroughly before upgrading
- Read enigo changelog for breaking changes

**Phase:** Throughout development

**Confidence:** HIGH - verified via [enigo GitHub issues](https://github.com/enigo-rs/enigo/issues)

---

### P-CROSS-2: System Tray Behavior Differences

**What goes wrong:** Tray icon appears on one platform but not another, or context menu behaves differently.

**Why it happens:** System tray APIs are completely different across platforms. Linux (especially Wayland) has significant limitations. Even macOS and Windows have different conventions.

**Known issues:**
- Linux Wayland: Tray icon may not appear in dev mode
- macOS: Tray icon may disappear after updates (reported regression)
- Linux: Requires libappindicator or libayatana-appindicator

**Warning signs:**
- Tray icon invisible but tooltip and context menu work (Raspberry Pi)
- Works in AppImage, fails in deb (Linux)
- Flatpak builds missing tray icon

**Prevention:**
- Focus on macOS and Windows initially (per project scope)
- Test tray icon visibility on both platforms in release builds
- Test both light and dark system themes
- Use tray-icon or tauri's tray implementation

**Phase:** Phase 1 (system tray implementation)

**Confidence:** HIGH - verified via [Tauri GitHub issues](https://github.com/tauri-apps/tauri/issues/14234)

---

### P-CROSS-3: Global Hotkey Conflicts

**What goes wrong:** User's chosen hotkey doesn't work because another application already registered it.

**Why it happens:** Global hotkeys are "first come first served" - whichever app registers first wins. There's no cross-platform API to discover what hotkeys are already taken.

**Warning signs:**
- "Hotkey doesn't work" reports with no pattern
- Works for developer, fails for users with specific software installed
- Hotkey works initially, then stops (another app launched)

**Prevention:**
- Handle registration failure gracefully with clear error message
- Suggest alternative hotkeys when registration fails
- Allow users to choose their own hotkeys rather than hardcoding
- Document commonly conflicting software (Discord, Slack, VS Code, etc.)
- Implement "test hotkey" feature

**Phase:** Phase 1 (hotkey configuration)

**Confidence:** HIGH - verified via [Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey) and [AutoHotkey forums](https://www.autohotkey.com/board/topic/8650-ahk-conflicts-with-other-apps-that-define-global-shortcuts/)

---

### P-CROSS-4: Focus Race Conditions

**What goes wrong:** Macro fires but keystrokes go to wrong application, or application focus changes mid-macro.

**Why it happens:** Between hotkey press and keystroke injection, focus can change. This is especially problematic on macOS where window activation events arrive in unpredictable order.

**Warning signs:**
- "Keystrokes went to wrong app"
- Partial macro output (some keys to right app, some to wrong app)
- Works most of the time, fails occasionally

**Prevention:**
- Inject keystrokes as quickly as possible after hotkey
- Do not force focus changes before injection
- Consider brief delay to let focus stabilize if user just switched windows
- Document that macro goes to currently focused app

**Phase:** Phase 2 (reliability improvements)

**Confidence:** MEDIUM - pattern observed in [AeroSpace issues](https://github.com/nikitabobko/AeroSpace/issues/1097) and general window manager experience

---

### P-CROSS-5: Timing and Speed Mismatches

**What goes wrong:** Fast keystroke injection overwhelms target application, causing dropped or reordered characters.

**Why it happens:** Different applications have different input processing speeds. Some apps (especially Electron apps, IDEs) have slow input handling. Injecting at maximum speed causes buffer overflows or race conditions in the target.

**Warning signs:**
- Missing characters in output
- Characters appear out of order
- Works in Notepad, fails in VS Code
- Works with short macros, fails with long ones

**Prevention:**
- Implement configurable delay between keystrokes (per-macro setting)
- Start with conservative default delay (e.g., 10-20ms)
- Allow users to tune per-macro based on target application
- Document that slow applications may need higher delays

**Phase:** Phase 1 (per-macro delay setting already in requirements)

**Confidence:** HIGH - documented in [enigo](https://github.com/enigo-rs/enigo) and [rdev](https://github.com/Narsil/rdev) libraries

---

## Keystroke Injection Pitfalls

### P-KEY-1: Unicode and Special Character Handling

**What goes wrong:** Emoji, accented characters, or non-ASCII text doesn't type correctly or produces garbage.

**Why it happens:** Keystroke injection typically works at the virtual key code level, which is designed for US-ASCII. Unicode requires different handling on each platform.

**enigo-specific:**
- macOS: `Key::Unicode('fire emoji')` enters wrong character (#343)
- Windows: Alt+Unicode code doesn't work
- rdev: Only shift and dead keys implemented for special characters

**Warning signs:**
- ASCII text works, Unicode fails
- Different results on different keyboard layouts
- Emoji appears as multiple characters or wrong character

**Prevention:**
- Clearly document supported character set (ASCII + common special keys)
- Test with Unicode input during development
- Consider separate code path for Unicode text vs. key sequences
- For MVP, potentially limit to ASCII and special keys (Enter, Tab, etc.)

**Phase:** Phase 1 (scope definition), Phase 3 (Unicode support if needed)

**Confidence:** HIGH - verified via [enigo issues](https://github.com/enigo-rs/enigo/issues) (#343, #465)

---

### P-KEY-2: Special Key Naming Inconsistency

**What goes wrong:** User configures "Enter" but system expects "Return", or "Escape" vs "Esc".

**Why it happens:** Different platforms and libraries use different names for special keys. Users have different expectations based on their OS background.

**Warning signs:**
- "Enter key doesn't work" (user typed wrong name)
- Config file from one platform doesn't work on another
- Confusing error messages about invalid key names

**Prevention:**
- Define canonical key names and document them clearly
- Support common aliases (Enter = Return, Esc = Escape)
- Validate key names on config load with helpful error messages
- Consider using a picker/dropdown rather than free text entry

**Phase:** Phase 1 (configuration design)

**Confidence:** MEDIUM - general UX pattern, reported in [enigo serde issues](https://github.com/enigo-rs/enigo/issues) (#439, #498)

---

### P-KEY-3: Modifier Key Combos in Output (OUT OF SCOPE but...)

**What goes wrong:** User expects Ctrl+C to copy but PROJECT.md says modifier combos are out of scope.

**Why it happens:** It's tempting to add "just one more feature" but modifier combos significantly complicate the injection logic and have platform-specific edge cases.

**Warning signs:**
- Feature creep requests for "just Ctrl+V support"
- Workarounds that try to simulate modifiers manually
- Inconsistent behavior when users attempt modifiers anyway

**Prevention:**
- Maintain clear scope boundary in UI/docs
- Explain WHY modifier combos are excluded (platform complexity)
- If users need this, it's a v2 feature with dedicated research

**Phase:** Not in v1.0 (maintain scope)

**Confidence:** HIGH - PROJECT.md explicitly excludes this

---

## Distribution Pitfalls

### P-DIST-1: Auto-Start Registration Differences

**What goes wrong:** "Start at login" works on one platform, fails or behaves differently on another.

**Why it happens:**
- macOS: Login Items API, LaunchAgents, or deprecated methods
- Windows: Registry Run keys, Task Scheduler, or Startup folder
- Each has different requirements and failure modes

**Warning signs:**
- "Auto-start doesn't work" platform-specific reports
- Works after fresh install, stops working after OS update
- Works for admin users, fails for standard users

**Prevention:**
- Research current best practice for each platform separately
- Test auto-start on clean installs
- Handle registration failure gracefully
- Provide manual instructions as fallback

**Phase:** Phase 3 (auto-start feature)

**Confidence:** MEDIUM - general cross-platform pattern

---

### P-DIST-2: Config File Location Platform Differences

**What goes wrong:** Config file ends up in wrong location, inaccessible, or not portable.

**Why it happens:**
- macOS: ~/Library/Application Support/
- Windows: %APPDATA%
- Different conventions for app name casing, organization prefix

**Warning signs:**
- "Where is my config file?"
- Config lost during app update
- Multiple config files from different app versions

**Prevention:**
- Use `dirs` or `directories` crate for platform-appropriate paths
- Document config file location prominently
- Consider migration path from old locations if app name changes
- Make config human-readable (JSON/TOML)

**Phase:** Phase 1 (configuration system)

**Confidence:** HIGH - standard cross-platform pattern

---

### P-DIST-3: Certificate and Signing Cost Overhead

**What goes wrong:** Project ships unsigned, users can't or won't run it, adoption suffers.

**Why it happens:** Code signing costs money and effort:
- Apple Developer ID: $99/year
- Windows OV certificate: $200-500/year
- EV certificate (immediate SmartScreen trust): $400-700/year
- CI/CD complexity for signing

**Warning signs:**
- "I'll sign it later" becomes never
- Users complaining about warnings
- Competitors have signed releases

**Prevention:**
- Budget for certificates from project start
- Set up CI/CD signing early (GitHub Actions supports this)
- Consider the cost part of "cost of doing business" for desktop apps

**Phase:** Distribution phase planning

**Confidence:** HIGH - industry standard

---

## Prevention Strategies Summary

| Pitfall Category | Primary Prevention | Phase to Address |
|-----------------|-------------------|------------------|
| macOS Permissions | Check AXIsProcessTrusted(), clear UI for permission state | Phase 1 |
| macOS Distribution | Skip App Sandbox, plan for notarization | Phase 1, Distribution |
| Windows UIPI | Document limitation, don't run elevated by default | Phase 1 |
| Windows Signing | Budget for OV/EV certificate, CI/CD signing | Distribution |
| Cross-Platform Consistency | Test both platforms continuously, not just at release | Throughout |
| Hotkey Conflicts | Graceful failure handling, user-configurable keys | Phase 1 |
| Timing Issues | Configurable per-macro delay, conservative defaults | Phase 1 |
| Unicode | Clearly scope to ASCII + special keys for MVP | Phase 1 |
| Focus Races | Fast injection, no unnecessary focus changes | Phase 2 |

---

## Sources

### Official Documentation
- [Apple: Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
- [Apple: AXIsProcessTrusted](https://developer.apple.com/documentation/accessibility/axisprocesstrusted())
- [Microsoft: SendInput Function](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)
- [Microsoft: RegisterHotKey Function](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey)
- [Microsoft: GetAsyncKeyState Function](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate)

### Library Documentation
- [enigo GitHub](https://github.com/enigo-rs/enigo)
- [enigo Issues](https://github.com/enigo-rs/enigo/issues)
- [rdev GitHub](https://github.com/Narsil/rdev)
- [Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri Windows Code Signing](https://tauri.app/distribute/sign/windows/)

### Community Resources
- [jano.dev: Accessibility Permission in macOS](https://jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html)
- [electron/osx-sign: App Sandbox and Entitlements](https://github.com/electron/osx-sign/wiki/3.-App-Sandbox-and-Entitlements)
- [AutoHotkey: Global Shortcut Conflicts](https://www.autohotkey.com/board/topic/8650-ahk-conflicts-with-other-apps-that-define-global-shortcuts/)
- [AeroSpace: Focus Race Conditions](https://github.com/nikitabobko/AeroSpace/issues/1097)

### Tauri Issues (System Tray)
- [Wayland Tray Icon Issues](https://github.com/tauri-apps/tauri/issues/14234)
- [macOS Tray Icon Issues](https://github.com/tauri-apps/tauri/issues/13770)
