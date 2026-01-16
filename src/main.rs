/// KeyBlast - A lightweight macro playback application.
///
/// Sits in the system tray and provides hotkey-triggered keystroke injection.

mod app;
mod tray;

use std::process;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;
use muda::MenuEvent;
use tray_icon::TrayIcon;

/// Application wrapper for winit event loop integration.
struct KeyBlastApp {
    state: app::AppState,
    menu: muda::Menu,
    menu_ids: tray::MenuIds,
    _tray_icon: Option<TrayIcon>,
}

impl KeyBlastApp {
    fn new() -> Self {
        Self {
            state: app::AppState::new(),
            menu: muda::Menu::new(),
            menu_ids: tray::MenuIds {
                toggle: muda::MenuId::new(""),
                quit: muda::MenuId::new(""),
            },
            _tray_icon: None,
        }
    }
}

impl ApplicationHandler for KeyBlastApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Create tray icon when the application is ready
        // On macOS, this must happen after the event loop starts
        if self._tray_icon.is_none() {
            println!("KeyBlast initializing...");

            // Build menu and create tray icon
            let (menu, menu_ids) = tray::build_menu(self.state.enabled);
            let tray_icon = tray::create_tray(&menu);

            self.menu = menu;
            self.menu_ids = menu_ids;
            self._tray_icon = Some(tray_icon);

            println!("KeyBlast running. Right-click tray icon for menu.");
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {
        // No windows in this application
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Process any pending menu events
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.menu_ids.toggle {
                // Toggle enabled state
                self.state.toggle();
                println!(
                    "KeyBlast {}",
                    if self.state.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );

                // Update the checkbox state
                for item in self.menu.items() {
                    if let muda::MenuItemKind::Check(check_item) = item {
                        if check_item.id() == &self.menu_ids.toggle {
                            check_item.set_checked(self.state.enabled);
                            break;
                        }
                    }
                }
            } else if event.id == self.menu_ids.quit {
                println!("KeyBlast shutting down.");
                process::exit(0);
            }
        }
    }
}

fn main() {
    // Create the event loop - this pumps the native event loop on macOS
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Set control flow to poll so we check for menu events regularly
    event_loop.set_control_flow(ControlFlow::Wait);

    // Create and run the application
    let mut app = KeyBlastApp::new();
    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}
