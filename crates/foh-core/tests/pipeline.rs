use foh_core::audio::ChannelRole;
use foh_core::diagnose::diagnose;
use foh_core::propose::propose;
use foh_core::session::Session;
use foh_core::{analyze_session, DEFAULT_SR};

fn sine(freq: f32, amp: f32, sr: u32, secs: f32) -> Vec<f32> {
    let n = (sr as f32 * secs) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / sr as f32;
            (2.0 * std::f32::consts::PI * freq * t).sin() * amp
        })
        .collect()
}

fn add(dst: &mut [f32], src: &[f32]) {
    for (d, s) in dst.iter_mut().zip(src) {
        *d += *s;
    }
}

#[test]
fn sine_peak_and_octave() {
    let s = sine(1000.0, 0.5, DEFAULT_SR, 0.5);
    let mut session = Session::new(DEFAULT_SR);
    session.push("tone", ChannelRole::Keys, s);
    let report = analyze_session(&session);
    let ch = &report.channels[0];
    assert!(ch.metrics.peak_db > -7.0 && ch.metrics.peak_db < -5.5);
    let k = ch.metrics.octave_db[5];
    assert!(k > ch.metrics.octave_db[2] + 8.0);
}

#[test]
fn detects_vocal_buried_and_proposes_lift() {
    let sr = DEFAULT_SR;
    let vocal = sine(1000.0, 0.03, sr, 1.0);
    let keys = sine(1200.0, 0.25, sr, 1.0);
    let mut session = Session::new(sr);
    session.push("vox", ChannelRole::Vocal, vocal);
    session.push("keys", ChannelRole::Keys, keys);
    let report = analyze_session(&session);
    let dx = diagnose(&report);
    assert!(
        dx.iter().any(|d| d.code == "vocal_buried" || d.code == "vocal_masked"),
        "diagnoses: {:?}",
        dx.iter().map(|d| d.code).collect::<Vec<_>>()
    );
    let prop = propose(&report, &dx);
    assert!(prop
        .moves
        .iter()
        .any(|m| m.channel == "vox" && m.delta_db > 0.0));
}

#[test]
fn ring_on_open_mic_flags_feedback() {
    let sr = DEFAULT_SR;
    let mut mic = sine(2500.0, 0.4, sr, 1.0);
    let noise = sine(400.0, 0.02, sr, 1.0);
    add(&mut mic, &noise);
    let mut session = Session::new(sr);
    session.push("vox", ChannelRole::Vocal, mic);
    let report = analyze_session(&session);
    let m = &report.channels[0].metrics;
    assert!(m.ring_hz.is_some(), "expected ring, got {:?}", m.ring_hz);
    let dx = diagnose(&report);
    assert!(
        dx.iter()
            .any(|d| d.code == "feedback_risk" || d.code == "ring_mode"),
        "{dx:?}"
    );
}
