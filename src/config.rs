/// Configuration management for KeyBlast.
///
/// Provides persistent storage of macro definitions in a TOML configuration file.
/// Handles cross-platform config paths and serialization/deserialization.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};

/// Error type for configuration operations.
#[derive(Debug)]
pub enum ConfigError {
    /// Failed to read/write file.
    Io(io::Error),
    /// Failed to parse TOML.
    Parse(toml::de::Error),
    /// Failed to serialize to TOML.
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
            ConfigError::Serialize(e) => write!(f, "Serialize error: {}", e),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::Serialize(e)
    }
}

/// Warnings found during config validation.
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    DuplicateName(String),
    DuplicateHotkey { hotkey: String, names: Vec<String> },
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::DuplicateName(name) => {
                write!(f, "Duplicate macro name: '{}'", name)
            }
            ValidationWarning::DuplicateHotkey { hotkey, names } => {
                write!(f, "Hotkey '{}' used by multiple macros: {}", hotkey, names.join(", "))
            }
        }
    }
}

/// Validate config and return any warnings.
/// Does NOT modify the config - caller decides what to do with warnings.
pub fn validate_config(config: &Config) -> Vec<ValidationWarning> {
    let mut warnings = Vec::new();

    // Check for duplicate names
    let mut seen_names: HashMap<String, usize> = HashMap::new();
    for macro_def in &config.macros {
        *seen_names.entry(macro_def.name.clone()).or_insert(0) += 1;
    }
    for (name, count) in &seen_names {
        if *count > 1 {
            warnings.push(ValidationWarning::DuplicateName(name.clone()));
        }
    }

    // Check for duplicate hotkeys
    let mut hotkey_to_names: HashMap<String, Vec<String>> = HashMap::new();
    for macro_def in &config.macros {
        let normalized = macro_def.hotkey.to_lowercase();
        hotkey_to_names.entry(normalized).or_default().push(macro_def.name.clone());
    }
    for (hotkey, names) in hotkey_to_names {
        if names.len() > 1 {
            warnings.push(ValidationWarning::DuplicateHotkey { hotkey, names });
        }
    }

    warnings
}

/// A single macro definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroDefinition {
    /// Human-readable name for the macro.
    pub name: String,
    /// Hotkey string like "ctrl+shift+k".
    pub hotkey: String,
    /// The text to inject, with {Enter}, {Tab}, etc.
    pub text: String,
    /// Delay between keystrokes in milliseconds. 0 for instant (bulk) typing.
    #[serde(default)]
    pub delay_ms: u64,
    /// Optional group/category for organization. None means "Ungrouped".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Configuration format version for future migrations.
    #[serde(default = "default_version")]
    pub version: u32,
    /// List of macro definitions.
    #[serde(default)]
    pub macros: Vec<MacroDefinition>,
}

fn default_version() -> u32 {
    1
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: 1,
            macros: Vec::new(),
        }
    }
}

/// Get the platform-specific configuration file path.
///
/// - macOS: ~/Library/Application Support/keyblast/config.toml
/// - Windows: %APPDATA%/keyblast/config.toml
/// - Linux: ~/.config/keyblast/config.toml
pub fn config_path() -> PathBuf {
    let config_dir = if cfg!(target_os = "macos") {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
    };

    config_dir.join("keyblast").join("config.toml")
}

/// Load configuration from disk.
///
/// Returns the default configuration if the file doesn't exist.
/// Returns an error only if the file exists but cannot be parsed.
pub fn load_config() -> Result<Config, ConfigError> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

/// Save configuration to disk.
///
/// Creates parent directories if needed.
/// Writes atomically by writing to a temp file first, then renaming.
pub fn save_config(config: &Config) -> Result<(), ConfigError> {
    let path = config_path();

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize to pretty TOML
    let content = toml::to_string_pretty(config)?;

    // Write atomically: temp file then rename
    let temp_path = path.with_extension("toml.tmp");
    fs::write(&temp_path, &content)?;

    // On Windows, fs::rename fails if destination exists - remove it first
    #[cfg(target_os = "windows")]
    {
        if path.exists() {
            fs::remove_file(&path)?;
        }
    }

    fs::rename(&temp_path, &path)?;

    Ok(())
}

