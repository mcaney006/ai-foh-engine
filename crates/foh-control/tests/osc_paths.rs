use foh_control::digico::{CommandSet, DigicoCommand};

#[test]
fn ipad_fader_path() {
    let cmd = DigicoCommand::Fader {
        channel: 12,
        level_01: 0.75,
    };
    let (addr, args) = cmd.address(CommandSet::Ipad).unwrap();
    assert_eq!(addr, "/Input_Channels/12/fader");
    assert_eq!(args.len(), 1);
}

#[test]
fn osc_set_prefixes_sd() {
    let cmd = DigicoCommand::Mute {
        channel: 3,
        muted: true,
    };
    let (addr, _) = cmd.address(CommandSet::Osc).unwrap();
    assert_eq!(addr, "/sd/Input_Channels/3/mute");
}

#[test]
fn s_series_rejects_fader() {
    let cmd = DigicoCommand::Fader {
        channel: 1,
        level_01: 0.5,
    };
    assert!(cmd.address(CommandSet::SSeries).is_err());
}

#[test]
fn snapshot_and_macro_encode() {
    let snap = DigicoCommand::FireSnapshot { number: 42 };
    let (addr, _) = snap.address(CommandSet::Ipad).unwrap();
    assert_eq!(addr, "/Snapshots/Fire_Snapshot_number");
    let mac = DigicoCommand::PressMacro { number: 4 };
    let bytes = mac.encode(CommandSet::Osc).unwrap();
    assert!(!bytes.is_empty());
}
