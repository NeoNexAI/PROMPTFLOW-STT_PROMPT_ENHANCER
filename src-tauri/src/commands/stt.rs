use crate::audio::capture::Recorder;
use crate::error::AppError;
use crate::storage::keychain::KeychainStore;
use crate::stt::{self, key_account};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{Emitter, State};

#[derive(Debug, Serialize, Deserialize)]
pub struct STTStatus {
    pub available: bool,
    pub reason: Option<String>,
}

/// Shared recording state managed by Tauri. Holds the active microphone
/// recorder (if any) and the engine that started it.
#[derive(Default)]
pub struct RecordingState {
    recorder: Option<Recorder>,
    engine: String,
}

/// Begins capturing audio from the microphone for the given backend STT engine.
///
/// `web_speech` is browser-only and must not reach this command. Any previously
/// running recording is discarded.
#[tauri::command]
pub async fn start_recording(
    app: tauri::AppHandle,
    engine: String,
    state: State<'_, Mutex<RecordingState>>,
) -> Result<(), AppError> {
    if !stt::is_backend_engine(&engine) {
        return Err(AppError::Stt(
            "web_speech is handled in the browser, not via start_recording".to_string(),
        ));
    }
    let recorder = Recorder::start(app)?;
    let mut guard = state
        .lock()
        .map_err(|_| AppError::Stt("recording state poisoned".to_string()))?;
    guard.recorder = Some(recorder);
    guard.engine = engine;
    Ok(())
}

/// Stops the active recording, transcribes the captured audio with the engine
/// that started it, emits `stt://done` with the transcript, and returns it.
#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle,
    state: State<'_, Mutex<RecordingState>>,
) -> Result<String, AppError> {
    // Take the recorder out under the lock, then release it before doing any
    // blocking/awaiting work (never hold a std Mutex across .await).
    let (recorder, engine) = {
        let mut guard = state
            .lock()
            .map_err(|_| AppError::Stt("recording state poisoned".to_string()))?;
        (guard.recorder.take(), std::mem::take(&mut guard.engine))
    };
    let recorder = recorder.ok_or_else(|| AppError::Stt("No active recording".to_string()))?;

    let recording = recorder.stop()?;

    let api_key = if stt::requires_api_key(&engine) {
        KeychainStore::new().get_api_key(key_account(&engine))?
    } else {
        None
    };
    // whisper.cpp needs its binary/model paths, sourced from saved settings.
    let settings = crate::commands::settings::load(&app)?;
    let config = stt::EngineConfig {
        api_key,
        binary_path: Some(settings.whisper_cpp_binary),
        model_path: Some(settings.whisper_cpp_model),
    };
    let stt_engine = stt::make_engine(&engine, config)?;
    let transcript = stt_engine
        .transcribe(recording.samples, recording.sample_rate)
        .await?;

    let _ = app.emit("stt://done", transcript.clone());
    Ok(transcript)
}

/// Reports whether the given STT engine is usable right now (key present for
/// engines that need one; `web_speech` is always available in the WebView).
#[tauri::command]
pub async fn check_stt_status(engine: String) -> Result<STTStatus, AppError> {
    let available = |reason: Option<&str>| STTStatus {
        available: reason.is_none(),
        reason: reason.map(str::to_string),
    };

    match engine.as_str() {
        "web_speech" => Ok(available(None)),
        "whisper_api" => {
            let has_key = KeychainStore::new()
                .has_api_key(key_account("whisper_api"))
                .unwrap_or(false);
            Ok(if has_key {
                available(None)
            } else {
                available(Some("No OpenAI API key — add one in Settings → API Keys"))
            })
        }
        other if stt::KNOWN_ENGINES.contains(&other) => {
            Ok(available(Some("This STT engine is not implemented yet")))
        }
        other => Err(AppError::Stt(format!("Unknown STT engine: {other}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recording_state_default_is_empty() {
        let s = RecordingState::default();
        assert!(s.recorder.is_none());
        assert!(s.engine.is_empty());
    }
}
