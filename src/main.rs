#![windows_subsystem = "windows"]

/// KeyBlast - A lightweight macro playback application.
///
/// Sits in the system tray and provides hotkey-triggered keystroke injection.

mod app;
mod autostart;
mod config;
mod execution;
mod hotkey;
mod injection;
mod logging;
mod notification;
mod permission;
mod tray;

use std::collections::HashMap;
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
use crossbeam_channel;
use tracing::{info, debug, error};

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
    /// Active execution handle (if macro running)
    active_execution: Option<execution::ExecutionHandle>,
    /// Receiver for execution commands from worker thread
    execution_rx: Option<crossbeam_channel::Receiver<execution::ExecutionCommand>>,
    /// Whether we've prepared the injector for this execution run
    execution_prepared: bool,
    /// ID of the stop macro hotkey (Ctrl+Escape)
    stop_hotkey_id: Option<u32>,
    /// Validation warnings from config load
    config_warnings: Vec<config::ValidationWarning>,
    /// Flag to signal clean shutdown
    should_exit: bool,
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
                open_logs: muda::MenuId::new(""),
                auto_start: muda::MenuId::new(""),
                stop_macro: muda::MenuId::new(""),
                quit: muda::MenuId::new(""),
                delete_macro_ids: std::collections::HashMap::new(),
                run_macro_ids: std::collections::HashMap::new(),
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
            active_execution: None,
            execution_rx: None,
            execution_prepared: false,
            stop_hotkey_id: None,
            config_warnings: Vec::new(),
            should_exit: false,
        }
    }

    /// Rebuild the tray menu with current macros.
    /// Call after config changes (import, delete).
    fn rebuild_menu(&mut self) {
        if let Some(ref config) = self.config {
            let (menu, menu_ids) = tray::build_menu(
                self.state.enabled,
                &config.macros,
                &self.config_warnings,
            );

            // Update the tray icon's menu
            if let Some(ref tray_icon) = self._tray_icon {
                tray_icon.set_menu(Some(Box::new(menu.clone())));
            }

            self.menu = menu;
            self.menu_ids = menu_ids;
        }
    }

    /// Set up file watcher for config hot-reload.
    ///
    /// Watches the parent directory to catch rename/create events from editors
    /// that use atomic save (write temp file, then rename).
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
                // Watch parent directory to catch rename/create events
                if let Some(parent) = config_path.parent() {
                    if let Err(e) = watcher.watch(parent, RecursiveMode::NonRecursive) {
                        eprintln!("Failed to watch config directory: {}", e);
                    } else {
                        println!("Watching config directory for changes: {}", parent.display());
                        self.config_watcher = Some(watcher);
                        self.config_change_rx = Some(rx);
                    }
                } else {
                    eprintln!("Could not determine config file parent directory");
                }
            }
            Err(e) => {
                eprintln!("Failed to create config watcher: {}", e);
            }
        }
    }

    /// Check for config file changes (non-blocking).
    fn check_config_changes(&mut self) {
        let config_path = config::config_path();
        // Collect any relevant events first (to avoid borrow issues)
        let mut should_reload = false;
        if let Some(ref rx) = self.config_change_rx {
            // Non-blocking receive - check if there are any pending events
            while let Ok(result) = rx.try_recv() {
                if let Ok(event) = result {
                    // Check if event affects our config file
                    let affects_config = event.paths.iter().any(|p| p == &config_path);
                    if !affects_config {
                        continue;
                    }
                    // Reload on modify, create, or rename events (editors use atomic save)
                    // Reset to defaults on file deletion
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) => {
                            should_reload = true;
                        }
                        EventKind::Remove(_) => {
                            should_reload = true; // Will trigger reload which handles missing file
                        }
                        _ => {}
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

                // Validate and store warnings
                let warnings = config::validate_config(&new_config);
                for warning in &warnings {
                    eprintln!("Config warning: {}", warning);
                }
                self.config_warnings = warnings;

                // Apply settings from config file (sync enabled state)
                self.state.enabled = new_config.settings.enabled;

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
            info!("KeyBlast initializing...");

            // Check accessibility permission (macOS)
            // Detailed guidance is printed by the permission module if not granted
            let _ = permission::check_accessibility_permission();

            // Initialize keystroke injector
            match injection::KeystrokeInjector::new() {
                Ok(inj) => {
                    info!("Keystroke injector initialized");
                    self.injector = Some(inj);
                }
                Err(e) => {
                    error!("Failed to initialize keystroke injector: {}", e);
                    notification::show_error(
                        "KeyBlast",
                        notification::permission_error_message(),
                        notification::NotificationSeverity::Permission,
                    );
                }
            }

            // Load configuration from disk
            let loaded_config = match config::load_config() {
                Ok(cfg) => {
                    let config_path = config::config_path();
                    if config_path.exists() {
                        info!("Config loaded from: {}", config_path.display());
                    }
                    cfg
                }
                Err(e) => {
                    error!("Failed to load config: {}. Using defaults.", e);
                    config::Config::default()
                }
            };

            // If config has no macros, create a default example macro and save it
            let final_config = if loaded_config.macros.is_empty() {
                let default_macro = config::MacroDefinition {
                    id: uuid::Uuid::new_v4(),
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

            // Validate config and store warnings
            let warnings = config::validate_config(&final_config);
            for warning in &warnings {
                eprintln!("Config warning: {}", warning);
            }
            self.config_warnings = warnings;
            self.config = Some(final_config.clone());

            // Load enabled state from config (before build_menu so menu shows correct state)
            self.state.enabled = final_config.settings.enabled;

            // Build menu with macros and create tray icon
            let (menu, menu_ids) = tray::build_menu(
                self.state.enabled,
                &final_config.macros,
                &self.config_warnings,
            );
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
                                        debug!(
                                            "Registered macro: {} ({})",
                                            macro_def.name, macro_def.hotkey
                                        );
                                    }
                                    Err(e) => {
                                        error!(
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

                    // Register stop hotkey (Ctrl+Escape on all platforms)
                    use global_hotkey::hotkey::{HotKey, Code, Modifiers};
                    let stop_hotkey = HotKey::new(Some(Modifiers::CONTROL), Code::Escape);
                    match manager.register_raw(stop_hotkey) {
                        Ok(()) => {
                            self.stop_hotkey_id = Some(stop_hotkey.id());
                            println!("Stop hotkey registered: Ctrl+Escape");
                        }
                        Err(e) => {
                            eprintln!("Failed to register stop hotkey: {}", e);
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
                    // Check for stop hotkey
                    if Some(hotkey_event.id) == self.stop_hotkey_id {
                        if let Some(ref handle) = self.active_execution {
                            handle.stop();
                            println!("Stop hotkey pressed - macro will stop");
                        }
                        return;
                    }

                    // Look up macro by hotkey_id
                    if let Some(macro_def) = self.macros.get(&hotkey_event.id) {
                        println!("Hotkey triggered: {}", macro_def.name);

                        // Check if macros are enabled
                        if !self.state.enabled {
                            println!("Macros disabled, ignoring hotkey");
                            return;
                        }

                        // Check if already executing
                        if self.active_execution.is_some() {
                            println!("Macro already running, ignoring new trigger");
                            return;
                        }

                        // Inject the macro text using async execution
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

                            let has_delay = segments.iter().any(|s| matches!(s, injection::MacroSegment::Delay(_)));
                            if macro_def.delay_ms == 0 && segments.len() <= 10 && !has_delay {
                                // Fast path: short macros with no delay run synchronously
                                // This avoids overhead for simple text expansion
                                match injector.execute_sequence(&segments, 0) {
                                    Ok(()) => {
                                        println!("Injection complete");
                                        self.flash_remaining = 4;
                                        self.flash_state = false;
                                        self.last_flash_toggle = Some(std::time::Instant::now());
                                    }
                                    Err(e) => {
                                        eprintln!("Injection failed: {}", e);
                                        notification::show_error(
                                            "KeyBlast",
                                            "Macro injection failed",
                                            notification::NotificationSeverity::InjectionFailed,
                                        );
                                    }
                                }
                            } else {
                                // Async path: spawn worker thread for long or delayed macros
                                let (rx, handle) = execution::start_execution(segments, macro_def.delay_ms);
                                self.execution_rx = Some(rx);
                                self.active_execution = Some(handle);
                                self.execution_prepared = false;
                                // Flash happens when Complete command received
                            }
                        } else {
                            eprintln!("No injector available");
                        }
                    }
                }
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Check for clean shutdown request
        if self.should_exit {
            event_loop.exit();
            return;
        }

        // Process async execution commands (non-blocking)
        // Collect commands first to avoid borrow issues when clearing state
        let commands: Vec<_> = self.execution_rx.as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();

        let mut injection_failed = false;
        for cmd in commands {
            match cmd {
                execution::ExecutionCommand::Inject(segment) => {
                    if let Some(ref mut injector) = self.injector {
                        // Prepare injector once at start of execution
                        if !self.execution_prepared {
                            if let Err(e) = injector.prepare_for_injection() {
                                eprintln!("Failed to prepare injection: {}", e);
                                notification::show_error(
                                    "KeyBlast",
                                    &format!("Failed to prepare injection: {}", e),
                                    notification::NotificationSeverity::InjectionFailed,
                                );
                                injection_failed = true;
                                break;
                            }
                            self.execution_prepared = true;
                        }
                        // Execute segment on main thread (safe for macOS TIS/TSM)
                        if let Err(e) = injector.execute_single_segment(&segment) {
                            eprintln!("Injection error: {}", e);
                            notification::show_error(
                                "KeyBlast",
                                "Macro injection failed",
                                notification::NotificationSeverity::InjectionFailed,
                            );
                            injection_failed = true;
                            break;
                        }
                    }
                }
                execution::ExecutionCommand::Complete => {
                    println!("Macro execution complete");
                    self.active_execution = None;
                    self.execution_rx = None;
                    self.execution_prepared = false;
                    // Trigger icon flash AFTER completion
                    self.flash_remaining = 4;
                    self.flash_state = false;
                    self.last_flash_toggle = Some(std::time::Instant::now());
                }
                execution::ExecutionCommand::Cancelled => {
                    println!("Macro execution cancelled");
                    self.active_execution = None;
                    self.execution_rx = None;
                    self.execution_prepared = false;
                    // No flash on cancel - user knows they cancelled
                }
            }
        }

        // Handle injection failure: stop execution and clean up
        if injection_failed {
            if let Some(ref handle) = self.active_execution {
                handle.stop();
            }
            self.active_execution = None;
            self.execution_rx = None;
            self.execution_prepared = false;
        }

        // Update Stop Macro menu item enabled state
        let is_running = self.active_execution.is_some();
        for item in self.menu.items() {
            if let muda::MenuItemKind::MenuItem(normal_item) = item {
                if normal_item.id() == &self.menu_ids.stop_macro {
                    normal_item.set_enabled(is_running);
                    break;
                }
            }
        }

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
            // Check if this is a run macro action (check before delete and static IDs)
            if let Some(macro_id) = self.menu_ids.run_macro_ids.get(&event.id) {
                let macro_id = *macro_id;

                // Find the macro definition by UUID
                let macro_def = self.config.as_ref()
                    .and_then(|cfg| cfg.macros.iter().find(|m| m.id == macro_id))
                    .cloned();

                if let Some(macro_def) = macro_def {
                    // Check if macros are enabled
                    if !self.state.enabled {
                        println!("Macros disabled, ignoring run request");
                        continue;
                    }

                    // Check if already executing
                    if self.active_execution.is_some() {
                        println!("Macro already running, ignoring new trigger");
                        continue;
                    }

                    // Trigger execution (same logic as hotkey trigger)
                    if let Some(ref mut injector) = self.injector {
                        let segments = injection::parse_macro_sequence(&macro_def.text);
                        println!("Running macro '{}' from menu", macro_def.name);

                        let has_delay = segments.iter().any(|s| matches!(s, injection::MacroSegment::Delay(_)));
                        if macro_def.delay_ms == 0 && segments.len() <= 10 && !has_delay {
                            // Fast path: short macros with no delay
                            match injector.execute_sequence(&segments, 0) {
                                Ok(()) => {
                                    println!("Injection complete");
                                    self.flash_remaining = 4;
                                    self.flash_state = false;
                                    self.last_flash_toggle = Some(std::time::Instant::now());
                                }
                                Err(e) => {
                                    eprintln!("Injection failed: {}", e);
                                    notification::show_error(
                                        "KeyBlast",
                                        "Macro injection failed",
                                        notification::NotificationSeverity::InjectionFailed,
                                    );
                                }
                            }
                        } else {
                            // Async path
                            let (rx, handle) = execution::start_execution(segments, macro_def.delay_ms);
                            self.execution_rx = Some(rx);
                            self.active_execution = Some(handle);
                            self.execution_prepared = false;
                        }
                    }
                }
                continue;
            }

            // Check if this is a delete macro action (check before static IDs)
            if let Some(macro_id) = self.menu_ids.delete_macro_ids.get(&event.id) {
                let macro_id = *macro_id; // Copy the UUID
                println!("Deleting macro with ID: {}", macro_id);

                if let Some(ref mut cfg) = self.config {
                    // Find and remove the macro by UUID
                    let original_len = cfg.macros.len();
                    cfg.macros.retain(|m| m.id != macro_id);

                    if cfg.macros.len() < original_len {
                        // Find and unregister the hotkey
                        if let Some(ref mut manager) = self.hotkey_manager {
                            let mut id_to_remove = None;
                            for (&hotkey_id, binding) in self.macros.iter() {
                                if binding.id == macro_id {
                                    if let Some(hotkey) = config::parse_hotkey_string(&binding.hotkey) {
                                        let _ = manager.unregister(&hotkey);
                                    }
                                    id_to_remove = Some(hotkey_id);
                                    break;
                                }
                            }
                            if let Some(id) = id_to_remove {
                                self.macros.remove(&id);
                            }
                        }

                        // Re-validate after deletion
                        self.config_warnings = config::validate_config(cfg);

                        // Save updated config
                        match config::save_config(cfg) {
                            Ok(()) => {
                                println!("Macro deleted and config saved");
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

                // Save enabled state to config immediately
                if let Some(ref mut cfg) = self.config {
                    cfg.settings.enabled = self.state.enabled;
                    if let Err(e) = config::save_config(cfg) {
                        eprintln!("Failed to save enabled state: {}", e);
                    }
                }

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
                                let mut existing_names: std::collections::HashSet<_> =
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
                                                        // Track this name to prevent duplicates within import
                                                        existing_names.insert(macro_def.name.clone());
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
            } else if event.id == self.menu_ids.open_logs {
                // Open logs directory in system file browser
                logging::open_logs_directory();
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
            } else if event.id == self.menu_ids.stop_macro {
                if let Some(ref handle) = self.active_execution {
                    handle.stop();
                    println!("Stop menu clicked - macro will stop");
                }
            } else if event.id == self.menu_ids.quit {
                // Clean up active execution if running
                if let Some(handle) = self.active_execution.take() {
                    handle.stop();
                    handle.join();
                }
                println!("KeyBlast shutting down.");
                // Set flag for clean exit (allows destructors to run for log flushing)
                self.should_exit = true;
            }
        }
    }
}

fn main() {
    // Initialize file logging BEFORE event loop creation
    // Keep guard alive for program lifetime
    let _log_guard = logging::init_file_logging();

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
