use crate::error::AppError;
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

/// Registers a global hotkey shortcut by its string representation.
///
/// When the shortcut is pressed, reads the current clipboard text and emits a
/// `hotkey://enhance` event with that text as the payload. If the clipboard
/// read fails the event is still emitted with an empty string, so the overlay
/// can open without crashing.
///
/// # Errors
/// Returns [`AppError::Hotkey`] if the shortcut string cannot be parsed or if
/// the OS-level registration fails (e.g. the shortcut is already taken by
/// another application).
#[tauri::command]
pub async fn register_hotkey(
    app: AppHandle,
    _id: String,
    shortcut: String,
) -> Result<(), AppError> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let sc = shortcut
        .parse::<tauri_plugin_global_shortcut::Shortcut>()
        .map_err(|e| AppError::Hotkey(e.to_string()))?;

    // Unregister first to make registration idempotent. Without this, calling
    // register_hotkey twice (e.g. via the Settings UI after the startup auto-register
    // in lib.rs) would stack two handlers and fire the event twice per keypress.
    app.global_shortcut().unregister(sc).ok();

    app.global_shortcut()
        .on_shortcut(sc, |app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                let text = app.clipboard().read_text().unwrap_or_default();
                let _ = app.emit("hotkey://enhance", text);
            }
        })
        .map_err(|e| AppError::Hotkey(e.to_string()))
}

/// Unregisters all previously registered global shortcuts.
///
/// For v0.1 the `id` parameter is accepted for API compatibility but ignored;
/// all shortcuts registered by this process are removed.
///
/// # Errors
/// Returns [`AppError::Hotkey`] if the OS-level unregistration fails.
#[tauri::command]
pub async fn unregister_hotkey(app: AppHandle, _id: String) -> Result<(), AppError> {
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| AppError::Hotkey(e.to_string()))
}

#[cfg(test)]
mod tests {
    use tauri_plugin_global_shortcut::Shortcut;

    /// An invalid shortcut string must fail at parse time so that
    /// `register_hotkey` never reaches the OS registration step.
    #[test]
    fn test_register_hotkey_invalid_shortcut_returns_error() {
        let result = "not-a-real-shortcut!!!".parse::<Shortcut>();
        assert!(
            result.is_err(),
            "Parsing an invalid shortcut string should return an error"
        );
    }

    /// A well-formed shortcut string must parse successfully.
    #[test]
    fn test_register_hotkey_valid_shortcut_parses_ok() {
        let result = "CommandOrControl+Shift+E".parse::<Shortcut>();
        assert!(
            result.is_ok(),
            "Parsing a valid shortcut string should succeed"
        );
    }
}
