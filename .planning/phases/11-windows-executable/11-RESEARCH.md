# Phase 11: Windows Executable - Research

**Researched:** 2026-01-17
**Domain:** Windows executable presentation (icon embedding, console suppression)
**Confidence:** HIGH

## Summary

This phase addresses two Windows-specific presentation requirements: hiding the console window for a GUI application and embedding a custom icon that displays in Explorer, taskbar, and Alt+Tab. Both are well-documented, standard practices in the Rust ecosystem.

The console window is suppressed via the `#![windows_subsystem = "windows"]` crate attribute, which has been stable since Rust 1.18. Icon embedding requires a build script (`build.rs`) that compiles a Windows resource file (`.rc`) containing the icon reference. The `winresource` crate is the recommended choice for this project because it handles cross-compilation from macOS to Windows using the MinGW toolchain (already installed: `x86_64-w64-mingw32-windres` confirmed available).

The tray icon (PNG) and executable icon (ICO) are separate concerns. The existing `assets/icon.png` must be converted to `.ico` format with multiple sizes (16x16, 32x32, 48x48, 256x256) for proper display at all Windows scales.

**Primary recommendation:** Use `#![windows_subsystem = "windows"]` for console suppression and `winresource` crate with a `build.rs` for icon embedding during cross-compilation.

## Standard Stack

The established libraries/tools for this domain:

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| winresource | 0.1.x | Embed icons/metadata into Windows executables | Maintained fork of winres, works with modern Rust (1.61+), supports GNU cross-compilation |

### Supporting
| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| mingw-w64 (windres) | 2.45 | Compile .rc files when cross-compiling | Already installed on this system |
| ImageMagick | 7.x | Convert PNG to multi-size ICO | Pre-build step, one-time conversion |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| winresource | embed-resource | Lower-level, requires writing .rc files manually; winresource is simpler |
| winresource | tauri-winres | Tauri-specific fork; winresource is more general-purpose |
| ImageMagick | ico-builder (Rust crate) | Could automate ICO creation in build.rs, but adds build dependency; ImageMagick is simpler for one-time conversion |

**Installation (already present):**
```bash
# MinGW windres already installed (verified: x86_64-w64-mingw32-windres v2.45)
# ImageMagick needs to be installed for ICO conversion:
brew install imagemagick
```

## Architecture Patterns

### Recommended Project Structure
```
keyblast/
├── Cargo.toml           # Add build = "build.rs" and build-dependencies
├── build.rs             # NEW: Windows resource compilation
├── assets/
│   ├── icon.png         # Existing tray icon (64x64)
│   ├── icon-flash.png   # Existing flash icon
│   └── icon.ico         # NEW: Multi-size ICO for Windows exe
└── src/
    └── main.rs          # Add #![windows_subsystem = "windows"]
```

### Pattern 1: Console Window Suppression
**What:** Prevent console window from appearing when Windows GUI app starts
**When to use:** Any Windows application without command-line interface
**Example:**
```rust
// Source: https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
// Place at the very top of main.rs (crate-level attribute)
#![windows_subsystem = "windows"]

// Rest of main.rs follows...
mod app;
// ...
```

**Important notes:**
- Attribute is silently ignored on non-Windows platforms (safe for cross-platform code)
- Must be crate-level attribute (at top of main.rs with `#!`)
- Using this disables stdout/stderr on Windows (no println! output visible)
- For conditional compilation during development: `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`

### Pattern 2: Icon Embedding via build.rs
**What:** Compile Windows resources into the executable at build time
**When to use:** When targeting Windows from any host platform
**Example:**
```rust
// Source: https://github.com/BenjaminRi/winresource
// build.rs
fn main() {
    // IMPORTANT: Use CARGO_CFG_TARGET_OS, not #[cfg(target_os)]
    // build.rs runs on HOST, not target - cfg attributes reflect host OS
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.compile().unwrap();
    }
}
```

### Pattern 3: ICO File with Multiple Sizes
**What:** Create .ico container with multiple resolution images
**When to use:** For Windows application icons
**Example (ImageMagick command):**
```bash
# Source: https://imagemagick.org/script/convert.php
# Create multi-size ICO from single PNG source
magick assets/icon.png -define icon:auto-resize="256,48,32,16" assets/icon.ico
```

