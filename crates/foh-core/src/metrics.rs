use crate::audio::{db_from_lin, ChannelRole};
use crate::OCTAVE_CENTERS_HZ;

#[derive(Debug, Clone)]
pub struct ChannelMetrics {
    pub role: ChannelRole,
    pub peak_db: f32,
    pub rms_db: f32,
    pub crest_db: f32,
    pub loudness_db: f32,
    pub dc: f32,
    pub octave_db: [f32; 10],
    pub ring_hz: Option<f32>,
    pub ring_prominence_db: f32,
    pub activity: f32,
}

impl ChannelMetrics {
    pub fn band_db(&self, center_hz: f32) -> Option<f32> {
        OCTAVE_CENTERS_HZ
            .iter()
            .position(|c| (*c - center_hz).abs() < 1.0)
            .map(|i| self.octave_db[i])
    }

    pub fn low_end_db(&self) -> f32 {
        db_from_lin((lin_approx(self.octave_db[1]) + lin_approx(self.octave_db[2])) * 0.5)
    }

    pub fn presence_db(&self) -> f32 {
        db_from_lin((lin_approx(self.octave_db[6]) + lin_approx(self.octave_db[7])) * 0.5)
    }

    pub fn mud_db(&self) -> f32 {
        self.octave_db[3]
    }
}

fn lin_approx(db: f32) -> f32 {
    if db <= -120.0 {
        0.0
    } else {
        10.0f32.powf(db / 20.0)
    }
}
