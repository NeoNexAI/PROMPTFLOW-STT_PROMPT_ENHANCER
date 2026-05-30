//! Minimal WAV (PCM 16-bit, mono) encoder.
//!
//! STT engines that take an uploaded file (e.g. OpenAI Whisper) need the
//! captured `f32` samples wrapped in a container. A 44-byte canonical WAV
//! header plus 16-bit little-endian PCM samples is the most broadly accepted
//! format and is trivial to produce without a dependency.

/// Encodes mono `f32` samples (range roughly -1.0..=1.0) at `sample_rate` Hz
/// into a complete WAV byte stream (header + PCM16 data).
pub fn encode_wav_pcm16(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    const BITS_PER_SAMPLE: u16 = 16;
    const CHANNELS: u16 = 1;
    let block_align: u16 = CHANNELS * BITS_PER_SAMPLE / 8; // = 2
    let byte_rate: u32 = sample_rate * block_align as u32;
    let data_len: u32 = (samples.len() * 2) as u32;

    let mut buf = Vec::with_capacity(44 + data_len as usize);
    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_len).to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    // fmt chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // PCM fmt chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // audio format = PCM
    buf.extend_from_slice(&CHANNELS.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&BITS_PER_SAMPLE.to_le_bytes());
    // data chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// Naive linear-interpolation resampler to `to` Hz. Used by the local
/// whisper.cpp engine, which requires 16 kHz mono input. Returns the input
/// unchanged when the rates already match.
pub fn resample_linear(input: &[f32], from: u32, to: u32) -> Vec<f32> {
    if input.is_empty() || from == to || from == 0 || to == 0 {
        return input.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let out_len = ((input.len() as f64) * ratio).round() as usize;
    let last = input.len() - 1;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src = i as f64 / ratio;
        let idx = src.floor() as usize;
        let frac = (src - idx as f64) as f32;
        let a = input[idx.min(last)];
        let b = input[(idx + 1).min(last)];
        out.push(a + (b - a) * frac);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_identity_when_rates_match() {
        let s = vec![0.1, 0.2, 0.3];
        assert_eq!(resample_linear(&s, 16_000, 16_000), s);
    }

    #[test]
    fn test_resample_downsample_halves_length() {
        let s = vec![0.0; 100];
        let out = resample_linear(&s, 32_000, 16_000);
        assert_eq!(out.len(), 50);
    }

    #[test]
    fn test_resample_constant_signal_stays_constant() {
        let s = vec![0.5f32; 48];
        let out = resample_linear(&s, 48_000, 16_000);
        assert_eq!(out.len(), 16);
        assert!(out.iter().all(|v| (*v - 0.5).abs() < 1e-6));
    }

    #[test]
    fn test_header_and_length() {
        let wav = encode_wav_pcm16(&[0.0, 0.5, -0.5, 1.0], 16_000);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
        // 44-byte header + 2 bytes per sample * 4 samples
        assert_eq!(wav.len(), 44 + 8);
        // data chunk length field
        assert_eq!(u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]), 8);
        // sample rate field
        assert_eq!(
            u32::from_le_bytes([wav[24], wav[25], wav[26], wav[27]]),
            16_000
        );
    }

    #[test]
    fn test_empty_samples_is_just_header() {
        let wav = encode_wav_pcm16(&[], 44_100);
        assert_eq!(wav.len(), 44);
        assert_eq!(u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]), 0);
    }

    #[test]
    fn test_clamps_and_scales_full_scale() {
        // +1.0 -> i16::MAX ; values beyond range are clamped.
        let wav = encode_wav_pcm16(&[2.0], 8000);
        let sample = i16::from_le_bytes([wav[44], wav[45]]);
        assert_eq!(sample, i16::MAX);
    }
}
