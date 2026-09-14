use foh_core::audio::ChannelRole;
use foh_core::propose::{MixMove, MoveKind};
use foh_core::session::Session;
use foh_mix::apply_moves;

fn sine(freq: f32, amp: f32, sr: u32, secs: f32) -> Vec<f32> {
    let n = (sr as f32 * secs) as usize;
    (0..n)
        .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sr as f32).sin() * amp)
        .collect()
}

fn rms(x: &[f32]) -> f32 {
    let s: f32 = x.iter().map(|v| v * v).sum();
    (s / x.len() as f32).sqrt()
}

#[test]
fn peak_cut_reduces_tone() {
    let sr = 48_000;
    let mut session = Session::new(sr);
    session.push("vox", ChannelRole::Vocal, sine(2500.0, 0.4, sr, 0.4));
    let mv = MixMove {
        kind: MoveKind::EqCut,
        channel: "vox".into(),
        freq_hz: Some(2500.0),
        q: Some(4.0),
        delta_db: -4.0,
        reason: "test".into(),
        source_code: "feedback_risk",
    };
    let before = rms(&session.stems[0].samples);
    let (after, app) = apply_moves(&session, &[mv]);
    assert_eq!(app.applied.len(), 1);
    let after_rms = rms(&after.stems[0].samples);
    assert!(after_rms < before * 0.85, "before={before} after={after_rms}");
}

#[test]
fn fader_plus_two_is_about_two_db() {
    let sr = 48_000;
    let mut session = Session::new(sr);
    session.push("vox", ChannelRole::Vocal, sine(1000.0, 0.1, sr, 0.3));
    let mv = MixMove {
        kind: MoveKind::FaderDb,
        channel: "vox".into(),
        freq_hz: None,
        q: None,
        delta_db: 2.0,
        reason: "test".into(),
        source_code: "vocal_buried",
    };
    let before = rms(&session.stems[0].samples);
    let (after, _) = apply_moves(&session, &[mv]);
    let after_rms = rms(&after.stems[0].samples);
    let ratio = after_rms / before;
    assert!((ratio - 1.2589).abs() < 0.02, "ratio={ratio}");
}
