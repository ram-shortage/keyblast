---
phase: 5-configuration-ui
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/config.rs
  - Cargo.toml
autonomous: true

must_haves:
  truths:
    - "Macros can have an optional group/category"
    - "All macros can be exported to a TOML file"
    - "Macros can be imported from a TOML file"
  artifacts:
    - path: "src/config.rs"
      provides: "Group field on MacroDefinition, export/import functions"
      exports: ["export_macros", "import_macros"]
    - path: "Cargo.toml"
      provides: "rfd dependency for native file dialogs"
      contains: "rfd"
  key_links:
    - from: "export_macros"
      to: "save_config"
      via: "TOML serialization"
      pattern: "toml::to_string"
    - from: "import_macros"
      to: "load_config"
      via: "TOML parsing"
      pattern: "toml::from_str"
---

<objective>
Enhance configuration layer with group support, export, and import capabilities.

Purpose: Foundation for macro organization (ORGN-01) and import/export features (CONF-05, CONF-06).
Output: Extended config.rs with group field and export/import functions.
</objective>

<execution_context>
@~/.claude/get-shit-done/workflows/execute-plan.md
@~/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@src/config.rs
@Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add group field to MacroDefinition</name>
  <files>src/config.rs</files>
  <action>
Add optional `group` field to MacroDefinition struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroDefinition {
    pub name: String,
    pub hotkey: String,
    pub text: String,
    #[serde(default)]
    pub delay_ms: u64,
    /// Optional group/category for organization. None means "Ungrouped".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}
```

The `skip_serializing_if` prevents cluttering config files when group is not used.
Update the default macro in main.rs to NOT include a group (stays None).
  </action>
  <verify>`cargo build` succeeds, existing config files still load (backward compatible)</verify>
  <done>MacroDefinition has optional group field, default is None, existing configs parse correctly</done>
</task>

<task type="auto">
  <name>Task 2: Add rfd dependency and export/import functions</name>
  <files>Cargo.toml, src/config.rs</files>
  <action>
Add rfd to Cargo.toml for native file dialogs:

```toml
rfd = "0.15"
```

Add export function to config.rs:

```rust
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
    std::fs::write(path, content)?;
    Ok(())
}

/// Import macros from a TOML file.
///
/// Parses a config file and returns the macros array.
/// Does NOT modify the current config - caller decides how to merge.
pub fn import_macros(path: &std::path::Path) -> Result<Vec<MacroDefinition>, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config.macros)
}
```

Both functions work with arbitrary paths (not just the app config path) so users can choose where to export/import.
  </action>
  <verify>`cargo build` succeeds, `cargo test` passes (add unit tests for export/import)</verify>
  <done>rfd in Cargo.toml, export_macros and import_macros functions exist with tests</done>
</task>

<task type="auto">
  <name>Task 3: Add unit tests for export/import and group field</name>
  <files>src/config.rs</files>
  <action>
Add tests to the `#[cfg(test)]` module in config.rs:

```rust
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
    use std::fs;
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
```

Add tempfile as dev dependency in Cargo.toml:
```toml
[dev-dependencies]
tempfile = "3"
```
  </action>
  <verify>`cargo test` passes all new tests</verify>
  <done>Unit tests for group field and export/import pass</done>
</task>

</tasks>

<verification>
1. `cargo build` completes without errors
2. `cargo test` passes all tests including new ones
3. Existing config.toml files load correctly (backward compatibility)
4. Group field is optional and defaults to None
5. Export creates valid TOML file
6. Import parses TOML file and returns macros
</verification>

<success_criteria>
- MacroDefinition has optional group field with serde skip_serializing_if
- export_macros() writes macros to arbitrary path
- import_macros() reads macros from arbitrary path
- rfd dependency added for future file dialogs
- All tests pass
</success_criteria>

<output>
After completion, create `.planning/phases/5-configuration-ui/5-01-SUMMARY.md`
</output>
