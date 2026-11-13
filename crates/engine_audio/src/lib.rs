//! Engine-owned audio clip loading and output playback.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Source};
use thiserror::Error;

/// A fully decoded, reusable in-memory audio clip.
#[derive(Clone, Debug)]
pub struct AudioClip {
    buffer: SamplesBuffer,
    duration: Duration,
    frames: usize,
}

impl AudioClip {
    /// Loads and validates one audio file without opening an output device.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, AudioError> {
        let path = path.as_ref();
        let file = File::open(path).map_err(|source| AudioError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let decoder = Decoder::try_from(file).map_err(|source| AudioError::Decode {
            path: path.to_path_buf(),
            source,
        })?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let samples = decoder.collect::<Vec<_>>();
        if samples.is_empty() {
            return Err(AudioError::Empty(path.to_path_buf()));
        }
        if samples.len() % channels.get() as usize != 0
            || samples.iter().any(|sample| !sample.is_finite())
        {
            return Err(AudioError::InvalidSamples(path.to_path_buf()));
        }
        let frames = samples.len() / channels.get() as usize;
        let duration = Duration::from_secs_f64(frames as f64 / sample_rate.get() as f64);
        Ok(Self {
            buffer: SamplesBuffer::new(channels, sample_rate, samples),
            duration,
            frames,
        })
    }

    pub fn duration(&self) -> Duration {
        self.duration
    }

    pub fn frames(&self) -> usize {
        self.frames
    }
}

/// Output-stream properties exposed without leaking the audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AudioOutputInfo {
    pub channels: u16,
    pub sample_rate: u32,
}

/// Owns the operating-system output stream and its mixer of active voices.
pub struct AudioSystem {
    output: MixerDeviceSink,
}

impl AudioSystem {
    /// Opens the default output device, falling back to another available output if necessary.
    pub fn new() -> Result<Self, AudioError> {
        let mut output = DeviceSinkBuilder::open_default_sink()
            .map_err(|error| AudioError::Output(error.to_string()))?;
        output.log_on_drop(false);
        Ok(Self { output })
    }

    pub fn output_info(&self) -> AudioOutputInfo {
        AudioOutputInfo {
            channels: self.output.config().channel_count().get(),
            sample_rate: self.output.config().sample_rate().get(),
        }
    }

    /// Queues one independent playback of a previously validated clip.
    pub fn play(&self, clip: &AudioClip) {
        self.output.mixer().add(clip.buffer.clone());
    }
}

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("failed to open an audio output device: {0}")]
    Output(String),
    #[error("failed to read audio clip {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to decode audio clip {path}: {source}")]
    Decode {
        path: PathBuf,
        #[source]
        source: rodio::decoder::DecoderError,
    },
    #[error("audio clip {0} contains no samples")]
    Empty(PathBuf),
    #[error("audio clip {0} contains incomplete frames or non-finite samples")]
    InvalidSamples(PathBuf),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn asset_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/audio/footstep.wav")
    }

    #[test]
    fn wav_clip_decodes_and_can_be_replayed() {
        let clip = AudioClip::load(asset_path()).unwrap();
        assert!(clip.frames() > 1_000);
        assert!(clip.duration() > Duration::from_millis(50));
        let first = clip.buffer.clone().take(8).collect::<Vec<_>>();
        let second = clip.buffer.clone().take(8).collect::<Vec<_>>();
        assert_eq!(first, second);
        assert!(first.iter().any(|sample| sample.abs() > 0.0));
    }

    #[test]
    fn missing_clip_has_path_context() {
        let path = asset_path().with_file_name("does-not-exist.wav");
        let error = AudioClip::load(&path).unwrap_err();
        assert!(error.to_string().contains("does-not-exist.wav"));
    }

    #[test]
    fn invalid_clip_has_path_context() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../artifacts/test-audio");
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("invalid-{}.wav", std::process::id()));
        fs::write(&path, b"not a wave file").unwrap();

        let error = AudioClip::load(&path).unwrap_err();
        assert!(matches!(error, AudioError::Decode { .. }));
        assert!(error.to_string().contains("invalid-"));
        fs::remove_file(path).unwrap();
    }
}
