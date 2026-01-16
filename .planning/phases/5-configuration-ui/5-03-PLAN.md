---
phase: 5-configuration-ui
plan: 03
type: execute
wave: 3
depends_on: ["5-01", "5-02"]
files_modified:
  - src/main.rs
  - Cargo.toml
autonomous: true

must_haves:
  truths:
    - "User clicks 'Edit Config File' and system text editor opens config.toml"
    - "User saves changes in editor and macros hot-reload automatically (no restart)"
    - "User can delete a macro via tray menu and it takes effect immediately"
    - "User can export macros to a chosen file via native save dialog"
    - "User can import macros from a chosen file via native open dialog"
  artifacts:
    - path: "src/main.rs"
      provides: "Event handlers for all menu actions plus file watcher for hot-reload"
      contains: "edit_config, export_macros, import_macros, notify"
    - path: "Cargo.toml"
      provides: "notify crate for file watching"
      contains: "notify"
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
      via: "After delete, import, or config file change"
      pattern: "self.rebuild_menu"
    - from: "notify::Watcher"
      to: "config::load_config"
      via: "File change event triggers reload"
      pattern: "RecommendedWatcher"
---

<objective>
Implement all menu action handlers: edit config, delete macro, export, import, plus file watcher for hot-reload.

Purpose: Completes CONF-02/03 (create/edit via config file with hot-reload), CONF-04 (delete), CONF-05 (export), CONF-06 (import).
Output: Fully functional macro management via tray menu with live config reloading.
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
@src/hotkey.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add notify crate and implement file watcher for config hot-reload</name>
  <files>Cargo.toml, src/main.rs</files>
  <action>
Add notify crate to Cargo.toml:

```toml
notify = "6"
```

In main.rs, add file watcher setup. The watcher monitors config.toml and triggers reload on changes:

```rust
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use std::sync::mpsc;

// In KeyBlastApp struct, add:
config_watcher: Option<RecommendedWatcher>,
config_change_rx: Option<mpsc::Receiver<notify::Result<Event>>>,

// In KeyBlastApp::new(), initialize:
config_watcher: None,
config_change_rx: None,

// In resumed(), after loading config, set up file watcher:
fn setup_config_watcher(&mut self) {
    let (tx, rx) = mpsc::channel();

    let watcher_result = RecommendedWatcher::new(
        move |res| {
            let _ = tx.send(res);
        },
        notify::Config::default(),
    );

    match watcher_result {
        Ok(mut watcher) => {
            let config_path = config::config_path();
            if let Err(e) = watcher.watch(&config_path, RecursiveMode::NonRecursive) {
                eprintln!("Failed to watch config file: {}", e);
            } else {
                println!("Watching config file for changes: {}", config_path.display());
                self.config_watcher = Some(watcher);
                self.config_change_rx = Some(rx);
            }
        }
        Err(e) => {
            eprintln!("Failed to create config watcher: {}", e);
        }
    }
}

// In about_to_wait(), check for config file changes (non-blocking):
fn check_config_changes(&mut self) {
    if let Some(ref rx) = self.config_change_rx {
        // Non-blocking receive - check if there are any pending events
        while let Ok(result) = rx.try_recv() {
            if let Ok(event) = result {
                // Only reload on modify events (not access, create, etc.)
                if matches!(event.kind, EventKind::Modify(_)) {
                    println!("Config file changed, reloading...");
                    self.reload_config();
                }
            }
        }
    }
}

fn reload_config(&mut self) {
    match config::load_config() {
        Ok(new_config) => {
            // Unregister all old hotkeys
            if let Some(ref mut manager) = self.hotkey_manager {
                for (_, macro_def) in self.macros.drain() {
                    if let Some(hotkey) = config::parse_hotkey_string(&macro_def.hotkey) {
                        let _ = manager.unregister(&hotkey);
                    }
                }
            }

            // Register new hotkeys
            for macro_def in &new_config.macros {
                if let Some(ref mut manager) = self.hotkey_manager {
                    if let Some(hotkey) = config::parse_hotkey_string(&macro_def.hotkey) {
                        match manager.register(hotkey, macro_def.name.clone()) {
                            Ok(()) => {
                                let hotkey_id = hotkey.id();
                                self.macros.insert(hotkey_id, macro_def.clone());
                                println!("Registered: {} -> {}", macro_def.hotkey, macro_def.name);
                            }
                            Err(e) => {
                                eprintln!("Failed to register '{}': {}", macro_def.name, e);
                            }
                        }
                    }
                }
            }

            self.config = Some(new_config);
            self.rebuild_menu();
            println!("Config reloaded successfully");
        }
        Err(e) => {
            eprintln!("Failed to reload config: {}", e);
        }
    }
}
```

