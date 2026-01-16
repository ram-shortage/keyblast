/// Application state management for KeyBlast.
///
/// Tracks whether macro playback is enabled or disabled.

pub struct AppState {
    pub enabled: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
