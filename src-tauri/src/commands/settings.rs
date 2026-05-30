use crate::error::AppError;
use crate::providers::KNOWN_PROVIDERS;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

/// Known STT engine ids (mirrors `STTEngine` in `src/types/index.ts`).
const KNOWN_STT_ENGINES: &[&str] = &[
    "whisper_api",
    "whisper_cpp",
    "deepgram",
    "assembly_ai",
    "google_stt",
    "azure_stt",
    "web_speech",
];

/// Known enhancement mode ids (mirrors `EnhancementMode` in `src/types/index.ts`).
const KNOWN_MODES: &[&str] = &[
    "fix_grammar",
    "formalize",
    "shorten",
    "expand",
    "translate",
    "brainstorm",
    "action_items",
    "summarize",
    "code_review",
    "simplify",
    "reframe",
    "custom",
];

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub provider: String,
    pub stt_engine: String,
    pub selected_mode: String,
    pub privacy_mode: bool,
    pub hotkey_enhance: String,
    pub hotkey_dictate: String,
    // Local whisper.cpp paths (optional). `#[serde(default)]` keeps older
    // settings.json files (written before these existed) loadable.
    #[serde(default)]
    pub whisper_cpp_binary: String,
    #[serde(default)]
    pub whisper_cpp_model: String,
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
            whisper_cpp_binary: String::new(),
            whisper_cpp_model: String::new(),
        }
    }
}

impl Settings {
    /// Validates that enum-like fields hold known values and hotkeys are
    /// non-empty, so bad input is rejected at the boundary rather than deep in
    /// the enhancement layer.
    pub fn validate(&self) -> Result<(), AppError> {
        if !KNOWN_PROVIDERS.contains(&self.provider.as_str()) {
            return Err(AppError::Storage(format!(
                "Unknown provider: {}",
                self.provider
            )));
        }
        if !KNOWN_STT_ENGINES.contains(&self.stt_engine.as_str()) {
            return Err(AppError::Storage(format!(
                "Unknown STT engine: {}",
                self.stt_engine
            )));
        }
        if !KNOWN_MODES.contains(&self.selected_mode.as_str()) {
            return Err(AppError::Storage(format!(
                "Unknown enhancement mode: {}",
                self.selected_mode
            )));
        }
        if self.hotkey_enhance.trim().is_empty() || self.hotkey_dictate.trim().is_empty() {
            return Err(AppError::Storage("Hotkeys must not be empty".to_string()));
        }
        Ok(())
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

/// Loads the current settings synchronously (reusable from other commands).
/// Falls back to [`Settings::default`] when no settings file exists yet.
pub fn load(app: &AppHandle) -> Result<Settings, AppError> {
    let path = settings_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(data) => serde_json::from_str(&data).map_err(|e| AppError::Storage(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(AppError::Storage(e.to_string())),
    }
}

/// Returns the current settings.
#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<Settings, AppError> {
    load(&app)
}

/// Persists `settings` to disk as pretty-printed JSON after validation.
/// Creates the parent directory if it does not exist.
#[tauri::command]
pub async fn set_settings(app: AppHandle, settings: Settings) -> Result<(), AppError> {
    settings.validate()?;
    let path = settings_path(&app)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Storage(e.to_string()))?;
    }
    let data =
        serde_json::to_string_pretty(&settings).map_err(|e| AppError::Storage(e.to_string()))?;
    std::fs::write(&path, data).map_err(|e| AppError::Storage(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_are_valid() {
        assert!(Settings::default().validate().is_ok());
    }

    #[test]
    fn test_unknown_provider_rejected() {
        let s = Settings {
            provider: "skynet".into(),
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_empty_hotkey_rejected() {
        let s = Settings {
            hotkey_enhance: "  ".into(),
            ..Default::default()
        };
        assert!(s.validate().is_err());
    }

    #[test]
    fn test_all_known_modes_valid() {
        for m in KNOWN_MODES {
            let s = Settings {
                selected_mode: (*m).to_string(),
                ..Default::default()
            };
            assert!(s.validate().is_ok(), "{m} should be valid");
        }
    }
}
