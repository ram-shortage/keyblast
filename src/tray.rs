/// System tray and menu setup for KeyBlast.
///
/// Uses tray-icon and muda crates for cross-platform tray functionality.

use std::collections::HashMap;
use muda::{Menu, MenuItem, PredefinedMenuItem, CheckMenuItem, Submenu};
use muda::accelerator::Accelerator;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};
use uuid::Uuid;

use crate::config;
use crate::config::ValidationWarning;

/// Menu item identifiers for event handling.
pub struct MenuIds {
    pub toggle: muda::MenuId,
    pub edit_config: muda::MenuId,
    pub export_macros: muda::MenuId,
    pub import_macros: muda::MenuId,
    pub auto_start: muda::MenuId,
    pub stop_macro: muda::MenuId,
    pub quit: muda::MenuId,
    /// Map from menu item ID to macro UUID for delete actions
    pub delete_macro_ids: HashMap<muda::MenuId, Uuid>,
    /// Map from menu item ID to macro UUID for run actions
    pub run_macro_ids: HashMap<muda::MenuId, Uuid>,
}

/// Load the normal application icon.
pub fn load_icon() -> Icon {
    load_icon_from_bytes(include_bytes!("../assets/icon.png"))
}

/// Load the flash variant icon for visual feedback.
/// Currently uses the same icon; visual feedback comes from the toggling effect.
pub fn load_flash_icon() -> Icon {
    load_icon_from_bytes(include_bytes!("../assets/icon-flash.png"))
}