Call `setup_config_watcher()` at end of `resumed()`.
Call `check_config_changes()` at start of `about_to_wait()`.
  </action>
  <verify>`cargo build` succeeds, edit config.toml externally, changes apply without restart</verify>
  <done>Config file changes are detected and macros hot-reload automatically</done>
</task>

<task type="auto">
  <name>Task 2: Implement Edit Config File and Delete Macro actions</name>
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

    println!("Changes will be applied automatically when you save the file.");
}
```

Add handler for delete actions. Use the binding returned by find() directly (no redundant .get()):

```rust
// Check if this is a delete macro action
if let Some(macro_name) = self.menu_ids.delete_macro_ids.get(&event.id) {
    println!("Deleting macro: {}", macro_name);

    if let Some(ref mut cfg) = self.config {
        // Find and remove the macro by name
        let original_len = cfg.macros.len();
        cfg.macros.retain(|m| m.name != *macro_name);

        if cfg.macros.len() < original_len {
            // Unregister the hotkey - find the binding directly
            if let Some(ref mut manager) = self.hotkey_manager {
                // Find and use the binding in one step
                if let Some((&hotkey_id, binding)) = self.macros.iter()
                    .find(|(_, m)| m.name == *macro_name)
                {
                    if let Some(hotkey) = config::parse_hotkey_string(&binding.hotkey) {
                        let _ = manager.unregister(&hotkey);
                    }
                    // Remove after iteration
                    let id_to_remove = hotkey_id;
                    self.macros.remove(&id_to_remove);
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

Place delete check BEFORE the toggle/quit checks in the event loop since delete IDs are dynamic.
  </action>
  <verify>`cargo build` succeeds, Edit Config opens editor, Delete removes macro and updates menu</verify>
  <done>Edit Config opens config.toml, Delete removes macro and unregisters hotkey correctly</done>
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

Add Import handler. Note: HotkeyManager::register signature is `register(&mut self, hotkey: HotKey, macro_id: String)`:

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
                            // HotkeyManager::register(hotkey: HotKey, macro_id: String) -> Result<(), String>
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
3. Hot-reload: After saving config.toml in editor, changes apply automatically (no restart needed)
4. Delete Macro:
   - Submenu shows "Delete" for each macro
   - Clicking Delete removes macro from config
   - Hotkey is unregistered
   - Menu updates immediately
5. Export Macros:
   - Shows native save dialog with .toml filter
   - Creates valid TOML file at chosen location
6. Import Macros:
   - Shows native open dialog with .toml filter
   - Adds new macros (skips duplicates by name)
   - Registers hotkeys for imported macros
   - Menu updates to show new macros
</verification>

<success_criteria>
- CONF-02: User creates macros by editing config file, hot-reloads on save
- CONF-03: User edits macros by editing config file, hot-reloads on save
- CONF-04: User deletes macros via tray menu Delete action
- CONF-05: User exports macros via Export with file dialog
- CONF-06: User imports macros via Import with file dialog
- All changes persist across restarts
- Hotkeys are properly registered/unregistered
- No restart required for config file edits
</success_criteria>

<output>
After completion, create `.planning/phases/5-configuration-ui/5-03-SUMMARY.md`
</output>
