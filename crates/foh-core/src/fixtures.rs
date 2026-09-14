//! Deterministic stems used by the CLI demo and the integration tests.
//!
//! These are not program material. They are constructed so a known set of
//! diagnosis codes fire: vocal standing peak near 2.5 kHz, kick/bass fight
//! at 63 Hz. Amplitudes are scaled so the summed LR peak stays under 0 dBFS.

use crate::audio::ChannelRole;
use crate::session::Session;

pub const DEFAULT_MAP: &str = "# name desk role\nkick 1 kick\nbass 2 bass\nvox 12 vocal\nkeys 8 keys\n";

pub fn problem_mix(sr: u32) -> Session {
    let secs = 1.2;
    let mut session = Session::new(sr);

    session.push(
        "kick",
        ChannelRole::Kick,
        mix(&[
            &sine(60.0, 0.18, sr, secs),
            &sine(125.0, 0.10, sr, secs),
        ]),
    );

    session.push(
        "bass",
        ChannelRole::Bass,
        mix(&[
            &sine(63.0, 0.17, sr, secs),
            &sine(125.0, 0.10, sr, secs),
            &sine(250.0, 0.08, sr, secs),
        ]),
    );

    session.push(
        "vox",
        ChannelRole::Vocal,
        mix(&[
            &sine(1000.0, 0.028, sr, secs),
            &sine(2500.0, 0.09, sr, secs),
        ]),
    );

    session.push(
        "keys",
        ChannelRole::Keys,
        mix(&[
            &sine(1200.0, 0.11, sr, secs),
            &sine(3000.0, 0.07, sr, secs),
        ]),
    );

    session
}

pub fn sine(freq: f32, amp: f32, sr: u32, secs: f32) -> Vec<f32> {
    let n = (sr as f32 * secs) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / sr as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin() * amp
        })
        .collect()
}

pub fn mix(parts: &[&[f32]]) -> Vec<f32> {
    let n = parts.iter().map(|p| p.len()).max().unwrap_or(0);
    let mut out = vec![0.0f32; n];
    for p in parts {
        for (i, s) in p.iter().enumerate() {
            out[i] += *s;
        }
    }
    out
}
