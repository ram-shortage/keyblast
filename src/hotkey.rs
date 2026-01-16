/// Global hotkey management for KeyBlast.
///
/// Provides registration and lookup of global keyboard shortcuts that trigger macro playback.

use std::collections::HashMap;
use global_hotkey::hotkey::HotKey;
use global_hotkey::GlobalHotKeyManager;

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

    /// Register a hotkey with an associated macro ID.
    ///
    /// Returns an error if the hotkey is already registered by this app or the OS.
    pub fn register(&mut self, hotkey: HotKey, macro_id: String) -> Result<(), global_hotkey::Error> {
        self.manager.register(hotkey)?;
        self.bindings.insert(hotkey.id(), HotkeyBinding { hotkey, macro_id });
        Ok(())
    }

    /// Look up the macro ID for a given hotkey ID.
    pub fn get_macro_id(&self, hotkey_id: u32) -> Option<&str> {
        self.bindings.get(&hotkey_id).map(|b| b.macro_id.as_str())
    }
}
