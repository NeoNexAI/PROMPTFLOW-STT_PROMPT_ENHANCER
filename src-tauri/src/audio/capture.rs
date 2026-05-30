//! Microphone capture via `cpal`.
//!
//! A `cpal` input stream is `!Send`, so it cannot be stored in shared state or
//! moved across threads. We therefore own the stream on a dedicated recording
//! thread: [`Recorder::start`] spawns that thread, samples are downmixed to
//! mono and accumulated, and [`Recorder::stop`] signals the thread and receives
//! the captured [`Recording`] back over a channel. The captured sample rate is
//! whatever the input device provides; the WAV encoder records that rate, so no
//! resampling is required.
//!
//! Voice Activity Detection runs in the capture callback: once speech has been
//! heard and then `silence_duration_ms` of silence follows, the recorder
//! auto-stops and emits `stt://autostop` so the frontend can finalize the
//! transcription (exactly as it would on a manual stop).

use crate::audio::vad::VoiceActivityDetector;
use crate::error::AppError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Mono `f32` samples plus the sample rate they were captured at.
pub struct Recording {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// Accumulated audio plus VAD bookkeeping, shared with the capture callback.
#[derive(Default)]
struct CaptureState {
    samples: Vec<f32>,
    had_speech: bool,
    silence_run: u32,
}

/// A live recording session. Dropping it without calling [`Recorder::stop`]
/// signals the thread to stop and discards the audio.
pub struct Recorder {
    stop: Arc<AtomicBool>,
    result_rx: mpsc::Receiver<Result<Recording, AppError>>,
}

impl Recorder {
    /// Starts capturing from the default input device on a dedicated thread.
    /// On VAD-triggered auto-stop, `stt://autostop` is emitted via `app`.
    pub fn start(app: AppHandle) -> Result<Self, AppError> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = stop.clone();
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("promptflow-audio".to_string())
            .spawn(move || {
                let _ = result_tx.send(record_loop(stop_for_thread, app));
            })
            .map_err(|e| AppError::Stt(format!("failed to spawn audio thread: {e}")))?;

        Ok(Self { stop, result_rx })
    }

    /// Signals the recording thread to stop and returns the captured audio.
    /// Safe to call after a VAD auto-stop (the buffered result is returned).
    pub fn stop(self) -> Result<Recording, AppError> {
        self.stop.store(true, Ordering::SeqCst);
        self.result_rx
            .recv()
            .map_err(|e| AppError::Stt(format!("audio thread did not return: {e}")))?
    }
}

/// Updates capture state with one block of mono samples and flips `auto_stop`
/// once speech has been followed by enough silence.
fn feed(
    state: &Mutex<CaptureState>,
    vad: &VoiceActivityDetector,
    silence_threshold_samples: u32,
    auto_stop: &AtomicBool,
    block: &[f32],
) {
    let silent = vad.is_silence(block);
    let mut st = state.lock().expect("capture state poisoned");
    st.samples.extend_from_slice(block);
    if silent {
        if st.had_speech {
            st.silence_run += block.len() as u32;
            if st.silence_run >= silence_threshold_samples {
                auto_stop.store(true, Ordering::SeqCst);
            }
        }
    } else {
        st.had_speech = true;
        st.silence_run = 0;
    }
}

/// Runs entirely on the recording thread; owns the `!Send` stream for its whole
/// lifetime and returns the accumulated mono samples once stopped (manually or
/// by VAD).
fn record_loop(stop: Arc<AtomicBool>, app: AppHandle) -> Result<Recording, AppError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Permission("No microphone input device found".to_string()))?;
    let supported = device
        .default_input_config()
        .map_err(|e| AppError::Stt(format!("input config error: {e}")))?;

    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels().max(1) as usize;
    let vad = VoiceActivityDetector::new();
    let silence_threshold_samples = vad.silence_samples(sample_rate);
    let auto_stop = Arc::new(AtomicBool::new(false));
    let state: Arc<Mutex<CaptureState>> = Arc::new(Mutex::new(CaptureState::default()));
    let config: cpal::StreamConfig = supported.config();
    let err_fn = |err| {
        #[cfg(debug_assertions)]
        eprintln!("[PromptFlow] audio stream error: {err}");
        let _ = err;
    };

    macro_rules! build {
        ($sample:ty, $to_mono:expr) => {{
            let state = state.clone();
            let auto_stop = auto_stop.clone();
            device.build_input_stream(
                &config,
                move |data: &[$sample], _: &cpal::InputCallbackInfo| {
                    let mono: Vec<f32> = data.chunks(channels).map($to_mono).collect();
                    feed(&state, &vad, silence_threshold_samples, &auto_stop, &mono);
                },
                err_fn,
                None,
            )
        }};
    }

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => {
            build!(f32, |frame: &[f32]| frame.iter().copied().sum::<f32>()
                / frame.len() as f32)
        }
        cpal::SampleFormat::I16 => build!(i16, |frame: &[i16]| frame
            .iter()
            .map(|s| *s as f32 / i16::MAX as f32)
            .sum::<f32>()
            / frame.len() as f32),
        cpal::SampleFormat::U16 => build!(u16, |frame: &[u16]| frame
            .iter()
            .map(|s| (*s as f32 / u16::MAX as f32) * 2.0 - 1.0)
            .sum::<f32>()
            / frame.len() as f32),
        other => {
            return Err(AppError::Stt(format!(
                "unsupported input sample format: {other:?}"
            )))
        }
    }
    .map_err(|e| AppError::Stt(format!("failed to build input stream: {e}")))?;

    stream
        .play()
        .map_err(|e| AppError::Stt(format!("failed to start stream: {e}")))?;

    let mut auto_stopped = false;
    while !stop.load(Ordering::SeqCst) {
        if auto_stop.load(Ordering::SeqCst) {
            auto_stopped = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    drop(stream); // stop capture before reading the buffer

    if auto_stopped {
        // Tell the frontend to finalize (call stop_recording) just like a manual stop.
        let _ = app.emit("stt://autostop", ());
    }

    let samples = state.lock().map(|s| s.samples.clone()).unwrap_or_default();
    Ok(Recording {
        samples,
        sample_rate,
    })
}
