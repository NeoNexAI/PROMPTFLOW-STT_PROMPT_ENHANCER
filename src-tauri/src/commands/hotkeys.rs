use crate::error::AppError;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

fn show_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("overlay") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

/// Registers the enhance and dictate global shortcuts from their string forms
/// (e.g. `"CommandOrControl+Shift+E"`), replacing any previously registered
/// shortcuts. The handlers show the overlay and emit `hotkey://enhance` (with
/// the clipboard text) / `hotkey://dictate`.
///
/// Both strings are parsed *before* anything is unregistered, so an invalid
/// hotkey leaves the existing bindings untouched.
///
/// # Errors
/// [`AppError::Hotkey`] if either string fails to parse or OS registration fails.
pub fn register_pair(app: &AppHandle, enhance: &str, dictate: &str) -> Result<(), AppError> {
    use tauri_plugin_clipboard_manager::ClipboardExt;

    let enhance_sc = enhance
        .parse::<Shortcut>()
        .map_err(|e| AppError::Hotkey(format!("invalid enhance hotkey: {e}")))?;
    let dictate_sc = dictate
        .parse::<Shortcut>()
        .map_err(|e| AppError::Hotkey(format!("invalid dictate hotkey: {e}")))?;

    let gs = app.global_shortcut();
    gs.unregister_all().ok();

    gs.on_shortcut(enhance_sc, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let text = app.clipboard().read_text().unwrap_or_default();
            show_overlay(app);
            let _ = app.emit("hotkey://enhance", text);
        }
    })
    .map_err(|e| AppError::Hotkey(e.to_string()))?;

    gs.on_shortcut(dictate_sc, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            show_overlay(app);
            let _ = app.emit("hotkey://dictate", ());
        }
    })
    .map_err(|e| AppError::Hotkey(e.to_string()))?;

    Ok(())
}

/// Sets both global hotkeys at once. Called by the frontend on startup and
/// whenever the user edits a hotkey in Settings.
#[tauri::command]
pub async fn set_hotkeys(app: AppHandle, enhance: String, dictate: String) -> Result<(), AppError> {
    register_pair(&app, &enhance, &dictate)
}

#[cfg(test)]
mod tests {
    use tauri_plugin_global_shortcut::Shortcut;

    #[test]
    fn test_invalid_shortcut_returns_error() {
        assert!("not-a-real-shortcut!!!".parse::<Shortcut>().is_err());
    }

    #[test]
    fn test_valid_shortcuts_parse() {
        assert!("CommandOrControl+Shift+E".parse::<Shortcut>().is_ok());
        assert!("CommandOrControl+Shift+D".parse::<Shortcut>().is_ok());
        assert!("Alt+Space".parse::<Shortcut>().is_ok());
    }
}