/// Export all macros to a TOML file at the specified path.
///
/// Creates a standalone config file containing only the macros array.
/// Useful for backup or sharing macro collections.
pub fn export_macros(macros: &[MacroDefinition], path: &std::path::Path) -> Result<(), ConfigError> {
    let export_config = Config {
        version: 1,
        macros: macros.to_vec(),
    };
    let content = toml::to_string_pretty(&export_config)?;
    fs::write(path, content)?;
    Ok(())
}

/// De-duplicate macros by name, keeping the first occurrence.
pub fn dedupe_macros(macros: Vec<MacroDefinition>) -> Vec<MacroDefinition> {
    let mut seen: HashSet<String> = HashSet::new();
    macros.into_iter().filter(|m| seen.insert(m.name.clone())).collect()
}

/// Import macros from a TOML file.
///
/// Parses a config file and returns the macros array.
/// De-duplicates by name within the imported file.
/// Does NOT modify the current config - caller decides how to merge.
pub fn import_macros(path: &std::path::Path) -> Result<Vec<MacroDefinition>, ConfigError> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(dedupe_macros(config.macros))
}

/// Parse a hotkey string like "ctrl+shift+k" into a HotKey.
///
/// # Supported modifiers (case-insensitive)
///
/// - ctrl, control
/// - shift
/// - alt, option (option is alias for alt on macOS)
/// - meta, cmd, command, super, win (all map to Meta modifier)
///
/// # Supported keys
///
/// - a-z (letter keys)
/// - 0-9 (digit keys)
/// - f1-f12 (function keys)
///
/// # Examples
///
/// ```ignore
/// let hk = parse_hotkey_string("ctrl+shift+k");
/// let hk = parse_hotkey_string("Ctrl+Alt+F1");
/// let hk = parse_hotkey_string("meta+shift+1");
/// ```
pub fn parse_hotkey_string(s: &str) -> Option<HotKey> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();

    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        let lower = part.to_lowercase();

        // Check if it's a modifier
        match lower.as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "meta" | "cmd" | "command" | "super" | "win" => modifiers |= Modifiers::META,
            _ => {
                // Not a modifier, should be the key
                key_code = parse_key_code(&lower);
            }
        }
    }

    // Must have a key code
    let code = key_code?;

    // Modifiers are optional but typical
    let mods = if modifiers.is_empty() {
        None
    } else {
        Some(modifiers)
    };

    Some(HotKey::new(mods, code))
}

