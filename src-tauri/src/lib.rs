mod audio;
mod clipboard;
mod commands;
mod cost;
mod enhancement;
mod error;
mod hotkeys;
mod http;
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
            // Register the default hotkeys at startup so they work before the
            // frontend mounts. The frontend re-applies the user's saved hotkeys
            // via the `set_hotkeys` command on launch and on every change.
            #[cfg(debug_assertions)]
            eprintln!("[PromptFlow] Registering default hotkeys Ctrl+Shift+E / Ctrl+Shift+D...");
            if let Err(e) = commands::hotkeys::register_pair(
                app.handle(),
                "CommandOrControl+Shift+E",
                "CommandOrControl+Shift+D",
            ) {
                eprintln!("[PromptFlow] default hotkey registration failed: {e:?}");
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::enhance::enhance_text,
            commands::stt::start_recording,
            commands::stt::stop_recording,
            commands::stt::check_stt_status,
            commands::settings::get_settings,
            commands::settings::set_settings,
            commands::hotkeys::set_hotkeys,
            commands::clipboard::read_clipboard,
            commands::clipboard::write_clipboard,
            commands::api_key::save_api_key,
            commands::api_key::has_api_key,
            commands::api_key::delete_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
