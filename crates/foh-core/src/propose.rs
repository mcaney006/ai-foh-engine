use crate::analyze::MixReport;
use crate::audio::clamp;
use crate::diagnose::{Diagnosis, Severity};

pub const MAX_FADER_DELTA_DB: f32 = 3.0;
pub const MAX_EQ_GAIN_DB: f32 = 4.0;
pub const MAX_EQ_Q: f32 = 8.0;
pub const MIN_EQ_Q: f32 = 0.7;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveKind {
    FaderDb,
    EqPeak,
    EqCut,
    CompThresholdDb,
    GateThresholdDb,
    SendDb,
}

#[derive(Debug, Clone)]
pub struct MixMove {
    pub kind: MoveKind,
    pub channel: String,
    pub freq_hz: Option<f32>,
    pub q: Option<f32>,
    pub delta_db: f32,
    pub reason: String,
    pub source_code: &'static str,
}

#[derive(Debug, Clone)]
pub struct Proposal {
    pub moves: Vec<MixMove>,
}

pub fn propose(report: &MixReport, diagnoses: &[Diagnosis]) -> Proposal {
    let mut moves = Vec::new();
    for d in diagnoses {
        match d.code {
            "clip_risk" | "hot_peak" | "lr_clip" => {
                let ch = if d.channel == "LR" {
                    report
                        .channels
                        .iter()
                        .max_by(|a, b| {
                            a.metrics
                                .peak_db
                                .partial_cmp(&b.metrics.peak_db)
                                .unwrap_or(std::cmp::Ordering::Equal)
                        })
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| d.channel.clone())
                } else {
                    d.channel.clone()
                };
                moves.push(MixMove {
                    kind: MoveKind::FaderDb,
                    channel: ch,
                    freq_hz: None,
                    q: None,
                    delta_db: if d.severity == Severity::Crit { -2.5 } else { -1.5 },
                    reason: d.detail.clone(),
                    source_code: d.code,
                });
            }
            "feedback_risk" | "ring_mode" => {
                if let Some(ch) = report.channels.iter().find(|c| c.name == d.channel) {
                    if let Some(hz) = ch.metrics.ring_hz {
                        let cut = if d.severity == Severity::Crit { -3.5 } else { -2.0 };
                        moves.push(MixMove {
                            kind: MoveKind::EqCut,
                            channel: d.channel.clone(),
                            freq_hz: Some(snap_freq(hz)),
                            q: Some(4.5),
                            delta_db: cut,
                            reason: d.detail.clone(),
                            source_code: d.code,
                        });
                    }
                }
            }
            "vocal_buried" | "vocal_masked" => {
                moves.push(MixMove {
                    kind: MoveKind::FaderDb,
                    channel: d.channel.clone(),
                    freq_hz: None,
                    q: None,
                    delta_db: 2.0,
                    reason: d.detail.clone(),
                    source_code: d.code,
                });
                moves.push(MixMove {
                    kind: MoveKind::EqPeak,
                    channel: d.channel.clone(),
                    freq_hz: Some(3_200.0),
                    q: Some(1.1),
                    delta_db: 1.5,
                    reason: "presence lift after level".into(),
                    source_code: d.code,
                });
            }
            "kick_bass_63" => {
                if let Some(bass) = report
                    .channels
                    .iter()
                    .find(|c| c.metrics.role == crate::audio::ChannelRole::Bass)
                {
                    moves.push(MixMove {
                        kind: MoveKind::EqCut,
                        channel: bass.name.clone(),
                        freq_hz: Some(63.0),
                        q: Some(1.4),
                        delta_db: -2.5,
                        reason: d.detail.clone(),
                        source_code: d.code,
                    });
                }
            }
            "kick_bass_125" => {
                if let Some(kick) = report
                    .channels
                    .iter()
                    .find(|c| c.metrics.role == crate::audio::ChannelRole::Kick)
                {
                    moves.push(MixMove {
                        kind: MoveKind::EqCut,
                        channel: kick.name.clone(),
                        freq_hz: Some(125.0),
                        q: Some(1.2),
                        delta_db: -2.0,
                        reason: d.detail.clone(),
                        source_code: d.code,
                    });
                }
            }
            "boxy_low_mids" => {
                moves.push(MixMove {
                    kind: MoveKind::EqCut,
                    channel: d.channel.clone(),
                    freq_hz: Some(250.0),
                    q: Some(1.3),
                    delta_db: -2.0,
                    reason: d.detail.clone(),
                    source_code: d.code,
                });
            }
            _ => {}
        }
    }
    for m in &mut moves {
        bound_move(m);
    }
    dedupe(&mut moves);
    Proposal { moves }
}

fn snap_freq(hz: f32) -> f32 {
    clamp(hz, 80.0, 8_000.0)
}

fn bound_move(m: &mut MixMove) {
    match m.kind {
        MoveKind::FaderDb | MoveKind::SendDb => {
            m.delta_db = clamp(m.delta_db, -MAX_FADER_DELTA_DB, MAX_FADER_DELTA_DB);
        }
        MoveKind::EqPeak | MoveKind::EqCut | MoveKind::CompThresholdDb | MoveKind::GateThresholdDb => {
            m.delta_db = clamp(m.delta_db, -MAX_EQ_GAIN_DB, MAX_EQ_GAIN_DB);
            if let Some(q) = m.q.as_mut() {
                *q = clamp(*q, MIN_EQ_Q, MAX_EQ_Q);
            }
        }
    }
}

fn dedupe(moves: &mut Vec<MixMove>) {
    let mut seen = Vec::new();
    moves.retain(|m| {
        let key = (m.channel.clone(), m.kind, m.freq_hz.map(|f| (f / 10.0).round() as i32));
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}
