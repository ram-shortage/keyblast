/// System tray and menu setup for KeyBlast.
///
/// Uses tray-icon and muda crates for cross-platform tray functionality.

use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, CheckMenuItem};
use muda::accelerator::Accelerator;
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// Menu item identifiers for event handling.
pub struct MenuIds {
    pub toggle: muda::MenuId,
    pub quit: muda::MenuId,
}

/// Load the application icon from the assets directory.
pub fn load_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

/// Build the tray menu with enable/disable toggle and quit option.
///
/// Returns the menu and the menu item IDs for event handling.
pub fn build_menu(enabled: bool) -> (Menu, MenuIds) {
    let menu = Menu::new();

    // Create the toggle item as a CheckMenuItem (no keyboard accelerator)
    let toggle_item = CheckMenuItem::new("Enable", true, enabled, None::<Accelerator>);
    let toggle_id = toggle_item.id().clone();

    // Create the quit item (no keyboard accelerator)
    let quit_item = MenuItem::new("Quit", true, None::<Accelerator>);
    let quit_id = quit_item.id().clone();

    // Build menu structure
    menu.append(&toggle_item).expect("Failed to add toggle item");
    menu.append(&PredefinedMenuItem::separator()).expect("Failed to add separator");
    menu.append(&quit_item).expect("Failed to add quit item");

    let ids = MenuIds {
        toggle: toggle_id,
        quit: quit_id,
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

/// Get the menu event receiver for handling menu clicks.
pub fn menu_receiver() -> crossbeam_channel::Receiver<MenuEvent> {
    MenuEvent::receiver().clone()
}
