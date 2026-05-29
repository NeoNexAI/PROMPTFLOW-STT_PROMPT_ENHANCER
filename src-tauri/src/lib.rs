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
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        // updater: stub for future sprints — plugin requires config; skip for v0.1
        .setup(|app| {
            use tauri::Emitter;
            use tauri_plugin_clipboard_manager::ClipboardExt;
            use tauri_plugin_global_shortcut::{
                Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
            };

            // Use explicit Code+Modifiers instead of string parsing for reliability
            let shortcut = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::KeyE);

            #[cfg(debug_assertions)]
            eprintln!("[PromptFlow] Registering hotkey Ctrl+Shift+E...");
            app.handle()
                .global_shortcut()
                .on_shortcut(shortcut, |app, _shortcut, event| {
                    #[cfg(debug_assertions)]
                    eprintln!("[PromptFlow] Hotkey fired! state={:?}", event.state);
                    if event.state == ShortcutState::Pressed {
                        use tauri::Manager;
                        let text = app.clipboard().read_text().unwrap_or_default();
                        #[cfg(debug_assertions)]
                        eprintln!(
                            "[PromptFlow] Clipboard text: {:?}",
                            &text[..text.len().min(50)]
                        );
                        // Show the window from Rust — don't rely on JS win.show()
                        if let Some(win) = app.get_webview_window("overlay") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                        let _ = app.emit("hotkey://enhance", text);
                    }
                })?;
            #[cfg(debug_assertions)]
            eprintln!("[PromptFlow] Hotkey registered OK");

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
