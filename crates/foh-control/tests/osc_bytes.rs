use foh_control::digico::{encode_osc, CommandSet, DigicoCommand, OscArg};
use foh_control::safety::SafetyGate;
use foh_core::propose::{MixMove, MoveKind};

#[test]
fn osc_packet_is_multiple_of_four() {
    let bytes = encode_osc("/Input_Channels/12/fader", &[OscArg::Float(0.75)]);
    assert_eq!(bytes.len() % 4, 0);
    assert!(bytes.starts_with(b"/Input_Channels/12/fader\0"));
}

#[test]
fn encode_roundtrip_length_stable() {
    let a = DigicoCommand::Mute {
        channel: 3,
        muted: true,
    }
    .encode(CommandSet::Osc)
    .unwrap();
    let b = DigicoCommand::Mute {
        channel: 3,
        muted: true,
    }
    .encode(CommandSet::Osc)
    .unwrap();
    assert_eq!(a, b);
}

#[test]
fn safety_locks_talkback() {
    let mut g = SafetyGate::default();
    let mv = MixMove {
        kind: MoveKind::FaderDb,
        channel: "talkback".into(),
        freq_hz: None,
        q: None,
        delta_db: 1.0,
        reason: "test".into(),
        source_code: "x",
    };
    assert!(g.check(0, &mv).is_err());
}

#[test]
fn safety_rate_limit() {
    let mut g = SafetyGate::default();
    let mv = MixMove {
        kind: MoveKind::FaderDb,
        channel: "vox".into(),
        freq_hz: None,
        q: None,
        delta_db: 1.0,
        reason: "test".into(),
        source_code: "x",
    };
    assert!(g.check(0, &mv).is_ok());
    assert!(g.check(10, &mv).is_err());
    assert!(g.check(300, &mv).is_ok());
}
