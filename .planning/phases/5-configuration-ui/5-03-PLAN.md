---
phase: 5-configuration-ui
plan: 03
type: execute
wave: 3
depends_on: ["5-01", "5-02"]
files_modified:
  - src/main.rs
autonomous: true

must_haves:
  truths:
    - "User can create new macros by editing opened config file"
    - "User can edit existing macros by editing opened config file"
    - "User can delete a macro via tray menu"
    - "User can export macros to a chosen file"
    - "User can import macros from a chosen file"
  artifacts:
    - path: "src/main.rs"
      provides: "Event handlers for all menu actions"
      contains: "edit_config, export_macros, import_macros"
  key_links:
    - from: "src/main.rs"
      to: "config::export_macros"
      via: "Export menu handler"
      pattern: "config::export_macros"
    - from: "src/main.rs"
      to: "config::import_macros"
      via: "Import menu handler"
      pattern: "config::import_macros"
    - from: "src/main.rs"
      to: "config::save_config"
      via: "After import merge"
      pattern: "config::save_config"
    - from: "src/main.rs"
      to: "rebuild_menu"
      via: "After delete or import"
      pattern: "self.rebuild_menu"
---

<objective>
Implement all menu action handlers: edit config, delete macro, export, import.

Purpose: Completes CONF-02/03 (create/edit via config file), CONF-04 (delete), CONF-05 (export), CONF-06 (import).
Output: Fully functional macro management via tray menu.
</objective>

<execution_context>
@~/.claude/get-shit-done/workflows/execute-plan.md
@~/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@src/main.rs
@src/config.rs
@src/tray.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Implement Edit Config File action</name>
  <files>src/main.rs</files>
  <action>
Add handler for "Edit Config File" menu item in about_to_wait():

```rust
} else if event.id == self.menu_ids.edit_config {
    // Open config file in default editor
    let config_path = config::config_path();
    println!("Opening config file: {}", config_path.display());

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(&config_path)
            .spawn();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &config_path.to_string_lossy()])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&config_path)
            .spawn();
    }

    // Note: Config changes require app restart to take effect.
    // Future enhancement: file watcher for hot reload.
    println!("Edit the config file and restart KeyBlast to apply changes.");
}
```

This addresses CONF-02 (create) and CONF-03 (edit) - user edits the TOML file directly.
  </action>
  <verify>`cargo build` succeeds, clicking "Edit Config File" opens the config.toml in default editor</verify>
  <done>Edit Config File opens config.toml in system default text editor</done>
</task>

<task type="auto">
  <name>Task 2: Implement Delete Macro action</name>
  <files>src/main.rs</files>
  <action>
Add handler for delete actions in about_to_wait(). Check delete_macro_ids map:

```rust
// Check if this is a delete macro action
if let Some(macro_name) = self.menu_ids.delete_macro_ids.get(&event.id) {
    println!("Deleting macro: {}", macro_name);

    if let Some(ref mut cfg) = self.config {
        // Find and remove the macro by name
        let original_len = cfg.macros.len();
        cfg.macros.retain(|m| m.name != *macro_name);

        if cfg.macros.len() < original_len {
            // Unregister the hotkey
            if let Some(ref mut manager) = self.hotkey_manager {
                // Find the macro that was deleted to get its hotkey
                if let Some(hotkey_id) = self.macros.iter()
                    .find(|(_, m)| m.name == *macro_name)
                    .map(|(id, _)| *id)
                {
                    if let Some(binding) = self.macros.get(&hotkey_id) {
                        if let Some(hotkey) = config::parse_hotkey_string(&binding.hotkey) {
                            let _ = manager.unregister(&hotkey);
                        }
                    }
                    self.macros.remove(&hotkey_id);
                }
            }

            // Save updated config
            match config::save_config(cfg) {
                Ok(()) => {
                    println!("Macro '{}' deleted and config saved", macro_name);
                }
                Err(e) => {
                    eprintln!("Failed to save config after delete: {}", e);
                }
            }

            // Rebuild menu to reflect changes
            self.rebuild_menu();
        }
    }
    continue; // Skip further processing for this event
}
```

