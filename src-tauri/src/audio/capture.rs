//! Microphone capture via `cpal`.
//!
//! A `cpal` input stream is `!Send`, so it cannot be stored in shared state or
//! moved across threads. We therefore own the stream on a dedicated recording
//! thread: [`Recorder::start`] spawns that thread, samples are downmixed to
//! mono and accumulated, and [`Recorder::stop`] signals the thread and receives
//! the captured [`Recording`] back over a channel. The captured sample rate is
//! whatever the input device provides; the WAV encoder records that rate, so no
//! resampling is required.

use crate::error::AppError;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Mono `f32` samples plus the sample rate they were captured at.
pub struct Recording {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}

/// A live recording session. Dropping it without calling [`Recorder::stop`]
/// signals the thread to stop and discards the audio.
pub struct Recorder {
    stop: Arc<AtomicBool>,
    result_rx: mpsc::Receiver<Result<Recording, AppError>>,
}

impl Recorder {
    /// Starts capturing from the default input device on a dedicated thread.
    pub fn start() -> Result<Self, AppError> {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_for_thread = stop.clone();
        let (result_tx, result_rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("promptflow-audio".to_string())
            .spawn(move || {
                let _ = result_tx.send(record_loop(stop_for_thread));
            })
            .map_err(|e| AppError::Stt(format!("failed to spawn audio thread: {e}")))?;

        Ok(Self { stop, result_rx })
    }

    /// Signals the recording thread to stop and returns the captured audio.
    pub fn stop(self) -> Result<Recording, AppError> {
        self.stop.store(true, Ordering::SeqCst);
        self.result_rx
            .recv()
            .map_err(|e| AppError::Stt(format!("audio thread did not return: {e}")))?
    }
}

/// Runs entirely on the recording thread; owns the `!Send` stream for its whole
/// lifetime and returns the accumulated mono samples once `stop` is set.
fn record_loop(stop: Arc<AtomicBool>) -> Result<Recording, AppError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| AppError::Permission("No microphone input device found".to_string()))?;
    let supported = device
        .default_input_config()
        .map_err(|e| AppError::Stt(format!("input config error: {e}")))?;

    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels().max(1) as usize;
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let config: cpal::StreamConfig = supported.config();
    let err_fn = |err| {
        #[cfg(debug_assertions)]
        eprintln!("[PromptFlow] audio stream error: {err}");
        let _ = err;
    };

    let stream = match supported.sample_format() {
        cpal::SampleFormat::F32 => {
            let buf = buffer.clone();
            device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut b = buf.lock().expect("audio buffer poisoned");
                    for frame in data.chunks(channels) {
                        b.push(frame.iter().copied().sum::<f32>() / frame.len() as f32);
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let buf = buffer.clone();
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    let mut b = buf.lock().expect("audio buffer poisoned");
                    for frame in data.chunks(channels) {
                        let avg = frame
                            .iter()
                            .map(|s| *s as f32 / i16::MAX as f32)
                            .sum::<f32>()
                            / frame.len() as f32;
                        b.push(avg);
                    }
                },
                err_fn,
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let buf = buffer.clone();
            device.build_input_stream(
                &config,
                move |data: &[u16], _: &cpal::InputCallbackInfo| {
                    let mut b = buf.lock().expect("audio buffer poisoned");
                    for frame in data.chunks(channels) {
                        let avg = frame
                            .iter()
                            .map(|s| (*s as f32 / u16::MAX as f32) * 2.0 - 1.0)
                            .sum::<f32>()
                            / frame.len() as f32;
                        b.push(avg);
                    }
                },
                err_fn,
                None,
            )
        }
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

    while !stop.load(Ordering::SeqCst) {
        std::thread::sleep(Duration::from_millis(50));
    }
    drop(stream); // stop capture before reading the buffer

    let samples = buffer.lock().map(|b| b.clone()).unwrap_or_default();
    Ok(Recording {
        samples,
        sample_rate,
    })
}
