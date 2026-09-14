use crate::OCTAVE_CENTERS_HZ;

pub fn octave_energy(mags: &[f32], sample_rate: u32) -> [f32; 10] {
    let n_bins = mags.len();
    let fft_n = (n_bins.saturating_sub(1)) * 2;
    let hz_per = if fft_n == 0 {
        0.0
    } else {
        sample_rate as f32 / fft_n as f32
    };

    let mut out = [0.0f32; 10];
    let mut counts = [0u32; 10];
    for (k, mag) in mags.iter().enumerate() {
        let hz = k as f32 * hz_per;
        if hz < 20.0 || hz > 20_000.0 {
            continue;
        }
        if let Some(i) = band_index(hz) {
            out[i] += mag * mag;
            counts[i] += 1;
        }
    }
    for i in 0..10 {
        if counts[i] > 0 {
            out[i] = (out[i] / counts[i] as f32).sqrt();
        }
    }
    out
}

pub fn band_index(hz: f32) -> Option<usize> {
    for (i, c) in OCTAVE_CENTERS_HZ.iter().enumerate() {
        let lo = c / std::f32::consts::SQRT_2;
        let hi = c * std::f32::consts::SQRT_2;
        if hz >= lo && hz < hi {
            return Some(i);
        }
    }
    None
}

pub fn band_name(i: usize) -> &'static str {
    const NAMES: [&str; 10] = [
        "31", "63", "125", "250", "500", "1k", "2k", "4k", "8k", "16k",
    ];
    NAMES.get(i).copied().unwrap_or("?")
}
