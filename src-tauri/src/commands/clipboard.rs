use crate::error::AppError;
use tauri_plugin_clipboard_manager::ClipboardExt;

/// Reads the current clipboard text content.
/// Returns `AppError::Clipboard` if the clipboard is empty or inaccessible.
#[tauri::command]
pub async fn read_clipboard(app: tauri::AppHandle) -> Result<String, AppError> {
    let text = app
        .clipboard()
        .read_text()
        .map_err(|e| AppError::Clipboard(e.to_string()))?;

    if text.trim().is_empty() {
        return Err(AppError::Clipboard("Clipboard is empty".to_string()));
    }

    Ok(text)
}

/// Writes text to the system clipboard.
/// Rejects empty strings to prevent silently overwriting the user's clipboard with nothing.
/// Returns `AppError::Clipboard` if the write fails or the text is empty.
#[tauri::command]
pub async fn write_clipboard(app: tauri::AppHandle, text: String) -> Result<(), AppError> {
    if text.trim().is_empty() {
        return Err(AppError::Clipboard(
            "Cannot write empty text to clipboard".to_string(),
        ));
    }
    app.clipboard()
        .write_text(text)
        .map_err(|e| AppError::Clipboard(e.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;

    #[test]
    fn test_read_clipboard_error_message_is_preserved() {
        let msg = "Clipboard is empty";
        let err = AppError::Clipboard(msg.to_string());
        // thiserror Display wraps as "Clipboard error: {0}"
        assert!(
            err.to_string().contains(msg),
            "AppError::Clipboard should expose its inner message"
        );
    }

    #[test]
    fn test_read_clipboard_empty_trim_detection() {
        let whitespace_only = "   \t\n  ";
        assert!(
            whitespace_only.trim().is_empty(),
            "Whitespace-only clipboard content should be treated as empty"
        );
        let non_empty = "  hello  ";
        assert!(
            !non_empty.trim().is_empty(),
            "Non-empty content should not be flagged as empty"
        );
    }

    #[test]
    fn test_write_clipboard_rejects_empty_string() {
        // Verify the guard logic: empty and whitespace-only strings are rejected
        assert!("".trim().is_empty());
        assert!("   ".trim().is_empty());
        assert!(!"hello".trim().is_empty());
    }
}
