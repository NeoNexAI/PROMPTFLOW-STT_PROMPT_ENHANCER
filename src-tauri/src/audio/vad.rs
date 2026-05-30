/// Voice Activity Detection — used to auto-stop recording after a period of
/// silence (energy-based, see `audio/capture.rs`).
#[derive(Debug, Clone, Copy)]
pub struct VoiceActivityDetector {
    pub silence_threshold: f32,
    pub silence_duration_ms: u32,
}

impl VoiceActivityDetector {
    pub fn new() -> Self {
        Self {
            silence_threshold: 0.01,
            silence_duration_ms: 1500,
        }
    }

    /// Returns true if the audio chunk's mean absolute amplitude is below the
    /// silence threshold. Empty chunks count as silence.
    pub fn is_silence(&self, chunk: &[f32]) -> bool {
        if chunk.is_empty() {
            return true;
        }
        let energy = chunk.iter().map(|s| s.abs()).sum::<f32>() / chunk.len() as f32;
        energy < self.silence_threshold
    }

    /// Number of samples that constitute `silence_duration_ms` at `sample_rate`.
    pub fn silence_samples(&self, sample_rate: u32) -> u32 {
        (sample_rate as u64 * self.silence_duration_ms as u64 / 1000) as u32
    }
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_for_quiet_and_empty() {
        let vad = VoiceActivityDetector::new();
        assert!(vad.is_silence(&[]));
        assert!(vad.is_silence(&[0.0, 0.001, -0.002]));
    }

    #[test]
    fn test_not_silence_for_loud() {
        let vad = VoiceActivityDetector::new();
        assert!(!vad.is_silence(&[0.5, -0.4, 0.6]));
    }

    #[test]
    fn test_silence_samples_scales_with_rate() {
        let vad = VoiceActivityDetector::new(); // 1500 ms
        assert_eq!(vad.silence_samples(16_000), 24_000);
        assert_eq!(vad.silence_samples(48_000), 72_000);
    }
}
