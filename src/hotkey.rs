/// Global hotkey management for KeyBlast.
///
/// Provides registration and lookup of global keyboard shortcuts that trigger macro playback.

use std::collections::HashMap;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;

/// Result of attempting to register a hotkey.
#[derive(Debug)]
pub enum RegisterResult {
    /// Hotkey was successfully registered.
    Success,
    /// Hotkey is already registered by KeyBlast.
    ConflictInternal(String),
    /// Hotkey is taken by the OS or another application.
    ConflictExternal(String),
    /// Other registration error.
    Error(String),
}

/// A binding between a hotkey and its associated macro.
pub struct HotkeyBinding {
    pub hotkey: HotKey,
    pub macro_id: String,
}

/// Manages global hotkey registration and lookup.
pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    bindings: HashMap<u32, HotkeyBinding>,
}

impl HotkeyManager {
    /// Create a new HotkeyManager.
    ///
    /// Must be called on the main thread (required on macOS).
    pub fn new() -> Result<Self, global_hotkey::Error> {
        let manager = GlobalHotKeyManager::new()?;
        Ok(Self {
            manager,
            bindings: HashMap::new(),
        })
    }

    /// Try to register a hotkey with an associated macro ID.
    ///
    /// Returns a RegisterResult indicating success or the type of conflict.
    pub fn try_register(&mut self, hotkey: HotKey, macro_id: String) -> RegisterResult {
        // Check if already registered internally first
        if self.bindings.contains_key(&hotkey.id()) {
            return RegisterResult::ConflictInternal(format!(
                "Hotkey {} is already registered by KeyBlast",
                hotkey.into_string()
            ));
        }

        match self.manager.register(hotkey) {
            Ok(()) => {
                self.bindings.insert(hotkey.id(), HotkeyBinding { hotkey, macro_id });
                RegisterResult::Success
            }
            Err(global_hotkey::Error::AlreadyRegistered(hk)) => {
                RegisterResult::ConflictInternal(format!(
                    "Hotkey {} is already registered by KeyBlast",
                    hk.into_string()
                ))
            }
            Err(global_hotkey::Error::FailedToRegister(msg)) => {
                RegisterResult::ConflictExternal(format!(
                    "Hotkey unavailable (may be used by system or another app): {}",
                    msg
                ))
            }
            Err(e) => RegisterResult::Error(format!("Registration error: {}", e)),
        }
    }

    /// Register a hotkey with an associated macro ID.
    ///
    /// Returns an error if the hotkey is already registered by this app or the OS.
    pub fn register(&mut self, hotkey: HotKey, macro_id: String) -> Result<(), String> {
        match self.try_register(hotkey, macro_id) {
            RegisterResult::Success => Ok(()),
            RegisterResult::ConflictInternal(msg) => Err(msg),
            RegisterResult::ConflictExternal(msg) => Err(msg),
            RegisterResult::Error(msg) => Err(msg),
        }
    }

    /// Register a hotkey without a macro name (for system hotkeys like stop).
    pub fn register_raw(&mut self, hotkey: HotKey) -> Result<(), global_hotkey::Error> {
        self.manager.register(hotkey)
    }

    /// Unregister a hotkey.
    ///
    /// Returns an error if the hotkey was not registered.
    pub fn unregister(&mut self, hotkey: &HotKey) -> Result<(), String> {
        self.manager.unregister(*hotkey)
            .map_err(|e| format!("Failed to unregister: {}", e))?;
        self.bindings.remove(&hotkey.id());
        Ok(())
    }

    /// Look up the macro ID for a given hotkey ID.
    pub fn get_macro_id(&self, hotkey_id: u32) -> Option<&str> {
        self.bindings.get(&hotkey_id).map(|b| b.macro_id.as_str())
    }

    /// Returns up to `count` available hotkey combinations.
    ///
    /// Tests each candidate by registering then immediately unregistering.
    /// Skips hotkeys already registered by this application.
    pub fn suggest_available(&self, count: usize) -> Vec<HotKey> {
        let mut suggestions = Vec::new();

        for candidate in candidate_hotkeys() {
            // Skip if already registered by this app
            if self.bindings.contains_key(&candidate.id()) {
                continue;
            }

            // Try to register - if it succeeds, it's available
            match self.manager.register(candidate) {
                Ok(()) => {
                    // Available! Unregister immediately
                    let _ = self.manager.unregister(candidate);
                    suggestions.push(candidate);
                    if suggestions.len() >= count {
                        break;
                    }
                }
                Err(_) => {
                    // Not available, try next
                    continue;
                }
            }
        }

        suggestions
    }
}

/// Returns a list of candidate hotkeys to try for suggestions.
///
/// These are ordered from most likely to be available (Tier 1) to less likely (Tier 2).
fn candidate_hotkeys() -> Vec<HotKey> {
    let ctrl_shift = Some(Modifiers::CONTROL | Modifiers::SHIFT);
    let ctrl_alt = Some(Modifiers::CONTROL | Modifiers::ALT);

    vec![
        // Ctrl+Shift+<letter> - rarely conflicts
        HotKey::new(ctrl_shift, Code::KeyK),
        HotKey::new(ctrl_shift, Code::KeyM),
        HotKey::new(ctrl_shift, Code::KeyJ),
        HotKey::new(ctrl_shift, Code::KeyL),
        HotKey::new(ctrl_shift, Code::KeyU),
        HotKey::new(ctrl_shift, Code::KeyI),
        HotKey::new(ctrl_shift, Code::KeyO),
        HotKey::new(ctrl_shift, Code::KeyP),
        // Ctrl+Alt+<letter> - rarely conflicts
        HotKey::new(ctrl_alt, Code::KeyK),
        HotKey::new(ctrl_alt, Code::KeyM),
        HotKey::new(ctrl_alt, Code::KeyJ),
        HotKey::new(ctrl_alt, Code::KeyL),
        // Ctrl+Shift+<number>
        HotKey::new(ctrl_shift, Code::Digit1),
        HotKey::new(ctrl_shift, Code::Digit2),
        HotKey::new(ctrl_shift, Code::Digit3),
        HotKey::new(ctrl_shift, Code::Digit4),
    ]
}

/// Format a hotkey for display.
pub fn hotkey_display_string(hotkey: &HotKey) -> String {
    hotkey.into_string()
}
