/// KeyBlast - A lightweight macro playback application.
///
/// Sits in the system tray and provides hotkey-triggered keystroke injection.

mod app;
mod autostart;
mod config;
mod hotkey;
mod injection;
mod permission;
mod tray;

use std::collections::HashMap;
use std::process;
use std::sync::mpsc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;
use muda::MenuEvent;
use tray_icon::TrayIcon;
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event, EventKind};
use rfd::FileDialog;

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
    /// File watcher for config hot-reload
    config_watcher: Option<RecommendedWatcher>,
    /// Receiver for config file change events
    config_change_rx: Option<mpsc::Receiver<notify::Result<Event>>>,
    /// Flash counter for visual feedback (counts down)
    flash_remaining: u8,
    /// Normal tray icon
    normal_icon: Option<tray_icon::Icon>,
    /// Flash tray icon
    flash_icon: Option<tray_icon::Icon>,
    /// Current flash state (true = showing flash icon)
    flash_state: bool,
    /// Instant of last flash toggle for timing
    last_flash_toggle: Option<std::time::Instant>,
}

impl KeyBlastApp {
    fn new() -> Self {
        Self {
            state: app::AppState::new(),
            menu: muda::Menu::new(),
            menu_ids: tray::MenuIds {
                toggle: muda::MenuId::new(""),
                edit_config: muda::MenuId::new(""),
                export_macros: muda::MenuId::new(""),
                import_macros: muda::MenuId::new(""),
                auto_start: muda::MenuId::new(""),
                quit: muda::MenuId::new(""),
                delete_macro_ids: std::collections::HashMap::new(),
            },
            _tray_icon: None,
            hotkey_manager: None,
            injector: None,
            config: None,
            macros: HashMap::new(),
            config_watcher: None,
            config_change_rx: None,
            flash_remaining: 0,
            normal_icon: None,
            flash_icon: None,
            flash_state: false,
            last_flash_toggle: None,
        }
    }

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

    /// Set up file watcher for config hot-reload.
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

    /// Check for config file changes (non-blocking).
    fn check_config_changes(&mut self) {
        // Collect any modify events first (to avoid borrow issues)
        let mut should_reload = false;
        if let Some(ref rx) = self.config_change_rx {
            // Non-blocking receive - check if there are any pending events
            while let Ok(result) = rx.try_recv() {
                if let Ok(event) = result {
                    // Only reload on modify events (not access, create, etc.)
                    if matches!(event.kind, EventKind::Modify(_)) {
                        should_reload = true;
                    }
                }
            }
        }

        if should_reload {
            println!("Config file changed, reloading...");
            self.reload_config();
        }
    }

    /// Reload config from disk and re-register hotkeys.
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
}

impl ApplicationHandler<AppEvent> for KeyBlastApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // Create tray icon when the application is ready
        // On macOS, this must happen after the event loop starts
        if self._tray_icon.is_none() {
            println!("KeyBlast initializing...");

            // Check accessibility permission (macOS)
            // Detailed guidance is printed by the permission module if not granted
            let _ = permission::check_accessibility_permission();

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

            // Build menu with macros and create tray icon
            let (menu, menu_ids) = tray::build_menu(self.state.enabled, &final_config.macros);
            let tray_icon = tray::create_tray(&menu);

            self.menu = menu;
            self.menu_ids = menu_ids;
            self._tray_icon = Some(tray_icon);

            // Store icons for flash feedback
            self.normal_icon = Some(tray::load_icon());
            self.flash_icon = Some(tray::load_flash_icon());

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

            // Set up file watcher for hot-reload
            self.setup_config_watcher();

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
                                    // Trigger icon flash for visual feedback
                                    self.flash_remaining = 4; // 2 cycles of on/off
                                    self.flash_state = false;
                                    self.last_flash_toggle = Some(std::time::Instant::now());
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
        // Handle icon flash animation
        if self.flash_remaining > 0 {
            let should_toggle = self.last_flash_toggle
                .map(|t| t.elapsed() >= std::time::Duration::from_millis(100))
                .unwrap_or(true);

            if should_toggle {
                self.flash_state = !self.flash_state;
                self.flash_remaining -= 1;
                self.last_flash_toggle = Some(std::time::Instant::now());

                if let Some(ref tray_icon) = self._tray_icon {
                    let icon = if self.flash_state {
                        self.flash_icon.clone()
                    } else {
                        self.normal_icon.clone()
                    };
                    if let Some(i) = icon {
                        let _ = tray_icon.set_icon(Some(i));
                    }
                }
            }
        }

        // Check for config file changes (hot-reload)
        self.check_config_changes();

        // Process any pending menu events
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            // Check if this is a delete macro action (check before static IDs)
            if let Some(macro_name) = self.menu_ids.delete_macro_ids.get(&event.id) {
                let macro_name = macro_name.clone();
                println!("Deleting macro: {}", macro_name);

                if let Some(ref mut cfg) = self.config {
                    // Find and remove the macro by name
                    let original_len = cfg.macros.len();
                    cfg.macros.retain(|m| m.name != macro_name);

                    if cfg.macros.len() < original_len {
                        // Unregister the hotkey - find the binding
                        if let Some(ref mut manager) = self.hotkey_manager {
                            // Find the hotkey id for this macro
                            let mut id_to_remove = None;
                            for (&hotkey_id, binding) in self.macros.iter() {
                                if binding.name == macro_name {
                                    if let Some(hotkey) = config::parse_hotkey_string(&binding.hotkey) {
                                        let _ = manager.unregister(&hotkey);
                                    }
                                    id_to_remove = Some(hotkey_id);
                                    break;
                                }
                            }
                            // Remove after iteration
                            if let Some(id) = id_to_remove {
                                self.macros.remove(&id);
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
            } else if event.id == self.menu_ids.auto_start {
                // Toggle auto-start at login
                let currently_enabled = autostart::is_auto_start_enabled();
                match autostart::set_auto_start(!currently_enabled) {
                    Ok(()) => {
                        println!(
                            "Auto-start {}",
                            if !currently_enabled { "enabled" } else { "disabled" }
                        );
                        // Update the checkbox state in menu
                        for item in self.menu.items() {
                            if let muda::MenuItemKind::Check(check_item) = item {
                                if check_item.id() == &self.menu_ids.auto_start {
                                    check_item.set_checked(!currently_enabled);
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to toggle auto-start: {}", e);
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
