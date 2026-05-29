use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

// Note for v0.1 sprint: provider, stt_engine, selected_mode are String here for IPC simplicity.
// Before v0.1 ships, validate these against the known AIProvider/STTEngine/EnhancementMode values.
// See src/types/index.ts for the TypeScript contract.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub provider: String,
    pub stt_engine: String,
    pub selected_mode: String,
    pub privacy_mode: bool,
    pub hotkey_enhance: String,
    pub hotkey_dictate: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            provider: "openai".to_string(),
            stt_engine: "whisper_api".to_string(),
            selected_mode: "fix_grammar".to_string(),
            privacy_mode: false,
            hotkey_enhance: "CommandOrControl+Shift+E".to_string(),
            hotkey_dictate: "CommandOrControl+Shift+D".to_string(),
        }
    }
}

/// Returns the path to the settings JSON file inside the app data directory.
fn settings_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Storage(e.to_string()))?;
    Ok(dir.join("settings.json"))
}

/// Returns the current settings.
/// Falls back to [`Settings::default`] when no settings file exists yet.
// TODO(v0.1): add Settings::validate() to reject unknown provider/stt_engine/selected_mode
// strings at deserialization time so errors surface here, not deep in the enhancement layer.
#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, AppError> {
    let path = settings_path(&app)?;
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).map_err(|e| AppError::Storage(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(AppError::Storage(e.to_string())),
    }
}

/// Persists `settings` to disk as pretty-printed JSON.
/// Creates the parent directory if it does not exist.
#[tauri::command]
pub async fn set_settings(app: AppHandle, settings: Settings) -> Result<(), AppError> {
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Storage(e.to_string()))?;
    }
    let data =
        serde_json::to_string_pretty(&settings).map_err(|e| AppError::Storage(e.to_string()))?;
    std::fs::write(&path, data).map_err(|e| AppError::Storage(e.to_string()))
}
