/// KeyBlast - A lightweight macro playback application.
///
/// Sits in the system tray and provides hotkey-triggered keystroke injection.

mod app;
mod hotkey;
mod injection;
mod permission;
mod tray;

use std::process;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;
use muda::MenuEvent;
use tray_icon::TrayIcon;
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};

/// Custom events for the winit event loop.
#[derive(Debug)]
enum AppEvent {
    HotKey(GlobalHotKeyEvent),
}

/// Application wrapper for winit event loop integration.
struct KeyBlastApp {
    state: app::AppState,
    menu: muda::Menu,
    menu_ids: tray::MenuIds,
    _tray_icon: Option<TrayIcon>,
    hotkey_manager: Option<hotkey::HotkeyManager>,
    injector: Option<injection::KeystrokeInjector>,
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
            hotkey_manager: None,
            injector: None,
        }
    }
}

impl ApplicationHandler<AppEvent> for KeyBlastApp {
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

            // Check accessibility permission (macOS)
            if !permission::check_accessibility_permission() {
                eprintln!("Warning: Accessibility permission not granted. Keystroke injection may not work.");
                eprintln!("Grant permission in System Preferences > Privacy & Security > Accessibility");
            }

            // Initialize keystroke injector
            match injection::KeystrokeInjector::new() {
                Ok(inj) => {
                    println!("Keystroke injector initialized");
                    self.injector = Some(inj);
                }
                Err(e) => {
                    eprintln!("Failed to initialize keystroke injector: {}", e);
                }
            }

            // Initialize hotkey manager and register test hotkey
            match hotkey::HotkeyManager::new() {
                Ok(mut manager) => {
                    let test_hotkey = HotKey::new(
                        Some(Modifiers::CONTROL | Modifiers::SHIFT),
                        Code::KeyK,
                    );
                    match manager.register(test_hotkey, "test".to_string()) {
                        Ok(()) => {
                            println!("Registered test hotkey: Ctrl+Shift+K");
                        }
                        Err(e) => {
                            eprintln!("Failed to register test hotkey: {}", e);
                        }
                    }

                    // Test conflict detection - try to register same hotkey again
                    let duplicate = HotKey::new(
                        Some(Modifiers::CONTROL | Modifiers::SHIFT),
                        Code::KeyK,
                    );
                    match manager.try_register(duplicate, "duplicate".to_string()) {
                        hotkey::RegisterResult::ConflictInternal(msg) => {
                            println!("Conflict test passed: {}", msg);
                        }
                        other => {
                            println!("Unexpected result: {:?}", other);
                        }
                    }

                    // Get 3 available hotkey suggestions
                    let suggestions = manager.suggest_available(3);
                    println!("Available hotkeys:");
                    for hk in &suggestions {
                        println!("  - {}", hotkey::hotkey_display_string(hk));
                    }

                    self.hotkey_manager = Some(manager);
                }
                Err(e) => {
                    eprintln!("Failed to create hotkey manager: {}", e);
                }
            }

            println!("KeyBlast running. Right-click tray icon for menu.");
        }
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {
        // No windows in this application
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: AppEvent) {
        match event {
            AppEvent::HotKey(hotkey_event) => {
                if hotkey_event.state == HotKeyState::Pressed {
                    if let Some(ref manager) = self.hotkey_manager {
                        if let Some(macro_id) = manager.get_macro_id(hotkey_event.id) {
                            println!("Hotkey triggered: {}", macro_id);

                            // Check if macros are enabled
                            if !self.state.enabled {
                                println!("Macros disabled, ignoring hotkey");
                                return;
                            }

                            // Test macro sequence demonstrating special keys
                            let test_macro = "Hello from KeyBlast!{Enter}";
                            let test_delay_ms: u64 = 0; // Instant typing

                            // Inject the macro text
                            if let Some(ref mut injector) = self.injector {
                                let segments = injection::parse_macro_sequence(test_macro);
                                println!("Injecting macro: {}", test_macro);
                                match injector.execute_sequence(&segments, test_delay_ms) {
                                    Ok(()) => {
                                        println!("Injection complete");
                                    }
                                    Err(e) => {
                                        eprintln!("Injection failed: {}", e);
                                    }
                                }
                            } else {
                                eprintln!("No injector available");
                            }
                        }
                    }
                }
            }
        }
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
    // Create the event loop with custom event type for hotkey integration
    let event_loop = EventLoop::<AppEvent>::with_user_event()
        .build()
        .expect("Failed to create event loop");

    // Set up global hotkey event forwarding to the winit event loop
    let proxy = event_loop.create_proxy();
    GlobalHotKeyEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(AppEvent::HotKey(event));
    }));

    // Set control flow to wait so we check for events regularly
    event_loop.set_control_flow(ControlFlow::Wait);

    // Create and run the application
    let mut app = KeyBlastApp::new();
    event_loop
        .run_app(&mut app)
        .expect("Failed to run event loop");
}
