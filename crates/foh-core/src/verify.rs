use crate::analyze::MixReport;
use crate::audio::ChannelRole;
use crate::diagnose::Severity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyDecision {
    Accept,
    Reject,
}

#[derive(Debug, Clone)]
pub struct VerifyNote {
    pub severity: Severity,
    pub code: &'static str,
    pub detail: String,
}

#[derive(Debug, Clone)]
pub struct VerifyReport {
    pub decision: VerifyDecision,
    pub notes: Vec<VerifyNote>,
    pub peak_delta_db: f32,
    pub vocal_rms_delta_db: f32,
    pub feedback_worse: bool,
}

pub fn verify(before: &MixReport, after: &MixReport) -> VerifyReport {
    let mut notes = Vec::new();
    let peak_delta = after.lr_peak_db - before.lr_peak_db;

    if after.lr_peak_db > 0.0 {
        notes.push(VerifyNote {
            severity: Severity::Crit,
            code: "after_clip",
            detail: format!("LR peak {:.1} dBFS", after.lr_peak_db),
        });
    }
    if peak_delta > 1.5 && after.lr_peak_db > -3.0 {
        notes.push(VerifyNote {
            severity: Severity::Warn,
            code: "got_louder_hot",
            detail: format!("LR peak {peak_delta:+.1} dB to {:.1}", after.lr_peak_db),
        });
    }

    let vocal_before = before.channels.iter().find(|c| c.metrics.role == ChannelRole::Vocal).map(|c| c.metrics.rms_db);
    let vocal_after = after.channels.iter().find(|c| c.metrics.role == ChannelRole::Vocal).map(|c| c.metrics.rms_db);
    let vocal_delta = match (vocal_before, vocal_after) {
        (Some(a), Some(b)) => b - a,
        _ => 0.0,
    };

    let mut feedback_worse = false;
    for (b, a) in before.channels.iter().zip(after.channels.iter()) {
        if !b.metrics.role.is_open_mic() {
            continue;
        }
        let before_p = if b.metrics.ring_hz.is_some() { b.metrics.ring_prominence_db } else { 0.0 };
        let after_p = if a.metrics.ring_hz.is_some() { a.metrics.ring_prominence_db } else { 0.0 };
        if after_p > before_p + 2.0 && after_p >= 8.0 {
            feedback_worse = true;
            notes.push(VerifyNote {
                severity: Severity::Crit,
                code: "feedback_worse",
                detail: format!("{} ring {:+.1} dB prominence ({:.0} Hz)", a.name, after_p - before_p, a.metrics.ring_hz.unwrap_or(0.0)),
            });
        }
    }

    let vocal_ring_improved = before
        .channels
        .iter()
        .find(|c| c.metrics.role == ChannelRole::Vocal)
        .map(|b| b.metrics.ring_hz.is_some() && b.metrics.ring_prominence_db >= 8.0)
        .unwrap_or(false);

    if vocal_before.is_some() && vocal_delta < -1.5 && !vocal_ring_improved {
        notes.push(VerifyNote {
            severity: Severity::Warn,
            code: "vocal_lost",
            detail: format!("vocal RMS {vocal_delta:+.1} dB"),
        });
    }

    let reject = notes.iter().any(|n| n.severity == Severity::Crit)
        || notes.iter().filter(|n| n.severity == Severity::Warn).count() >= 2;

    VerifyReport {
        decision: if reject { VerifyDecision::Reject } else { VerifyDecision::Accept },
        notes,
        peak_delta_db: peak_delta,
        vocal_rms_delta_db: vocal_delta,
        feedback_worse,
    }
}