### Anti-Patterns to Avoid
- **Using `#[cfg(target_os = "windows")]` in build.rs:** build.rs executes on the HOST system, not target. This check will be false when cross-compiling from macOS even when targeting Windows. Use `CARGO_CFG_TARGET_OS` environment variable instead.
- **Single-size ICO files:** Windows uses different icon sizes in different contexts. A single 256x256 icon will look blurry when scaled to 16x16 in lists.
- **Forgetting to add build.rs to Cargo.toml:** The `build = "build.rs"` key must be in `[package]` section.

## Don't Hand-Roll

Problems that look simple but have existing solutions:

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| ICO file creation | Custom PNG-to-ICO converter | ImageMagick `magick convert` | ICO format is complex (multi-image container), ImageMagick handles all edge cases |
| Resource compilation | Direct windres invocation | winresource crate | Handles toolchain detection, MSVC vs GNU differences, path escaping |
| Build script conditionals | `#[cfg(target_os)]` | `CARGO_CFG_TARGET_OS` env var | cfg attributes in build.rs reflect host, not target |

**Key insight:** Windows resource embedding involves toolchain-specific details (rc.exe vs windres, path formats, target detection). The winresource crate encapsulates years of cross-compilation edge case fixes.

## Common Pitfalls

### Pitfall 1: Host vs Target Confusion in build.rs
**What goes wrong:** Using `#[cfg(target_os = "windows")]` in build.rs to conditionally compile resources. When cross-compiling from macOS to Windows, this evaluates to false (macOS is the host).
**Why it happens:** Intuition says "I'm building for Windows so target_os should be windows" but build.rs runs on the host machine.
**How to avoid:** Always use `std::env::var("CARGO_CFG_TARGET_OS")` which reflects the actual compilation target.
**Warning signs:** Resources not being embedded when cross-compiling, but working when building on Windows natively.

### Pitfall 2: Icon Resource ID Ordering
**What goes wrong:** Multiple icons in resources, but wrong icon shows in Explorer.
**Why it happens:** Windows Explorer displays the "first" icon in the executable. "First" means: alphabetically first named resource, or numerically lowest ID if all are numbered.
**How to avoid:** Use a single icon resource, or ensure your main icon has the lowest ID (typically ID 1).
**Warning signs:** Different icon appears in Explorer vs taskbar vs Alt+Tab.

### Pitfall 3: Single-Size ICO Files
**What goes wrong:** Icon looks blurry or pixelated in certain Windows contexts.
**Why it happens:** Windows needs different icon sizes: 16x16 for small icons/lists, 32x32 for medium, 48x48 for large, 256x256 for jumbo/tile view. Single-size ICOs get scaled.
**How to avoid:** Generate ICO with at least 16, 32, 48, and 256 pixel variants.
**Warning signs:** Icon looks sharp in Explorer large icons view but blurry in taskbar or file lists.

### Pitfall 4: Missing windres for Cross-Compilation
**What goes wrong:** Build fails with "windres not found" or similar error.
**Why it happens:** GNU target cross-compilation requires MinGW's windres tool.
**How to avoid:** Ensure mingw-w64 is installed (`brew install mingw-w64` on macOS) and windres is in PATH.
**Warning signs:** Build works on Windows but fails when cross-compiling from macOS/Linux.

### Pitfall 5: Losing Debug Output
**What goes wrong:** `println!` and `eprintln!` output disappears, making debugging difficult.
**Why it happens:** `#![windows_subsystem = "windows"]` completely disconnects stdout/stderr.
**How to avoid:** Use file-based logging (this project already uses tracing-appender). For development, consider conditional: `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
**Warning signs:** Application runs but produces no console output.

## Code Examples

Verified patterns from official sources:

### Cargo.toml Changes
```toml
# Source: https://github.com/BenjaminRi/winresource
[package]
name = "keyblast"
version = "0.1.0"
edition = "2021"
build = "build.rs"  # ADD THIS LINE

# ... existing dependencies ...

[build-dependencies]
winresource = "0.1"
```

### Complete build.rs
```rust
// Source: https://github.com/BenjaminRi/winresource
fn main() {
    // Only compile Windows resources when targeting Windows
    // CARGO_CFG_TARGET_OS is set by Cargo and reflects the actual target
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        let mut res = winresource::WindowsResource::new();

        // Set the application icon (shows in Explorer, taskbar, Alt+Tab)
        res.set_icon("assets/icon.ico");

        // Optional: Set version info that appears in file properties
        // These are auto-populated from Cargo.toml [package] section:
        // - FileVersion from version
        // - ProductName from name
        // - FileDescription from description

        res.compile().unwrap();
    }
}
```

### main.rs Modification
```rust
// Source: https://rust-lang.github.io/rfcs/1665-windows-subsystem.html
// This MUST be at the very top of main.rs, before any other items
#![windows_subsystem = "windows"]