Place this check BEFORE the toggle/quit checks in the event loop since delete IDs are dynamic.
  </action>
  <verify>`cargo build` succeeds, can delete a macro from tray menu and it persists</verify>
  <done>Delete action removes macro from config, unregisters hotkey, saves config, rebuilds menu</done>
</task>

<task type="auto">
  <name>Task 3: Implement Export and Import actions</name>
  <files>src/main.rs</files>
  <action>
Add use statement at top: `use rfd::FileDialog;`

Add Export handler:
```rust
} else if event.id == self.menu_ids.export_macros {
    // Show save file dialog
    if let Some(path) = FileDialog::new()
        .add_filter("TOML", &["toml"])
        .set_file_name("keyblast-macros.toml")
        .save_file()
    {
        if let Some(ref cfg) = self.config {
            match config::export_macros(&cfg.macros, &path) {
                Ok(()) => {
                    println!("Macros exported to: {}", path.display());
                }
                Err(e) => {
                    eprintln!("Failed to export macros: {}", e);
                }
            }
        }
    }
}
```

Add Import handler:
```rust
} else if event.id == self.menu_ids.import_macros {
    // Show open file dialog
    if let Some(path) = FileDialog::new()
        .add_filter("TOML", &["toml"])
        .pick_file()
    {
        match config::import_macros(&path) {
            Ok(imported_macros) => {
                println!("Imported {} macros from: {}", imported_macros.len(), path.display());

                if let Some(ref mut cfg) = self.config {
                    // Merge imported macros (add new ones, skip duplicates by name)
                    let existing_names: std::collections::HashSet<_> =
                        cfg.macros.iter().map(|m| m.name.clone()).collect();

                    let mut added = 0;
                    for macro_def in imported_macros {
                        if !existing_names.contains(&macro_def.name) {
                            // Register the hotkey for the new macro
                            if let Some(ref mut manager) = self.hotkey_manager {
                                if let Some(hotkey) = config::parse_hotkey_string(&macro_def.hotkey) {
                                    match manager.register(hotkey, macro_def.name.clone()) {
                                        Ok(()) => {
                                            let hotkey_id = hotkey.id();
                                            self.macros.insert(hotkey_id, macro_def.clone());
                                            cfg.macros.push(macro_def);
                                            added += 1;
                                        }
                                        Err(e) => {
                                            eprintln!("Failed to register imported macro '{}': {}", macro_def.name, e);
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("Skipping duplicate macro: {}", macro_def.name);
                        }
                    }

                    // Save updated config
                    match config::save_config(cfg) {
                        Ok(()) => {
                            println!("Added {} new macros, config saved", added);
                        }
                        Err(e) => {
                            eprintln!("Failed to save config after import: {}", e);
                        }
                    }

                    // Rebuild menu to show new macros
                    self.rebuild_menu();
                }
            }
            Err(e) => {
                eprintln!("Failed to import macros: {}", e);
            }
        }
    }
}
```

Import uses merge strategy: adds new macros, skips duplicates by name.
  </action>
  <verify>`cargo build` succeeds, export saves file, import loads and merges macros</verify>
  <done>Export shows save dialog and writes TOML, Import shows open dialog, merges macros, registers hotkeys</done>
</task>

</tasks>

<verification>
1. `cargo build` completes without errors
2. Edit Config File: Opens config.toml in system default editor
3. Delete Macro:
   - Submenu shows "Delete" for each macro
   - Clicking Delete removes macro from config
   - Hotkey is unregistered
   - Menu updates immediately
4. Export Macros:
   - Shows native save dialog with .toml filter
   - Creates valid TOML file at chosen location
5. Import Macros:
   - Shows native open dialog with .toml filter
   - Adds new macros (skips duplicates by name)
   - Registers hotkeys for imported macros
   - Menu updates to show new macros
</verification>

<success_criteria>
- CONF-02: User creates macros by editing config file (Edit Config opens file)
- CONF-03: User edits macros by editing config file (same as above)
- CONF-04: User deletes macros via tray menu Delete action
- CONF-05: User exports macros via Export with file dialog
- CONF-06: User imports macros via Import with file dialog
- All changes persist across restarts
- Hotkeys are properly registered/unregistered
</success_criteria>

<output>
After completion, create `.planning/phases/5-configuration-ui/5-03-SUMMARY.md`
</output>
