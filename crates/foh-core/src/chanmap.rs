//! Name to desk channel + role.
//!
//! File format, one entry per line:
//!   <name> <desk_channel> <role>
//!
//! Desk channels are 1-based DiGiCo input numbers as used by public OSC paths.

use crate::audio::ChannelRole;

#[derive(Debug, Clone)]
pub struct MapEntry {
    pub name: String,
    pub desk_channel: u32,
    pub role: ChannelRole,
}

#[derive(Debug, Clone, Default)]
pub struct ChannelMap {
    pub entries: Vec<MapEntry>,
}

impl ChannelMap {
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut entries = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() != 3 {
                return Err(format!("line {}: expected `name channel role`", i + 1));
            }
            let desk_channel: u32 = parts[1]
                .parse()
                .map_err(|_| format!("line {}: bad channel `{}`", i + 1, parts[1]))?;
            if desk_channel == 0 {
                return Err(format!("line {}: desk channel is 1-based", i + 1));
            }
            let role = parse_role(parts[2]).ok_or_else(|| {
                format!("line {}: unknown role `{}`", i + 1, parts[2])
            })?;
            entries.push(MapEntry {
                name: parts[0].to_string(),
                desk_channel,
                role,
            });
        }
        Ok(Self { entries })
    }

    pub fn desk_channel(&self, name: &str) -> Option<u32> {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .map(|e| e.desk_channel)
    }

    pub fn role(&self, name: &str) -> Option<ChannelRole> {
        self.entries.iter().find(|e| e.name == name).map(|e| e.role)
    }
}

pub fn parse_role(s: &str) -> Option<ChannelRole> {
    Some(match s.to_ascii_lowercase().as_str() {
        "kick" => ChannelRole::Kick,
        "bass" => ChannelRole::Bass,
        "snare" => ChannelRole::Snare,
        "hats" | "hat" => ChannelRole::Hats,
        "tom" => ChannelRole::Tom,
        "guitar" | "gtr" => ChannelRole::Guitar,
        "keys" | "key" => ChannelRole::Keys,
        "vocal" | "vox" => ChannelRole::Vocal,
        "bv" | "vocalbv" => ChannelRole::VocalBv,
        "oh" | "drumoh" => ChannelRole::DrumOh,
        "playback" => ChannelRole::Playback,
        "talkback" | "tb" => ChannelRole::Talkback,
        "fx" | "fxreturn" => ChannelRole::FxReturn,
        "drumbus" => ChannelRole::DrumBus,
        "vocalbus" => ChannelRole::VocalBus,
        "musicbus" => ChannelRole::MusicBus,
        "lr" => ChannelRole::Lr,
        "unknown" => ChannelRole::Unknown,
        _ => return None,
    })
}
