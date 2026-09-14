use foh_core::propose::{MixMove, MoveKind, MAX_EQ_GAIN_DB, MAX_FADER_DELTA_DB};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SafetyViolation {
    FaderCap,
    EqCap,
    RateLimit,
    Locked,
}

#[derive(Debug, Clone)]
pub struct SafetyGate {
    pub locked_channels: Vec<String>,
    pub min_interval_ms: u64,
    last_ms: Vec<(String, u64)>,
}

impl Default for SafetyGate {
    fn default() -> Self {
        Self {
            locked_channels: vec!["talkback".into(), "announce".into()],
            min_interval_ms: 250,
            last_ms: Vec::new(),
        }
    }
}

impl SafetyGate {
    pub fn check(&mut self, now_ms: u64, mv: &MixMove) -> Result<(), SafetyViolation> {
        if self.locked_channels.iter().any(|c| c.eq_ignore_ascii_case(&mv.channel)) {
            return Err(SafetyViolation::Locked);
        }
        match mv.kind {
            MoveKind::FaderDb | MoveKind::SendDb => {
                if mv.delta_db.abs() > MAX_FADER_DELTA_DB + 1.0e-3 {
                    return Err(SafetyViolation::FaderCap);
                }
            }
            MoveKind::EqPeak | MoveKind::EqCut => {
                if mv.delta_db.abs() > MAX_EQ_GAIN_DB + 1.0e-3 {
                    return Err(SafetyViolation::EqCap);
                }
            }
            _ => {}
        }
        if let Some((_, t)) = self.last_ms.iter().find(|(c, _)| c == &mv.channel) {
            if now_ms.saturating_sub(*t) < self.min_interval_ms {
                return Err(SafetyViolation::RateLimit);
            }
        }
        if let Some(slot) = self.last_ms.iter_mut().find(|(c, _)| c == &mv.channel) {
            slot.1 = now_ms;
        } else {
            self.last_ms.push((mv.channel.clone(), now_ms));
        }
        Ok(())
    }
}

impl std::fmt::Display for SafetyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SafetyViolation::FaderCap => write!(f, "fader delta exceeds cap"),
            SafetyViolation::EqCap => write!(f, "eq gain exceeds cap"),
            SafetyViolation::RateLimit => write!(f, "rate limit"),
            SafetyViolation::Locked => write!(f, "channel is locked"),
        }
    }
}