/// Parse a key name into a Code.
fn parse_key_code(s: &str) -> Option<Code> {
    // Single letter (a-z)
    if s.len() == 1 {
        let c = s.chars().next()?;
        if c.is_ascii_lowercase() {
            return match c {
                'a' => Some(Code::KeyA),
                'b' => Some(Code::KeyB),
                'c' => Some(Code::KeyC),
                'd' => Some(Code::KeyD),
                'e' => Some(Code::KeyE),
                'f' => Some(Code::KeyF),
                'g' => Some(Code::KeyG),
                'h' => Some(Code::KeyH),
                'i' => Some(Code::KeyI),
                'j' => Some(Code::KeyJ),
                'k' => Some(Code::KeyK),
                'l' => Some(Code::KeyL),
                'm' => Some(Code::KeyM),
                'n' => Some(Code::KeyN),
                'o' => Some(Code::KeyO),
                'p' => Some(Code::KeyP),
                'q' => Some(Code::KeyQ),
                'r' => Some(Code::KeyR),
                's' => Some(Code::KeyS),
                't' => Some(Code::KeyT),
                'u' => Some(Code::KeyU),
                'v' => Some(Code::KeyV),
                'w' => Some(Code::KeyW),
                'x' => Some(Code::KeyX),
                'y' => Some(Code::KeyY),
                'z' => Some(Code::KeyZ),
                _ => None,
            };
        }
        // Single digit (0-9)
        if c.is_ascii_digit() {
            return match c {
                '0' => Some(Code::Digit0),
                '1' => Some(Code::Digit1),
                '2' => Some(Code::Digit2),
                '3' => Some(Code::Digit3),
                '4' => Some(Code::Digit4),
                '5' => Some(Code::Digit5),
                '6' => Some(Code::Digit6),
                '7' => Some(Code::Digit7),
                '8' => Some(Code::Digit8),
                '9' => Some(Code::Digit9),
                _ => None,
            };
        }
    }

    // Function keys (f1-f12)
    if s.starts_with('f') && s.len() <= 3 {
        if let Ok(num) = s[1..].parse::<u8>() {
            return match num {
                1 => Some(Code::F1),
                2 => Some(Code::F2),
                3 => Some(Code::F3),
                4 => Some(Code::F4),
                5 => Some(Code::F5),
                6 => Some(Code::F6),
                7 => Some(Code::F7),
                8 => Some(Code::F8),
                9 => Some(Code::F9),
                10 => Some(Code::F10),
                11 => Some(Code::F11),
                12 => Some(Code::F12),
                _ => None,
            };
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.version, 1);
        assert!(config.macros.is_empty());
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config {
            version: 1,
            macros: vec![
                MacroDefinition {
                    name: "Test Macro".to_string(),
                    hotkey: "ctrl+shift+k".to_string(),
                    text: "Hello{Enter}World".to_string(),
                    delay_ms: 0,
                    group: None,
                },
                MacroDefinition {
                    name: "Slow Macro".to_string(),
                    hotkey: "ctrl+alt+m".to_string(),
                    text: "Typing slowly...".to_string(),
                    delay_ms: 20,
                    group: Some("Work".to_string()),
                },
            ],
        };

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(&config).unwrap();

        // Deserialize back
        let parsed: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config, parsed);
    }

    #[test]
    fn test_macro_definition_serialization() {
        let macro_def = MacroDefinition {
            name: "Test".to_string(),
            hotkey: "ctrl+shift+k".to_string(),
            text: "Hello".to_string(),
            delay_ms: 0,
            group: None,
        };

        let toml_str = toml::to_string(&macro_def).unwrap();
        assert!(toml_str.contains("name = \"Test\""));
        assert!(toml_str.contains("hotkey = \"ctrl+shift+k\""));
        assert!(toml_str.contains("text = \"Hello\""));
    }

    #[test]
    fn test_delay_ms_default() {
        // When delay_ms is missing, it should default to 0
        let toml_str = r#"
            name = "Test"
            hotkey = "ctrl+k"
            text = "Hello"
        "#;

        let macro_def: MacroDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(macro_def.delay_ms, 0);
    }

    #[test]
    fn test_parse_hotkey_ctrl_shift_k() {
        let hk = parse_hotkey_string("ctrl+shift+k").unwrap();
        let expected = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyK,
        );
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn test_parse_hotkey_case_insensitive() {
        let hk1 = parse_hotkey_string("Ctrl+Shift+K").unwrap();
        let hk2 = parse_hotkey_string("CTRL+SHIFT+K").unwrap();
        let hk3 = parse_hotkey_string("ctrl+shift+k").unwrap();

        assert_eq!(hk1.id(), hk2.id());
        assert_eq!(hk2.id(), hk3.id());
    }

    #[test]
    fn test_parse_hotkey_alt_modifier() {
        let hk = parse_hotkey_string("ctrl+alt+m").unwrap();
        let expected = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::ALT),
            Code::KeyM,
        );
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn test_parse_hotkey_meta_modifier() {
        let hk1 = parse_hotkey_string("meta+shift+a").unwrap();
        let hk2 = parse_hotkey_string("cmd+shift+a").unwrap();
        let hk3 = parse_hotkey_string("command+shift+a").unwrap();
        let hk4 = parse_hotkey_string("super+shift+a").unwrap();

        // All should produce the same hotkey
        assert_eq!(hk1.id(), hk2.id());
        assert_eq!(hk2.id(), hk3.id());
        assert_eq!(hk3.id(), hk4.id());
    }

    #[test]
    fn test_parse_hotkey_digit() {
        let hk = parse_hotkey_string("ctrl+shift+1").unwrap();
        let expected = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::Digit1,
        );
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn test_parse_hotkey_function_key() {
        let hk = parse_hotkey_string("ctrl+f1").unwrap();
        let expected = HotKey::new(
            Some(Modifiers::CONTROL),
            Code::F1,
        );
        assert_eq!(hk.id(), expected.id());

        let hk12 = parse_hotkey_string("alt+f12").unwrap();
        let expected12 = HotKey::new(
            Some(Modifiers::ALT),
            Code::F12,
        );
        assert_eq!(hk12.id(), expected12.id());
    }

    #[test]
    fn test_parse_hotkey_no_modifiers() {
        // Hotkey without modifiers (unusual but valid)
        let hk = parse_hotkey_string("f1").unwrap();
        let expected = HotKey::new(None, Code::F1);
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn test_parse_hotkey_invalid() {
        // Invalid key
        assert!(parse_hotkey_string("ctrl+shift+invalid").is_none());

        // Empty string
        assert!(parse_hotkey_string("").is_none());

        // Only modifiers, no key
        assert!(parse_hotkey_string("ctrl+shift").is_none());
    }

    #[test]
    fn test_parse_hotkey_with_spaces() {
        // Should handle spaces around + separators
        let hk = parse_hotkey_string("ctrl + shift + k").unwrap();
        let expected = HotKey::new(
            Some(Modifiers::CONTROL | Modifiers::SHIFT),
            Code::KeyK,
        );
        assert_eq!(hk.id(), expected.id());
    }

    #[test]
    fn test_config_path_not_empty() {
        let path = config_path();
        assert!(!path.as_os_str().is_empty());
        assert!(path.to_string_lossy().contains("keyblast"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn test_group_field_optional() {
        // Group is optional and defaults to None
        let toml_str = r#"
            name = "Test"
            hotkey = "ctrl+k"
            text = "Hello"
        "#;
        let macro_def: MacroDefinition = toml::from_str(toml_str).unwrap();
        assert_eq!(macro_def.group, None);
    }

    #[test]
    fn test_group_field_serialization() {
        // With group set
        let macro_def = MacroDefinition {
            name: "Test".to_string(),
            hotkey: "ctrl+k".to_string(),
            text: "Hello".to_string(),
            delay_ms: 0,
            group: Some("Work".to_string()),
        };
        let toml_str = toml::to_string(&macro_def).unwrap();
        assert!(toml_str.contains("group = \"Work\""));

        // Without group (should not serialize the field)
        let macro_def_no_group = MacroDefinition {
            name: "Test".to_string(),
            hotkey: "ctrl+k".to_string(),
            text: "Hello".to_string(),
            delay_ms: 0,
            group: None,
        };
        let toml_str_no_group = toml::to_string(&macro_def_no_group).unwrap();
        assert!(!toml_str_no_group.contains("group"));
    }

    #[test]
    fn test_export_import_roundtrip() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let export_path = dir.path().join("export.toml");

        let macros = vec![
            MacroDefinition {
                name: "Macro 1".to_string(),
                hotkey: "ctrl+1".to_string(),
                text: "Text 1".to_string(),
                delay_ms: 0,
                group: Some("Group A".to_string()),
            },
            MacroDefinition {
                name: "Macro 2".to_string(),
                hotkey: "ctrl+2".to_string(),
                text: "Text 2".to_string(),
                delay_ms: 10,
                group: None,
            },
        ];

        // Export
        export_macros(&macros, &export_path).unwrap();
        assert!(export_path.exists());

        // Import
        let imported = import_macros(&export_path).unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].name, "Macro 1");
        assert_eq!(imported[0].group, Some("Group A".to_string()));
        assert_eq!(imported[1].name, "Macro 2");
        assert_eq!(imported[1].group, None);
    }

    #[test]
    fn test_import_dedupes_within_file() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let path = dir.path().join("dupes.toml");

        // Write a file with duplicate names
        let content = r#"
version = 1

[[macros]]
name = "test"
hotkey = "ctrl+1"
text = "first"

[[macros]]
name = "test"
hotkey = "ctrl+2"
text = "second"

[[macros]]
name = "unique"
hotkey = "ctrl+3"
text = "unique"
"#;
        fs::write(&path, content).unwrap();

        let imported = import_macros(&path).unwrap();
        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].name, "test");
        assert_eq!(imported[0].text, "first"); // First one wins
        assert_eq!(imported[1].name, "unique");
    }
}
