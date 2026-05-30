mod audio;
mod clipboard;
mod commands;
mod cost;
mod enhancement;
mod error;
mod hotkeys;
mod permissions;
mod providers;
mod storage;
mod stt;
mod telemetry;
mod updater;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::sync::Mutex;

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        // updater: disabled until a signing keypair exists — see docs/DECISIONS.md
        .manage(Mutex::new(commands::stt::RecordingState::default()))
        .setup(|app| {
            use tauri::Emitter;
            use tauri_plugin_clipboard_manager::ClipboardExt;
            use tauri_plugin_global_shortcut::{
                Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
            };

            // Use explicit Code+Modifiers instead of string parsing for reliability.
            let enhance_shortcut =
                Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);
            let dictate_shortcut =
                Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyD);

            #[cfg(debug_assertions)]
            eprintln!("[PromptFlow] Registering hotkeys Ctrl+Shift+E / Ctrl+Shift+D...");

            // Enhance: read the clipboard, show the overlay, emit hotkey://enhance.
            app.handle().global_shortcut().on_shortcut(
                enhance_shortcut,
                |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        use tauri::Manager;
                        let text = app.clipboard().read_text().unwrap_or_default();
                        if let Some(win) = app.get_webview_window("overlay") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                        let _ = app.emit("hotkey://enhance", text);
                    }
                },
            )?;

            // Dictate: show the overlay and emit hotkey://dictate (the frontend
            // starts/stops recording).
            app.handle().global_shortcut().on_shortcut(
                dictate_shortcut,
                |app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        use tauri::Manager;
                        if let Some(win) = app.get_webview_window("overlay") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                        let _ = app.emit("hotkey://dictate", ());
                    }
                },
            )?;

            #[cfg(debug_assertions)]
            eprintln!("[PromptFlow] Hotkeys registered OK");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::enhance::enhance_text,
            commands::stt::start_recording,
            commands::stt::stop_recording,
            commands::stt::check_stt_status,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::hotkeys::register_hotkey,
            commands::hotkeys::unregister_hotkey,
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            commands::api_key::save_api_key,
            commands::api_key::has_api_key,
            commands::api_key::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
