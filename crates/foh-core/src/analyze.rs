use crate::audio::{db_from_lin, Buffer, ChannelRole};
use crate::bands::octave_energy;
use crate::metrics::ChannelMetrics;
use crate::session::Session;

const FFT_N: usize = 4096;
const HOP: usize = 1024;

#[derive(Debug, Clone)]
pub struct ChannelReport {
    pub name: String,
    pub index: usize,
    pub metrics: ChannelMetrics,
}

#[derive(Debug, Clone)]
pub struct MixReport {
    pub sample_rate: u32,
    pub duration_secs: f32,
    pub channels: Vec<ChannelReport>,
    pub lr_correlation: f32,
    pub lr_peak_db: f32,
    pub lr_rms_db: f32,
    pub kick_bass_corr: f32,
}

pub fn analyze_session(session: &Session) -> MixReport {
    analyze_named(&session.stems, session.sample_rate)
}

pub fn analyze_buffer(name: &str, role: ChannelRole, buf: &Buffer) -> ChannelReport {
    let metrics = analyze_mono(role, &buf.channels[0], buf.sample_rate);
    ChannelReport { name: name.to_string(), index: 0, metrics }
}

pub fn analyze_named(stems: &[crate::session::NamedStem], sample_rate: u32) -> MixReport {
    let mut channels = Vec::with_capacity(stems.len());
    let mut max_len = 0usize;
    for (i, stem) in stems.iter().enumerate() {
        max_len = max_len.max(stem.samples.len());
        channels.push(ChannelReport {
            name: stem.name.clone(),
            index: i,
            metrics: analyze_mono(stem.role, &stem.samples, sample_rate),
        });
    }
    let mix = sum_stems(stems, max_len);
    let mix_m = analyze_mono(ChannelRole::Lr, &mix, sample_rate);
    MixReport {
        sample_rate,
        duration_secs: max_len as f32 / sample_rate as f32,
        channels,
        lr_correlation: 1.0,
        lr_peak_db: mix_m.peak_db,
        lr_rms_db: mix_m.rms_db,
        kick_bass_corr: kick_bass_correlation(stems),
    }
}

pub fn sum_stems(stems: &[crate::session::NamedStem], len: usize) -> Vec<f32> {
    let mut mix = vec![0.0f32; len];
    for stem in stems {
        for (i, s) in stem.samples.iter().enumerate() {
            if i < len {
                mix[i] += *s;
            }
        }
    }
    mix
}

pub fn analyze_mono(role: ChannelRole, samples: &[f32], sample_rate: u32) -> ChannelMetrics {
    let n = samples.len().max(1);
    let mut peak = 0.0f32;
    let mut sum_sq = 0.0f32;
    let mut dc = 0.0f32;
    let mut active = 0u32;
    let gate = 1.0e-3f32;
    for &s in samples {
        let a = s.abs();
        if a > peak { peak = a; }
        sum_sq += s * s;
        dc += s;
        if a > gate { active += 1; }
    }
    let rms = (sum_sq / n as f32).sqrt();
    let peak_db = db_from_lin(peak);
    let rms_db = db_from_lin(rms);
    let (octave, ring_hz, ring_prom) = spectrum_features(samples, sample_rate);
    ChannelMetrics {
        role,
        peak_db,
        rms_db,
        crest_db: peak_db - rms_db,
        loudness_db: k_weighted_rms_db(samples, sample_rate),
        dc: dc / n as f32,
        octave_db: octave,
        ring_hz,
        ring_prominence_db: ring_prom,
        activity: active as f32 / n as f32,
    }
}

fn spectrum_features(samples: &[f32], sample_rate: u32) -> ([f32; 10], Option<f32>, f32) {
    if samples.len() < FFT_N {
        return ([-120.0; 10], None, 0.0);
    }
    let mut acc = vec![0.0f32; FFT_N / 2 + 1];
    let mut frames = 0u32;
    let window = hann(FFT_N);
    let mut frame = vec![0.0f32; FFT_N];
    let mut pos = 0usize;
    while pos + FFT_N <= samples.len() {
        for i in 0..FFT_N {
            frame[i] = samples[pos + i] * window[i];
        }
        let mags = crate::fft::rfft_mags(&frame);
        for (i, m) in mags.iter().enumerate() {
            acc[i] += *m;
        }
        frames += 1;
        pos += HOP;
    }
    if frames == 0 {
        return ([-120.0; 10], None, 0.0);
    }
    for v in &mut acc { *v /= frames as f32; }
    let octave_lin = octave_energy(&acc, sample_rate);
    let mut octave_db = [-120.0f32; 10];
    for i in 0..10 {
        octave_db[i] = db_from_lin(octave_lin[i]);
    }
    let (ring_hz, prom) = find_ring(&acc, sample_rate);
    (octave_db, ring_hz, prom)
}

