use std::net::{SocketAddr, UdpSocket};

use foh_core::audio::clamp;
use foh_core::propose::{MixMove, MoveKind};

/// Command sets actually used in the wild.
///
/// OSC: `/sd/Input_Channels/{n}/fader`  (Companion "OSC" set)
/// IPAD: `/Input_Channels/{n}/fader`    (console iPad protocol; Companion default)
/// S: snapshot-only `/digico/snapshots/fire`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandSet {
    Osc,
    Ipad,
    SSeries,
}

#[derive(Debug, Clone)]
pub struct DigicoTarget {
    pub addr: SocketAddr,
    pub set: CommandSet,
}

#[derive(Debug)]
pub enum ControlError {
    Io(std::io::Error),
    Encode,
    Unsupported(&'static str),
}

impl From<std::io::Error> for ControlError {
    fn from(e: std::io::Error) -> Self {
        ControlError::Io(e)
    }
}

impl std::fmt::Display for ControlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControlError::Io(e) => write!(f, "io: {e}"),
            ControlError::Encode => write!(f, "osc encode"),
            ControlError::Unsupported(s) => write!(f, "unsupported on this command set: {s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OscArg {
    Int(i32),
    Float(f32),
}

#[derive(Debug, Clone)]
pub enum DigicoCommand {
    Fader { channel: u32, level_01: f32 },
    Mute { channel: u32, muted: bool },
    Solo { channel: u32, solo: bool },
    Phantom { channel: u32, on: bool },
    FireSnapshot { number: u32 },
    FireNextSnapshot,
    FirePrevSnapshot,
    PressMacro { number: u32 },
}

impl DigicoCommand {
    pub fn from_move(channel_number: u32, current_fader_db: f32, mv: &MixMove) -> Option<Self> {
        match mv.kind {
            MoveKind::FaderDb => {
                let next = clamp(current_fader_db + mv.delta_db, -80.0, 10.0);
                Some(Self::Fader {
                    channel: channel_number,
                    level_01: db_to_fader_01(next),
                })
            }
            _ => None,
        }
    }

    pub fn address(&self, set: CommandSet) -> Result<(String, Vec<OscArg>), ControlError> {
        let sd = match set {
            CommandSet::Osc => "/sd",
            CommandSet::Ipad => "",
            CommandSet::SSeries => return self.s_series(),
        };
        match *self {
            DigicoCommand::Fader { channel, level_01 } => Ok((
                format!("{sd}/Input_Channels/{channel}/fader"),
                vec![OscArg::Float(level_01)],
            )),
            DigicoCommand::Mute { channel, muted } => Ok((
                format!("{sd}/Input_Channels/{channel}/mute"),
                vec![OscArg::Int(i32::from(muted))],
            )),
            DigicoCommand::Solo { channel, solo } => Ok((
                format!("{sd}/Input_Channels/{channel}/solo"),
                vec![OscArg::Int(i32::from(solo))],
            )),
            DigicoCommand::Phantom { channel, on } => Ok((
                format!("{sd}/Input_Channels/{channel}/Channel_Input/phantom"),
                vec![OscArg::Int(i32::from(on))],
            )),
            DigicoCommand::FireSnapshot { number } => Ok((
                format!("{sd}/Snapshots/Fire_Snapshot_number"),
                vec![OscArg::Int(number as i32)],
            )),
            DigicoCommand::FireNextSnapshot => Ok((
                format!("{sd}/Snapshots/Fire_Next_Snapshot"),
                vec![OscArg::Int(0)],
            )),
            DigicoCommand::FirePrevSnapshot => Ok((
                format!("{sd}/Snapshots/Fire_Prev_Snapshot"),
                vec![OscArg::Int(0)],
            )),
            DigicoCommand::PressMacro { number } => Ok((
                format!("{sd}/Macros/Buttons/press"),
                vec![OscArg::Int(number.saturating_sub(1) as i32)],
            )),
        }
    }

    fn s_series(&self) -> Result<(String, Vec<OscArg>), ControlError> {
        match *self {
            DigicoCommand::FireSnapshot { number } => Ok((
                "/digico/snapshots/fire".into(),
                vec![OscArg::Int(number as i32)],
            )),
            DigicoCommand::FireNextSnapshot => {
                Ok(("/digico/snapshots/fire/next".into(), vec![OscArg::Int(0)]))
            }
            DigicoCommand::FirePrevSnapshot => Ok((
                "/digico/snapshots/fire/previous".into(),
                vec![OscArg::Int(0)],
            )),
            _ => Err(ControlError::Unsupported(
                "S-Series module only documents snapshot fire",
            )),
        }
    }

    pub fn encode(&self, set: CommandSet) -> Result<Vec<u8>, ControlError> {
        let (addr, args) = self.address(set)?;
        Ok(encode_osc(&addr, &args))
    }
}

fn pad4(buf: &mut Vec<u8>) {
    while buf.len() % 4 != 0 {
        buf.push(0);
    }
}

pub fn encode_osc(addr: &str, args: &[OscArg]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(addr.as_bytes());
    out.push(0);
    pad4(&mut out);
    let mut tags = String::from(",");
    for a in args {
        tags.push(match a {
            OscArg::Int(_) => 'i',
            OscArg::Float(_) => 'f',
        });
    }
    out.extend_from_slice(tags.as_bytes());
    out.push(0);
    pad4(&mut out);
    for a in args {
        match *a {
            OscArg::Int(v) => out.extend_from_slice(&v.to_be_bytes()),
            OscArg::Float(v) => out.extend_from_slice(&v.to_be_bytes()),
        }
    }
    out
}

pub struct OscClient {
    sock: UdpSocket,
    target: DigicoTarget,
}

impl OscClient {
    pub fn bind(local: SocketAddr, target: DigicoTarget) -> Result<Self, ControlError> {
        let sock = UdpSocket::bind(local)?;
        sock.set_nonblocking(true)?;
        Ok(Self { sock, target })
    }

    pub fn send(&self, cmd: &DigicoCommand) -> Result<(), ControlError> {
        let bytes = cmd.encode(self.target.set)?;
        self.sock.send_to(&bytes, self.target.addr)?;
        Ok(())
    }
}

/// DiGiCo-style fader: 0.0 = -inf, ~0.75 = 0 dB, 1.0 = +10 dB.
pub fn db_to_fader_01(db: f32) -> f32 {
    if db <= -80.0 {
        return 0.0;
    }
    if db < 0.0 {
        clamp(0.75 * (db + 80.0) / 80.0, 0.0, 0.75)
    } else {
        clamp(0.75 + 0.25 * (db / 10.0), 0.75, 1.0)
    }
}

pub fn fader_01_to_db(x: f32) -> f32 {
    let x = clamp(x, 0.0, 1.0);
    if x <= 0.0 {
        -120.0
    } else if x <= 0.75 {
        -80.0 + 80.0 * (x / 0.75)
    } else {
        10.0 * ((x - 0.75) / 0.25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fader_law_unity_at_zero_db() {
        let x = db_to_fader_01(0.0);
        assert!((x - 0.75).abs() < 1.0e-6);
        assert!((fader_01_to_db(0.75)).abs() < 1.0e-4);
        assert!((fader_01_to_db(1.0) - 10.0).abs() < 1.0e-4);
        assert!(foh_core::audio::lin_from_db(-6.0) > 0.4);
    }
}
