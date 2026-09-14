use crate::analyze::MixReport;
use crate::audio::ChannelRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warn,
    Crit,
}

#[derive(Debug, Clone)]
pub struct Diagnosis {
    pub severity: Severity,
    pub code: &'static str,
    pub channel: String,
    pub detail: String,
}

pub fn diagnose(report: &MixReport) -> Vec<Diagnosis> {
    let mut out = Vec::new();
    for ch in &report.channels {
        let m = &ch.metrics;
        if m.peak_db > -1.0 {
            out.push(Diagnosis {
                severity: Severity::Crit,
                code: "clip_risk",
                channel: ch.name.clone(),
                detail: format!("peak {:.1} dBFS", m.peak_db),
            });
        } else if m.peak_db > -3.0 {
            out.push(Diagnosis {
                severity: Severity::Warn,
                code: "hot_peak",
                channel: ch.name.clone(),
                detail: format!("peak {:.1} dBFS", m.peak_db),
            });
        }
        if m.role.is_open_mic() {
            if let Some(hz) = m.ring_hz {
                if m.ring_prominence_db >= 10.0 {
                    out.push(Diagnosis {
                        severity: Severity::Crit,
                        code: "feedback_risk",
                        channel: ch.name.clone(),
                        detail: format!("standing peak {:.0} Hz, +{:.1} dB vs neighbors", hz, m.ring_prominence_db),
                    });
                } else if m.ring_prominence_db >= 8.0 {
                    out.push(Diagnosis {
                        severity: Severity::Warn,
                        code: "ring_mode",
                        channel: ch.name.clone(),
                        detail: format!("{:.0} Hz +{:.1} dB", hz, m.ring_prominence_db),
                    });
                }
            }
        }
        if m.role == ChannelRole::Vocal && m.activity > 0.15 && m.rms_db < -28.0 {
            out.push(Diagnosis {
                severity: Severity::Warn,
                code: "vocal_buried",
                channel: ch.name.clone(),
                detail: format!("vocal RMS {:.1} dBFS", m.rms_db),
            });
        }
        if m.role.is_low_end() && m.mud_db() > m.low_end_db() + 4.0 {
            out.push(Diagnosis {
                severity: Severity::Warn,
                code: "boxy_low_mids",
                channel: ch.name.clone(),
                detail: format!("250 Hz {:.1} dB vs low-end {:.1} dB", m.mud_db(), m.low_end_db()),
            });
        }
    }
    if let (Some(kick), Some(bass)) = (
        report.channels.iter().find(|c| c.metrics.role == ChannelRole::Kick),
        report.channels.iter().find(|c| c.metrics.role == ChannelRole::Bass),
    ) {
        let k63 = kick.metrics.octave_db[1];
        let b63 = bass.metrics.octave_db[1];
        let k125 = kick.metrics.octave_db[2];
        let b125 = bass.metrics.octave_db[2];
        if k63 > -50.0 && b63 > -50.0 && (k63 - b63).abs() < 4.0 {
            out.push(Diagnosis {
                severity: Severity::Warn,
                code: "kick_bass_63",
                channel: format!("{}/{}", kick.name, bass.name),
                detail: format!("both occupy 63 Hz (K {:.1} / B {:.1})", k63, b63),
            });
        }
        if k125 > -50.0 && b125 > -50.0 && (k125 - b125).abs() < 3.0 && k125 > k63 - 2.0 {
            out.push(Diagnosis {
                severity: Severity::Warn,
                code: "kick_bass_125",
                channel: format!("{}/{}", kick.name, bass.name),
                detail: format!("both occupy 125 Hz (K {:.1} / B {:.1})", k125, b125),
            });
        }
    }
    if let Some(vocal) = report.channels.iter().find(|c| c.metrics.role == ChannelRole::Vocal) {
        let v_pres = vocal.metrics.presence_db();
        let others_pres: f32 = report
            .channels
            .iter()
            .filter(|c| c.metrics.role != ChannelRole::Vocal && c.metrics.role != ChannelRole::Lr)
            .map(|c| c.metrics.presence_db())
            .fold(-120.0f32, f32::max);
        if vocal.metrics.activity > 0.1 && v_pres + 2.0 < others_pres {
            out.push(Diagnosis {
                severity: Severity::Warn,
                code: "vocal_masked",
                channel: vocal.name.clone(),
                detail: format!("vocal presence {:.1} dB vs competing {:.1} dB", v_pres, others_pres),
            });
        }
    }
    if report.kick_bass_corr.abs() > 0.85 {
        out.push(Diagnosis {
            severity: Severity::Info,
            code: "kick_bass_corr",
            channel: "kick/bass".into(),
            detail: format!("pearson {:+.2} — shared waveform, not just shared band", report.kick_bass_corr),
        });
    }
    if report.lr_peak_db > -1.0 {
        out.push(Diagnosis {
            severity: Severity::Crit,
            code: "lr_clip",
            channel: "LR".into(),
            detail: format!("mix peak {:.1} dBFS", report.lr_peak_db),
        });
    }
    out
}
