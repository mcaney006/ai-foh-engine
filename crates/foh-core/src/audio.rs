use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug)]
pub enum AudioError {
    Io(String),
    Format(String),
    Empty,
    Channels,
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::Io(s) | AudioError::Format(s) => write!(f, "{s}"),
            AudioError::Empty => write!(f, "empty buffer"),
            AudioError::Channels => write!(f, "channel count mismatch"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelRole {
    Kick,
    Bass,
    Snare,
    Hats,
    Tom,
    Guitar,
    Keys,
    Vocal,
    VocalBv,
    DrumOh,
    Playback,
    Talkback,
    FxReturn,
    DrumBus,
    VocalBus,
    MusicBus,
    Lr,
    Unknown,
}

impl ChannelRole {
    pub fn is_open_mic(self) -> bool {
        matches!(
            self,
            ChannelRole::Vocal
                | ChannelRole::VocalBv
                | ChannelRole::DrumOh
                | ChannelRole::Talkback
                | ChannelRole::Guitar
        )
    }

    pub fn is_vocal(self) -> bool {
        matches!(self, ChannelRole::Vocal | ChannelRole::VocalBv | ChannelRole::VocalBus)
    }

    pub fn is_low_end(self) -> bool {
        matches!(self, ChannelRole::Kick | ChannelRole::Bass)
    }
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub sample_rate: u32,
    pub channels: Vec<Vec<f32>>,
}

impl Buffer {
    pub fn new(sample_rate: u32, channels: Vec<Vec<f32>>) -> Result<Self, AudioError> {
        if channels.is_empty() || channels.iter().any(|c| c.is_empty()) {
            return Err(AudioError::Empty);
        }
        let n = channels[0].len();
        if channels.iter().any(|c| c.len() != n) {
            return Err(AudioError::Channels);
        }
        Ok(Self { sample_rate, channels })
    }

    pub fn frames(&self) -> usize {
        self.channels.first().map(|c| c.len()).unwrap_or(0)
    }

    pub fn channel_count(&self) -> usize {
        self.channels.len()
    }

    pub fn duration_secs(&self) -> f32 {
        self.frames() as f32 / self.sample_rate as f32
    }

    pub fn from_wav(path: &Path) -> Result<Self, AudioError> {
        let mut f = File::open(path).map_err(|e| AudioError::Io(e.to_string()))?;
        let mut hdr = [0u8; 12];
        f.read_exact(&mut hdr).map_err(|e| AudioError::Io(e.to_string()))?;
        if &hdr[0..4] != b"RIFF" || &hdr[8..12] != b"WAVE" {
            return Err(AudioError::Format("not RIFF/WAVE".into()));
        }

        let mut audio_fmt = 0u16;
        let mut ch = 0u16;
        let mut sr = 0u32;
        let mut bits = 0u16;
        let mut data = Vec::new();

        loop {
            let mut chunk_hdr = [0u8; 8];
            if f.read_exact(&mut chunk_hdr).is_err() {
                break;
            }
            let id = &chunk_hdr[0..4];
            let size = u32::from_le_bytes(chunk_hdr[4..8].try_into().unwrap()) as u64;
            if id == b"fmt " {
                let mut fmt = vec![0u8; size as usize];
                f.read_exact(&mut fmt).map_err(|e| AudioError::Io(e.to_string()))?;
                if fmt.len() < 16 {
                    return Err(AudioError::Format("short fmt".into()));
                }
                audio_fmt = u16::from_le_bytes(fmt[0..2].try_into().unwrap());
                ch = u16::from_le_bytes(fmt[2..4].try_into().unwrap());
                sr = u32::from_le_bytes(fmt[4..8].try_into().unwrap());
                bits = u16::from_le_bytes(fmt[14..16].try_into().unwrap());
            } else if id == b"data" {
                data.resize(size as usize, 0);
                f.read_exact(&mut data).map_err(|e| AudioError::Io(e.to_string()))?;
            } else if f.seek(SeekFrom::Current(size as i64)).is_err() {
                break;
            }
            if size % 2 == 1 {
                let _ = f.seek(SeekFrom::Current(1));
            }
        }

        if ch == 0 || sr == 0 {
            return Err(AudioError::Format("missing fmt".into()));
        }
        let chn = ch as usize;
        let mut planar = vec![Vec::new(); chn];
        match (audio_fmt, bits) {
            (1, 16) => {
                for (i, frame) in data.chunks_exact(2).enumerate() {
                    let v = i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0;
                    planar[i % chn].push(v);
                }
            }
            (3, 32) => {
                for (i, frame) in data.chunks_exact(4).enumerate() {
                    let v = f32::from_le_bytes(frame.try_into().unwrap());
                    planar[i % chn].push(v);
                }
            }
            (fmt, b) => return Err(AudioError::Format(format!("unsupported wav fmt={fmt} bits={b}"))),
        }
        Buffer::new(sr, planar)
    }
}

pub fn db_from_lin(x: f32) -> f32 {
    if x <= 1.0e-12 { -120.0 } else { 20.0 * x.log10() }
}

pub fn lin_from_db(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn clamp(x: f32, lo: f32, hi: f32) -> f32 {
    x.max(lo).min(hi)
}
