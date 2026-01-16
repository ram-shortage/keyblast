/// KeyBlast - A lightweight macro playback application.
///
/// Sits in the system tray and provides hotkey-triggered keystroke injection.

mod app;
mod tray;

use std::process;

fn main() {
    println!("KeyBlast initializing...");

    // Initialize application state
    let mut state = app::AppState::new();

    // Build initial menu and create tray icon
    let (menu, menu_ids) = tray::build_menu(state.enabled);
    let _tray_icon = tray::create_tray(&menu);

    // Get the menu event receiver
    let menu_receiver = tray::menu_receiver();

    println!("KeyBlast running. Right-click tray icon for menu.");

    // Main event loop
    loop {
        // Block waiting for menu events
        match menu_receiver.recv() {
            Ok(event) => {
                if event.id == menu_ids.toggle {
                    // Toggle enabled state
                    state.toggle();
                    println!("KeyBlast {}", if state.enabled { "enabled" } else { "disabled" });

                    // Update the checkbox state
                    // Find the toggle menu item and update its checked state
                    for item in menu.items() {
                        if let muda::MenuItemKind::Check(check_item) = item {
                            if check_item.id() == &menu_ids.toggle {
                                check_item.set_checked(state.enabled);
                                break;
                            }
                        }
                    }
                } else if event.id == menu_ids.quit {
                    println!("KeyBlast shutting down.");
                    process::exit(0);
                }
            }
            Err(e) => {
                eprintln!("Menu event error: {}", e);
                break;
            }
        }
    }
}
