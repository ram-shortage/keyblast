# KeyBlast

A lightweight macro playback application that sits in the system tray and types pre-configured keystroke sequences when triggered by hotkeys. Built for quickly inserting code snippets, text templates, and multi-step input sequences wherever the cursor is focused.

**Press a hotkey, get consistent keystrokes injected instantly — no clipboard, no context switching, works in any application.**

## Features

- **Global Hotkeys** — Trigger macros from any application
- **System Tray** — Unobtrusive, always accessible
- **Async Execution** — Long macros don't freeze the UI
- **Stop Macro** — Cancel running macro via Ctrl+Escape or menu
- **Click-to-Run** — Execute macros from the tray menu
- **Special Keys** — Support for Enter, Tab, Escape, arrows, and more
- **Expanded DSL** — Delays, modifier keys, clipboard paste, literal braces
- **Macro Groups** — Organize macros into categories
- **Import/Export** — Share macro configurations
- **Config Validation** — Warns on duplicate names or hotkey conflicts
- **File Logging** — Troubleshoot with "Open Logs..." menu
- **Auto-Start** — Launch at login (optional)
- **Cross-Platform** — macOS and Windows

## Installation

### From Source

Requires [Rust](https://rustup.rs/).

```bash
# Clone the repo
git clone https://github.com/ram-shortage/keyblast.git
cd keyblast

# Build for your platform
cargo build --release

# Run
./target/release/keyblast
```

### Cross-Compile for Windows (from macOS)

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
# Output: target/x86_64-pc-windows-gnu/release/keyblast.exe
```

## Usage

1. Launch KeyBlast — it appears in your system tray
2. Right-click the tray icon to access the menu
3. Create macros via **New Macro...** or edit the config file directly
4. Assign a hotkey and macro text
5. Press the hotkey anywhere to inject the keystrokes

### macOS Permissions

KeyBlast requires Accessibility permissions to inject keystrokes. On first launch, you'll be prompted to grant access in **System Settings > Privacy & Security > Accessibility**.

## Configuration

Macros are stored in a TOML file:
- **macOS**: `~/Library/Application Support/keyblast/config.toml`
- **Windows**: `%APPDATA%\keyblast\config.toml`

### Example Config

```toml
[settings]
enabled = true

[[macros]]
name = "Email Signature"
hotkey = "Ctrl+Shift+S"
text = "Best regards,{Enter}John Doe"
delay_ms = 0

[[macros]]
name = "Code Block"
hotkey = "Ctrl+Shift+C"
text = "```{Enter}{Enter}```{Up}"
delay_ms = 10
group = "Developer"
```

## Macro DSL

### Special Keys

Use `{KeyName}` syntax for special keys:

| Key | Syntax |
|-----|--------|
| Enter/Return | `{Enter}` |
| Tab | `{Tab}` |
| Escape | `{Escape}` |
| Backspace | `{Backspace}` |
| Delete | `{Delete}` |
| Arrow Keys | `{Up}` `{Down}` `{Left}` `{Right}` |
| Home/End | `{Home}` `{End}` |
| Page Up/Down | `{PageUp}` `{PageDown}` |
| Function Keys | `{F1}` through `{F12}` |
| Space | `{Space}` |

### Extended Commands

| Command | Description | Example |
|---------|-------------|---------|
| `{Delay N}` | Pause for N milliseconds | `{Delay 500}` |
| `{KeyDown Mod}` | Press and hold modifier | `{KeyDown Ctrl}` |
| `{KeyUp Mod}` | Release modifier | `{KeyUp Ctrl}` |
| `{Paste}` | Type clipboard contents | `{Paste}` |
| `{{` | Literal `{` character | `{{example}}` |
| `}}` | Literal `}` character | `{{example}}` |

### Modifier Keys

For `{KeyDown}` and `{KeyUp}`:
- `Ctrl`, `LCtrl`, `RCtrl`
- `Shift`, `LShift`, `RShift`
- `Alt`, `LAlt`, `RAlt`
- `Meta` (Cmd on macOS, Win on Windows)

### Example: Complex Macro

```toml
[[macros]]
name = "Select All and Copy"
hotkey = "Ctrl+Shift+A"
text = "{KeyDown Ctrl}a{KeyUp Ctrl}{Delay 50}{KeyDown Ctrl}c{KeyUp Ctrl}"
```

## Tray Menu

- **Enable/Disable** — Toggle all macro hotkeys
- **Run Macro** — Click to execute (alphabetized list)
- **Macros** — View, edit, delete macros by group
- **New Macro...** — Create a new macro
- **Import/Export** — Share configurations
- **Open Logs...** — View application logs
- **Auto-Start** — Toggle launch at login
- **Quit** — Exit KeyBlast

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Your hotkeys | Trigger assigned macros |
| Ctrl+Escape | Stop running macro |

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a pull request.
