use foh_core::audio::ChannelRole;
use foh_core::diagnose::diagnose;
use foh_core::fixtures::{mix, problem_mix, sine};
use foh_core::propose::propose;
use foh_core::session::Session;
use foh_core::verify::{verify, VerifyDecision};
use foh_core::{analyze_session, DEFAULT_SR};
use foh_mix::apply_proposal;

#[test]
fn problem_mix_accepts_after_capped_cuts() {
    let session = problem_mix(DEFAULT_SR);
    let before = analyze_session(&session);
    let dx = diagnose(&before);
    assert!(
        dx.iter().any(|d| d.code == "feedback_risk"),
        "codes: {:?}",
        dx.iter().map(|d| d.code).collect::<Vec<_>>(),
    );
    assert!(dx.iter().any(|d| d.code == "kick_bass_63"));
    let proposal = propose(&before, &dx);
    assert!(proposal
        .moves
        .iter()
        .any(|m| m.channel == "vox" && m.freq_hz.unwrap_or(0.0) > 2000.0));
    let after_sess = apply_proposal(&session, &proposal).session;
    let after = analyze_session(&after_sess);
    let v = verify(&before, &after);
    assert_eq!(v.decision, VerifyDecision::Accept, "{v:?}");
    assert!(!v.feedback_worse);
}

#[test]
fn after_clip_rejects() {
    let mut before_s = Session::new(DEFAULT_SR);
    before_s.push("keys", ChannelRole::Keys, sine(1000.0, 0.4, DEFAULT_SR, 0.5));
    let mut after_s = Session::new(DEFAULT_SR);
    after_s.push("keys", ChannelRole::Keys, sine(1000.0, 1.2, DEFAULT_SR, 0.5));
    let v = verify(&analyze_session(&before_s), &analyze_session(&after_s));
    assert_eq!(v.decision, VerifyDecision::Reject);
    assert!(v.notes.iter().any(|n| n.code == "after_clip"));
}

#[test]
fn kick_fundamental_is_not_feedback() {
    let kick = mix(&[
        &sine(70.0, 0.4, DEFAULT_SR, 1.0),
        &sine(140.0, 0.08, DEFAULT_SR, 1.0),
    ]);
    let mut session = Session::new(DEFAULT_SR);
    session.push("kick", ChannelRole::Kick, kick);
    let report = analyze_session(&session);
    let dx = diagnose(&report);
    assert!(
        !dx.iter()
            .any(|d| d.code == "feedback_risk" || d.code == "ring_mode"),
        "kick flagged as howl: {dx:?}"
    );
}