fn find_ring(mags: &[f32], sample_rate: u32) -> (Option<f32>, f32) {
    let fft_n = (mags.len().saturating_sub(1)) * 2;
    if fft_n == 0 || mags.len() < 16 {
        return (None, 0.0);
    }
    let hz_per = sample_rate as f32 / fft_n as f32;
    let start = ((200.0 / hz_per) as usize).max(4);
    let end = ((8_000.0 / hz_per) as usize).min(mags.len().saturating_sub(3));
    if start >= end {
        return (None, 0.0);
    }
    let peak_mag = mags[start..end].iter().cloned().fold(0.0f32, f32::max);
    if peak_mag <= 1.0e-9 {
        return (None, 0.0);
    }
    let mut best_k = start;
    let mut best_score = 0.0f32;
    let mut best_prom = 0.0f32;
    for k in start..end {
        if mags[k] < peak_mag * 0.25 { continue; }
        let neigh = (mags[k - 2] + mags[k - 1] + mags[k + 1] + mags[k + 2]) * 0.25;
        if neigh <= 1.0e-12 { continue; }
        let prom = db_from_lin(mags[k] / neigh);
        let score = mags[k] * prom.max(0.0);
        if prom >= 8.0 && score > best_score {
            best_score = score;
            best_prom = prom;
            best_k = k;
        }
    }
    if best_prom >= 8.0 {
        (Some(best_k as f32 * hz_per), best_prom)
    } else {
        (None, best_prom)
    }
}

fn hann(n: usize) -> Vec<f32> {
    (0..n).map(|i| 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (n.saturating_sub(1) as f32)).cos()).collect()
}

fn k_weighted_rms_db(samples: &[f32], sample_rate: u32) -> f32 {
    let hp = biquad_highpass(60.0, sample_rate);
    let hs = biquad_highshelf(1500.0, 4.0, sample_rate);
    let mut s1 = BiquadState::default();
    let mut s2 = BiquadState::default();
    let mut sum_sq = 0.0f32;
    for &x in samples {
        let y = s2.process(&hs, s1.process(&hp, x));
        sum_sq += y * y;
    }
    db_from_lin((sum_sq / samples.len().max(1) as f32).sqrt())
}

#[derive(Clone, Copy, Default)]
struct BiquadState { z1: f32, z2: f32 }
impl BiquadState {
    fn process(&mut self, c: &Biquad, x: f32) -> f32 {
        let y = c.b0 * x + self.z1;
        self.z1 = c.b1 * x - c.a1 * y + self.z2;
        self.z2 = c.b2 * x - c.a2 * y;
        y
    }
}
struct Biquad { b0: f32, b1: f32, b2: f32, a1: f32, a2: f32 }

fn biquad_highpass(fc: f32, sr: u32) -> Biquad {
    let w = 2.0 * std::f32::consts::PI * fc / sr as f32;
    let cosw = w.cos();
    let sinw = w.sin();
    let alpha = sinw / (2.0 * 0.707);
    let b0 = (1.0 + cosw) * 0.5;
    let b1 = -(1.0 + cosw);
    let b2 = (1.0 + cosw) * 0.5;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cosw;
    let a2 = 1.0 - alpha;
    Biquad { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0 }
}

fn biquad_highshelf(fc: f32, gain_db: f32, sr: u32) -> Biquad {
    let a = 10.0f32.powf(gain_db / 40.0);
    let w = 2.0 * std::f32::consts::PI * fc / sr as f32;
    let cosw = w.cos();
    let sinw = w.sin();
    let alpha = sinw / 2.0 * ((a + 1.0 / a) * (1.0 / 0.707 - 1.0) + 2.0).sqrt();
    let two = 2.0 * a.sqrt() * alpha;
    let b0 = a * ((a + 1.0) + (a - 1.0) * cosw + two);
    let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cosw);
    let b2 = a * ((a + 1.0) + (a - 1.0) * cosw - two);
    let a0 = (a + 1.0) - (a - 1.0) * cosw + two;
    let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cosw);
    let a2 = (a + 1.0) - (a - 1.0) * cosw - two;
    Biquad { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0 }
}

fn kick_bass_correlation(stems: &[crate::session::NamedStem]) -> f32 {
    let kick = stems.iter().find(|s| s.role == ChannelRole::Kick);
    let bass = stems.iter().find(|s| s.role == ChannelRole::Bass);
    match (kick, bass) {
        (Some(k), Some(b)) => crate::corr::pearson(&k.samples, &b.samples),
        _ => 0.0,
    }
}
