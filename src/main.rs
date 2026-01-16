/// KeyBlast - A lightweight macro playback application.
///
/// Sits in the system tray and provides hotkey-triggered keystroke injection.

mod app;
mod config;
mod hotkey;
mod injection;
mod permission;
mod tray;

use std::collections::HashMap;
use std::process;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;
use muda::MenuEvent;
use tray_icon::TrayIcon;
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
    /// Loaded configuration
    config: Option<config::Config>,
    /// Map hotkey_id -> macro definition for quick lookup
    macros: HashMap<u32, config::MacroDefinition>,
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
            config: None,
            macros: HashMap::new(),
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

            // Load configuration from disk
            let loaded_config = match config::load_config() {
                Ok(cfg) => {
                    let config_path = config::config_path();
                    if config_path.exists() {
                        println!("Config loaded from: {}", config_path.display());
                    }
                    cfg
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load config: {}. Using defaults.", e);
                    config::Config::default()
                }
            };

            // If config has no macros, create a default example macro and save it
            let final_config = if loaded_config.macros.is_empty() {
                let default_macro = config::MacroDefinition {
                    name: "example".to_string(),
                    hotkey: "ctrl+shift+k".to_string(),
                    text: "Hello from KeyBlast!{Enter}".to_string(),
                    delay_ms: 0,
                    group: None,
                };
                let mut cfg = loaded_config;
                cfg.macros.push(default_macro);

                // Save the default config so user has a template
                match config::save_config(&cfg) {
                    Ok(()) => {
                        let config_path = config::config_path();
                        println!("Created default config at: {}", config_path.display());
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to save default config: {}", e);
                    }
                }
                cfg
            } else {
                loaded_config
            };

            self.config = Some(final_config.clone());

            // Initialize hotkey manager and register macros from config
            match hotkey::HotkeyManager::new() {
                Ok(mut manager) => {
                    // Register each macro from config
                    for macro_def in &final_config.macros {
                        match config::parse_hotkey_string(&macro_def.hotkey) {
                            Some(hotkey) => {
                                match manager.register(hotkey, macro_def.name.clone()) {
                                    Ok(()) => {
                                        let hotkey_id = hotkey.id();
                                        self.macros.insert(hotkey_id, macro_def.clone());
                                        println!(
                                            "Registered macro: {} ({})",
                                            macro_def.name, macro_def.hotkey
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Failed to register macro '{}': {}",
                                            macro_def.name, e
                                        );
                                    }
                                }
                            }
                            None => {
                                eprintln!(
                                    "Invalid hotkey '{}' for macro '{}'",
                                    macro_def.hotkey, macro_def.name
                                );
                            }
                        }
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
                    // Look up macro by hotkey_id
                    if let Some(macro_def) = self.macros.get(&hotkey_event.id) {
                        println!("Hotkey triggered: {}", macro_def.name);

                        // Check if macros are enabled
                        if !self.state.enabled {
                            println!("Macros disabled, ignoring hotkey");
                            return;
                        }

                        // Inject the macro text using config-defined text and delay
                        if let Some(ref mut injector) = self.injector {
                            let segments = injection::parse_macro_sequence(&macro_def.text);
                            let mode_name = if macro_def.delay_ms == 0 {
                                "instant"
                            } else {
                                "slow"
                            };
                            println!(
                                "Injecting macro '{}' ({}): {}",
                                macro_def.name, mode_name, macro_def.text
                            );
                            match injector.execute_sequence(&segments, macro_def.delay_ms) {
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