/// KeyBlast - A lightweight macro playback application.
// ... rest of existing main.rs ...
```

### ICO Generation Command
```bash
# Source: https://imagemagick.org/script/convert.php
# Run once to generate the ICO file from existing PNG
# The -define flag creates multiple sizes in one ICO file
magick assets/icon.png -define icon:auto-resize="256,48,32,16" assets/icon.ico

# Verify the ICO contains multiple sizes:
magick identify assets/icon.ico
# Should output multiple lines showing different dimensions
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| winres crate | winresource crate | 2022 (Rust 1.61) | Original winres broke with Rust 1.61; winresource is the maintained fork |
| Manual windres invocation | winresource/embed-resource | N/A | Crates handle toolchain detection automatically |
| Single-size icons | Multi-size ICO containers | Always best practice | Better visual quality across Windows UI contexts |

**Deprecated/outdated:**
- **winres crate:** Unmaintained, broken on Rust 1.61+. Use winresource instead.
- **Manual .rc file compilation:** While possible, winresource handles edge cases better.

## Open Questions

Things that couldn't be fully resolved:

1. **Tray icon vs Exe icon relationship**
   - What we know: They are completely separate. Tray icon (PNG) is loaded at runtime by tray-icon crate. Exe icon (ICO) is embedded at compile time.
   - What's unclear: Whether users expect them to visually match (they should).
   - Recommendation: Generate ICO from the same source PNG to ensure visual consistency.

2. **Debug build console visibility**
   - What we know: `windows_subsystem = "windows"` removes console entirely. Can use conditional `cfg_attr`.
   - What's unclear: Whether debug builds need console output (project uses file logging).
   - Recommendation: Since file logging is already implemented, unconditional `windows_subsystem = "windows"` is fine. Developers can check log files.

## Sources

### Primary (HIGH confidence)
- [Rust RFC 1665: windows_subsystem](https://rust-lang.github.io/rfcs/1665-windows-subsystem.html) - Official RFC for console window attribute
- [winresource GitHub README](https://github.com/BenjaminRi/winresource) - Cross-compilation setup, build.rs patterns
- [embed-resource docs.rs](https://docs.rs/embed-resource/latest/embed_resource/) - API documentation for alternative crate

### Secondary (MEDIUM confidence)
- [Raymond Chen: How Explorer finds the first icon](https://devblogs.microsoft.com/oldnewthing/20250210-00/?p=110854) - Icon resource ordering algorithm
- [Microsoft Icons Design Guidelines](https://learn.microsoft.com/en-us/windows/win32/uxguide/vis-icons) - Recommended icon sizes

### Tertiary (LOW confidence)
- WebSearch results for ImageMagick ICO conversion commands - Need to verify exact syntax on local system

## Metadata

**Confidence breakdown:**
- Console suppression: HIGH - Official Rust RFC, stable since 1.18
- Icon embedding: HIGH - winresource is well-documented, MinGW windres verified available
- ICO format requirements: HIGH - Microsoft documentation is authoritative
- Cross-compilation: HIGH - windres verified available on this system (v2.45)

**Research date:** 2026-01-17
**Valid until:** 2026-04-17 (90 days - this domain is stable)

---

## Implementation Checklist for Planner

Based on this research, Phase 11 implementation requires:

1. [ ] Install ImageMagick (`brew install imagemagick`)
2. [ ] Generate `assets/icon.ico` from `assets/icon.png` with multiple sizes
3. [ ] Create `build.rs` with winresource configuration
4. [ ] Add `build = "build.rs"` to Cargo.toml `[package]`
5. [ ] Add `winresource = "0.1"` to `[build-dependencies]`
6. [ ] Add `#![windows_subsystem = "windows"]` to top of `main.rs`
7. [ ] Verify cross-compilation still works: `cargo build --release --target x86_64-pc-windows-gnu`
8. [ ] Test on Windows: no console window, correct icon in Explorer/taskbar/Alt+Tab
