---
phase: 5-configuration-ui
plan: 02
type: execute
wave: 2
depends_on: ["5-01"]
files_modified:
  - src/tray.rs
  - src/main.rs
autonomous: true

must_haves:
  truths:
    - "Tray menu shows macros organized by group"
    - "User can see all their macros listed in the tray menu"
    - "Menu items exist for Create, Edit, Delete, Export, Import"
  artifacts:
    - path: "src/tray.rs"
      provides: "Dynamic menu building with macro submenus"
      exports: ["build_menu", "MenuIds"]
    - path: "src/main.rs"
      provides: "Menu rebuilding when config changes"
      contains: "rebuild_menu"
  key_links:
    - from: "src/tray.rs"
      to: "config::MacroDefinition"
      via: "build_menu receives macros"
      pattern: "Vec<MacroDefinition>"
    - from: "src/main.rs"
      to: "src/tray.rs"
      via: "calls build_menu with current macros"
      pattern: "tray::build_menu"
---

<objective>
Restructure tray menu to display macros grouped by category with management actions.

Purpose: Provides the UI surface for CONF-02/03/04 (create/edit/delete) and shows organized macros (ORGN-01).
Output: Dynamic tray menu that reflects current macro configuration.
</objective>

<execution_context>
@~/.claude/get-shit-done/workflows/execute-plan.md
@~/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@src/tray.rs
@src/main.rs
@src/config.rs
</context>

<tasks>

<task type="auto">
  <name>Task 1: Redesign MenuIds and build_menu for dynamic content</name>
  <files>src/tray.rs</files>
  <action>
Redesign the tray menu structure and MenuIds to support:
1. Enable/Disable toggle (existing)
2. Macros section with grouped submenus
3. Management actions (Edit Config, Export, Import)
4. Separator + Quit (existing)

Update MenuIds struct:
```rust
pub struct MenuIds {
    pub toggle: muda::MenuId,
    pub edit_config: muda::MenuId,
    pub export_macros: muda::MenuId,
    pub import_macros: muda::MenuId,
    pub quit: muda::MenuId,
    /// Map from menu item ID to macro name for delete actions
    pub delete_macro_ids: std::collections::HashMap<muda::MenuId, String>,
}
```

Update build_menu to accept macros and build dynamic structure:
```rust
/// Build the tray menu with macros organized by group.
///
/// Menu structure:
/// - [x] Enable
/// - ---
/// - Macros > (submenu showing grouped macros)
///   - [Group Name] > (submenu if group exists)
///     - Macro Name (Ctrl+Shift+K) > Delete
///   - [Ungrouped] > (for macros without group)
///     - Macro Name (hotkey) > Delete
/// - ---
/// - Edit Config File...
/// - Export Macros...
/// - Import Macros...
/// - ---
/// - Quit
pub fn build_menu(enabled: bool, macros: &[config::MacroDefinition]) -> (Menu, MenuIds)
```

Implementation approach:
1. Group macros by their `group` field (None -> "Ungrouped")
2. Create a Submenu for "Macros"
3. For each group, create a nested Submenu
4. For each macro in a group, create a Submenu with just "Delete" action
   - Display format: "macro_name (hotkey)"
5. Track delete menu item IDs in HashMap for event handling

Import needed: `use crate::config;` and `use std::collections::HashMap;`
  </action>
  <verify>`cargo build` succeeds</verify>
  <done>build_menu accepts macros slice and creates grouped menu structure with all action items</done>
</task>

<task type="auto">
  <name>Task 2: Update main.rs to pass macros to build_menu</name>
  <files>src/main.rs</files>
  <action>
Update KeyBlastApp to use the new build_menu signature:

1. In `resumed()`, after loading config, pass macros to build_menu:
```rust
let (menu, menu_ids) = tray::build_menu(self.state.enabled, &final_config.macros);
```

2. Update KeyBlastApp struct to store menu_ids with new fields:
```rust
menu_ids: tray::MenuIds,
```

3. Update the default initialization of menu_ids in KeyBlastApp::new():
```rust
menu_ids: tray::MenuIds {
    toggle: muda::MenuId::new(""),
    edit_config: muda::MenuId::new(""),
    export_macros: muda::MenuId::new(""),
    import_macros: muda::MenuId::new(""),
    quit: muda::MenuId::new(""),
    delete_macro_ids: std::collections::HashMap::new(),
},
```

Note: The actual menu action handlers will be implemented in plan 05-03.
For now, just ensure the menu builds correctly with the new structure.
  </action>
  <verify>`cargo build` succeeds, app runs and shows new menu structure</verify>
  <done>main.rs passes macros to build_menu, app compiles and runs</done>
</task>

<task type="auto">
  <name>Task 3: Add menu rebuild capability</name>
  <files>src/main.rs</files>
  <action>
Add ability to rebuild the menu when config changes (needed for import and delete).

Create a helper method on KeyBlastApp:
```rust
impl KeyBlastApp {
    /// Rebuild the tray menu with current macros.
    /// Call after config changes (import, delete).
    fn rebuild_menu(&mut self) {
        if let Some(ref config) = self.config {
            let (menu, menu_ids) = tray::build_menu(self.state.enabled, &config.macros);

            // Update the tray icon's menu
            if let Some(ref tray_icon) = self._tray_icon {
                tray_icon.set_menu(Some(Box::new(menu.clone())));
            }

            self.menu = menu;
            self.menu_ids = menu_ids;
        }
    }
}
```

This method will be called by action handlers in 05-03 after modifying config.
  </action>
  <verify>`cargo build` succeeds</verify>
  <done>rebuild_menu method exists and can be called after config changes</done>
</task>

</tasks>

<verification>
1. `cargo build` completes without errors
2. App runs and tray menu shows:
   - Enable toggle (with checkmark)
   - Macros submenu with grouped structure
   - Edit Config File, Export Macros, Import Macros items
   - Quit item
3. Default "example" macro appears in Ungrouped section
4. Menu items are visible and clickable (handlers pending 05-03)
</verification>

<success_criteria>
- Tray menu shows macros organized by group (Ungrouped for None)
- Each macro shows name and hotkey
- Each macro has Delete action in submenu
- Edit Config, Export, Import menu items present
- rebuild_menu method available for config changes
</success_criteria>

<output>
After completion, create `.planning/phases/5-configuration-ui/5-02-SUMMARY.md`
</output>
