//! Offline channel strip used to audition proposals before they touch a desk.

mod biquad;
mod dynamics;
mod strip;

pub use strip::{apply_proposal, ChannelStrip, ProcessedSession};

use foh_core::audio::{lin_from_db, ChannelRole};
use foh_core::propose::{MixMove, MoveKind};
use foh_core::session::{NamedStem, Session};
use crate::biquad::PeakEq;
use crate::dynamics::{Compressor, Gate};

#[derive(Debug, Clone)]
pub struct MoveApplication {
    pub applied: Vec<MixMove>,
    pub skipped: Vec<(MixMove, &'static str)>,
}

pub fn apply_moves(session: &Session, moves: &[MixMove]) -> (Session, MoveApplication) {
    let mut out = Session::new(session.sample_rate);
    let mut applied = Vec::new();
    let mut skipped = Vec::new();
    for stem in &session.stems {
        let mine: Vec<&MixMove> = moves.iter().filter(|m| m.channel == stem.name).collect();
        let mut samples = stem.samples.clone();
        for m in mine {
            match apply_one(session.sample_rate, stem.role, &mut samples, m) {
                Ok(()) => applied.push((*m).clone()),
                Err(why) => skipped.push(((*m).clone(), why)),
            }
        }
        out.push(stem.name.clone(), stem.role, samples);
    }
    for m in moves {
        if !session.stems.iter().any(|s| s.name == m.channel)
            && !skipped.iter().any(|(x, _)| x.channel == m.channel && x.kind == m.kind)
        {
            skipped.push((m.clone(), "unknown channel"));
        }
    }
    (out, MoveApplication { applied, skipped })
}

fn apply_one(sr: u32, _role: ChannelRole, samples: &mut [f32], m: &MixMove) -> Result<(), &'static str> {
    match m.kind {
        MoveKind::FaderDb | MoveKind::SendDb => {
            let g = lin_from_db(m.delta_db);
            for s in samples.iter_mut() {
                *s *= g;
            }
            Ok(())
        }
        MoveKind::EqPeak | MoveKind::EqCut => {
            let freq = m.freq_hz.ok_or("eq missing freq")?;
            let q = m.q.unwrap_or(1.0);
            let mut eq = PeakEq::new(sr, freq, q, m.delta_db);
            eq.process(samples);
            Ok(())
        }
        MoveKind::CompThresholdDb => {
            let mut c = Compressor::new(sr, m.delta_db, 4.0, 8.0, 80.0);
            c.process(samples);
            Ok(())
        }
        MoveKind::GateThresholdDb => {
            let mut g = Gate::new(sr, m.delta_db, 2.0, 80.0);
            g.process(samples);
            Ok(())
        }
    }
}

pub fn session_from_stems(sr: u32, stems: Vec<NamedStem>) -> Session {
    Session { sample_rate: sr, stems }
}