fn load_icon_from_bytes(bytes: &[u8]) -> Icon {
    let image = image::load_from_memory(bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

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
/// - Warnings (N) > (submenu if there are validation warnings)
///   - Warning 1
///   - Warning 2
/// - ---
/// - Edit Config File...
/// - Export Macros...
/// - Import Macros...
/// - ---
/// - Quit
///
/// Returns the menu and the menu item IDs for event handling.
pub fn build_menu(
    enabled: bool,
    macros: &[config::MacroDefinition],
    warnings: &[ValidationWarning],
) -> (Menu, MenuIds) {
    let menu = Menu::new();
    let mut delete_macro_ids: HashMap<muda::MenuId, Uuid> = HashMap::new();
    let mut run_macro_ids: HashMap<muda::MenuId, Uuid> = HashMap::new();

    // Create the toggle item as a CheckMenuItem (no keyboard accelerator)
    let toggle_item = CheckMenuItem::new("Enable", true, enabled, None::<Accelerator>);
    let toggle_id = toggle_item.id().clone();

    menu.append(&toggle_item).expect("Failed to add toggle item");

    // Stop Macro item (initially disabled - enabled when macro is running)
    let stop_item = MenuItem::new("Stop Macro", false, None::<Accelerator>);
    let stop_id = stop_item.id().clone();
    menu.append(&stop_item).expect("Failed to add stop item");

    menu.append(&PredefinedMenuItem::separator()).expect("Failed to add separator");

    // Build Run Macro submenu (flat alphabetized list for quick access)
    let run_submenu = Submenu::new("Run Macro", true);
    let mut sorted_macros: Vec<_> = macros.iter().collect();
    sorted_macros.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    for macro_def in &sorted_macros {
        let label = format!("{} ({})", macro_def.name, macro_def.hotkey);
        let item = MenuItem::new(&label, true, None::<Accelerator>);
        let item_id = item.id().clone();
        run_macro_ids.insert(item_id, macro_def.id);
        run_submenu.append(&item).expect("Failed to add run item");
    }

    menu.append(&run_submenu).expect("Failed to add run submenu");

    // Build the Macros submenu with grouped macros
    let macros_submenu = Submenu::new("Macros", true);

    // Group macros by their `group` field (None -> "Ungrouped")
    let mut groups: HashMap<String, Vec<&config::MacroDefinition>> = HashMap::new();
    for macro_def in macros {
        let group_name = macro_def.group.clone().unwrap_or_else(|| "Ungrouped".to_string());
        groups.entry(group_name).or_default().push(macro_def);
    }

    // Sort group names for consistent ordering, but keep "Ungrouped" at the end
    let mut group_names: Vec<&String> = groups.keys().collect();
    group_names.sort_by(|a, b| {
        if *a == "Ungrouped" {
            std::cmp::Ordering::Greater
        } else if *b == "Ungrouped" {
            std::cmp::Ordering::Less
        } else {
            a.cmp(b)
        }
    });

    for group_name in group_names {
        let group_macros = groups.get(group_name).unwrap();

        // Create a submenu for this group
        let group_submenu = Submenu::new(group_name, true);

        for macro_def in group_macros {
            // Format: "macro_name (hotkey)"
            let label = format!("{} ({})", macro_def.name, macro_def.hotkey);

            // Each macro gets a submenu with just "Delete" action
            let macro_submenu = Submenu::new(&label, true);

            let delete_item = MenuItem::new("Delete", true, None::<Accelerator>);
            let delete_id = delete_item.id().clone();
            delete_macro_ids.insert(delete_id, macro_def.id);

            macro_submenu.append(&delete_item).expect("Failed to add delete item");
            group_submenu.append(&macro_submenu).expect("Failed to add macro submenu");
        }

        macros_submenu.append(&group_submenu).expect("Failed to add group submenu");
    }

    menu.append(&macros_submenu).expect("Failed to add macros submenu");

    // Add Warnings submenu if there are validation warnings
    if !warnings.is_empty() {
        let warnings_submenu = Submenu::new(format!("Warnings ({})", warnings.len()), true);

        for warning in warnings {
            let label = warning.to_string();
            let item = MenuItem::new(&label, false, None::<Accelerator>);
            warnings_submenu.append(&item).expect("Failed to add warning item");
        }

        menu.append(&warnings_submenu).expect("Failed to add warnings submenu");
    }

    menu.append(&PredefinedMenuItem::separator()).expect("Failed to add separator");

    // Management actions
    let edit_config_item = MenuItem::new("Edit Config File...", true, None::<Accelerator>);
    let edit_config_id = edit_config_item.id().clone();

    let export_item = MenuItem::new("Export Macros...", true, None::<Accelerator>);
    let export_id = export_item.id().clone();

    let import_item = MenuItem::new("Import Macros...", true, None::<Accelerator>);
    let import_id = import_item.id().clone();

    menu.append(&edit_config_item).expect("Failed to add edit config item");
    menu.append(&export_item).expect("Failed to add export item");
    menu.append(&import_item).expect("Failed to add import item");
    menu.append(&PredefinedMenuItem::separator()).expect("Failed to add separator");

    // Auto-start toggle
    let auto_start_enabled = crate::autostart::is_auto_start_enabled();
    let auto_start_item = CheckMenuItem::new(
        "Start at Login",
        true,
        auto_start_enabled,
        None::<Accelerator>,
    );
    let auto_start_id = auto_start_item.id().clone();
    menu.append(&auto_start_item).expect("Failed to add auto-start item");
    menu.append(&PredefinedMenuItem::separator()).expect("Failed to add separator");

    // Quit item
    let quit_item = MenuItem::new("Quit", true, None::<Accelerator>);
    let quit_id = quit_item.id().clone();

    menu.append(&quit_item).expect("Failed to add quit item");

    let ids = MenuIds {
        toggle: toggle_id,
        edit_config: edit_config_id,
        export_macros: export_id,
        import_macros: import_id,
        auto_start: auto_start_id,
        stop_macro: stop_id,
        quit: quit_id,
        delete_macro_ids,
        run_macro_ids,
    };

    (menu, ids)
}

/// Create the tray icon with the given menu.
pub fn create_tray(menu: &Menu) -> TrayIcon {
    let icon = load_icon();

    TrayIconBuilder::new()
        .with_menu(Box::new(menu.clone()))
        .with_tooltip("KeyBlast")
        .with_icon(icon)
        .build()
        .expect("Failed to create tray icon")
}

