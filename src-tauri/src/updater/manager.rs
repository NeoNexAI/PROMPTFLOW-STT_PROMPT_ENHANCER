/// Auto-update via tauri-plugin-updater
pub struct UpdaterManager;

impl UpdaterManager {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UpdaterManager {
    fn default() -> Self {
        Self::new()
    }
}
