use std::env;

use foh_control::digico::{db_to_fader_01, CommandSet, DigicoCommand};
use foh_control::journal;
use foh_control::safety::SafetyGate;
use foh_core::analyze::analyze_session;
use foh_core::audio::ChannelRole;
use foh_core::chanmap::ChannelMap;
use foh_core::diagnose::diagnose;
use foh_core::propose::propose;
use foh_core::session::Session;
use foh_core::verify::{verify, VerifyDecision};
use foh_core::DEFAULT_SR;
use foh_mix::apply_proposal;

mod demo;

struct Args {
    stems: Vec<String>,
    sample_rate: u32,
    osc_set: CommandSet,
    json: bool,
    map_path: Option<String>,
}

fn parse_args() -> Args {
    let mut stems = Vec::new();
    let mut sample_rate = DEFAULT_SR;
    let mut osc_set = CommandSet::Ipad;
    let mut json = false;
    let mut map_path = None;
    let mut it = env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--stem" => {
                if let Some(v) = it.next() {
                    stems.push(v);
                }
            }
            "--sample-rate" => {
                if let Some(v) = it.next() {
                    sample_rate = v.parse().unwrap_or(DEFAULT_SR);
                }
            }
            "--osc-set" => {
                osc_set = match it.next().as_deref() {
                    Some("osc") => CommandSet::Osc,
                    Some("s") => CommandSet::SSeries,
                    _ => CommandSet::Ipad,
                };
            }
            "--json" => json = true,
            "--map" => map_path = it.next(),
            "--help" | "-h" => {
                eprintln!(
                    "foh-engine [--json] [--map file] [--stem name=file.wav] [--osc-set ipad|osc|s] [--sample-rate 48000]"
                );
                std::process::exit(0);
            }
            other => eprintln!("unknown arg {other}"),
        }
    }
    Args {
        stems,
        sample_rate,
        osc_set,
        json,
        map_path,
    }
}

fn main() {
    let args = parse_args();
    let mut session = if args.stems.is_empty() {
        demo::problem_mix(args.sample_rate)
    } else {
        load_stems(&args.stems, args.sample_rate)
    };

    let cmap = match args.map_path.as_deref() {
        Some(path) => {
            let text = std::fs::read_to_string(path).unwrap_or_default();
            ChannelMap::parse(&text).unwrap_or_default()
        }
        None => ChannelMap::parse(demo::DEFAULT_MAP).unwrap_or_default(),
    };
    for stem in session.stems.iter_mut() {
        if let Some(role) = cmap.role(&stem.name) {
            stem.role = role;
        }
    }

    let before = analyze_session(&session);
    let diagnoses = diagnose(&before);
    let mut proposal = propose(&before, &diagnoses);

    let mut gate = SafetyGate::default();
    let mut allowed = Vec::new();
    for (i, mv) in proposal.moves.iter().enumerate() {
        match gate.check(i as u64 * 1_000, mv) {
            Ok(()) => allowed.push(mv.clone()),
            Err(e) => eprintln!("safety skip {}/{:?}: {e}", mv.channel, mv.kind),
        }
    }
    proposal.moves = allowed;

    let processed = apply_proposal(&session, &proposal);
    let after = analyze_session(&processed.session);
    let verdict = verify(&before, &after);

    if args.json {
        println!("decision={:?}", verdict.decision);
        for d in &diagnoses {
            println!("diag {} {} {}", d.code, d.channel, d.detail.replace("\n", " "));
        }
        for m in &proposal.moves {
            println!("move {:?} {} {:+.2}", m.kind, m.channel, m.delta_db);
        }
        return;
    }

    println!("AI FOH Engine");
    println!(
        "stems={}  sr={}  dur={:.2}s",
        session.stems.len(),
        session.sample_rate,
        before.duration_secs
    );
    println!();
    println!("LISTEN");
    for ch in &before.channels {
        let m = &ch.metrics;
        print!(
            "  {:<10} {:>7.1} peak  {:>7.1} rms  crest {:>5.1}",
            ch.name, m.peak_db, m.rms_db, m.crest_db
        );
        if let Some(hz) = m.ring_hz {
            print!("  ring {:.0}Hz {:+.1}dB", hz, m.ring_prominence_db);
        }
        println!();
    }
    println!(
        "  {:<10} {:>7.1} peak  {:>7.1} rms  kick/bass r={:+.2}",
        "LR", before.lr_peak_db, before.lr_rms_db, before.kick_bass_corr
    );

    println!();
    println!("REASON");
    if diagnoses.is_empty() {
        println!("  no flags");
    } else {
        for d in &diagnoses {
            println!(
                "  [{:?}] {}  {} — {}",
                d.severity, d.code, d.channel, d.detail
            );
        }
    }

    println!();
    println!("MIX (offline strip; OSC dry-run for faders only)");
    if proposal.moves.is_empty() {
        println!("  no moves");
    } else {
        for (i, m) in proposal.moves.iter().enumerate() {
            let extra = match (m.freq_hz, m.q) {
                (Some(f), Some(q)) => format!("  {f:.0} Hz Q{q:.1}"),
                _ => String::new(),
            };
            println!(
                "  {:>2}. {:<10} {:<12} {:+.1} dB{}  ({})",
                i + 1,
                m.channel,
                format!("{:?}", m.kind),
                m.delta_db,
                extra,
                m.reason
            );
            if matches!(m.kind, foh_core::propose::MoveKind::FaderDb) {
                let ch = cmap.desk_channel(&m.channel).unwrap_or(1);
                let cmd = DigicoCommand::Fader {
                    channel: ch,
                    level_01: db_to_fader_01(m.delta_db),
                };
                if let Ok((addr, _)) = cmd.address(args.osc_set) {
                    println!("      osc {addr}");
                }
            }
        }
    }

    println!();
    println!("VERIFY {:?}", verdict.decision);
    println!(
        "  LR peak {:+.2} dB   vocal RMS {:+.2} dB   feedback_worse={}",
        verdict.peak_delta_db, verdict.vocal_rms_delta_db, verdict.feedback_worse
    );
    for n in &verdict.notes {
        println!("  [{:?}] {} — {}", n.severity, n.code, n.detail);
    }
    if verdict.decision == VerifyDecision::Reject {
        println!("  changes discarded for live send");
    }

    let mut cmds = Vec::new();
    if verdict.decision == VerifyDecision::Accept {
        for m in &proposal.moves {
            if let Some(ch) = cmap.desk_channel(&m.channel) {
                if let Some(cmd) = DigicoCommand::from_move(ch, 0.0, m) {
                    cmds.push(cmd);
                }
            }
        }
    }
    if let Ok(lines) = journal::preview(args.osc_set, &cmds) {
        println!();
        if lines.is_empty() {
            println!("# OSC dry-run — not sent");
            println!("# no public fader/mute/snapshot path for the accepted moves");
            println!("# EQ/dynamics stay on the offline strip (docs/CONTROL.md)");
        } else {
            print!("{}", journal::render(&lines));
        }
    }
}

fn load_stems(specs: &[String], fallback_sr: u32) -> Session {
    let mut session = Session::new(fallback_sr);
    for spec in specs {
        let (name, path) = spec.split_once('=').unwrap_or(("stem", spec.as_str()));
        match foh_core::audio::Buffer::from_wav(std::path::Path::new(path)) {
            Ok(buf) => {
                session.sample_rate = buf.sample_rate;
                session.push(name, guess_role(name), buf.channels[0].clone());
            }
            Err(e) => eprintln!("skip {path}: {e}"),
        }
    }
    session
}

fn guess_role(name: &str) -> ChannelRole {
    let n = name.to_ascii_lowercase();
    if n.contains("kick") {
        ChannelRole::Kick
    } else if n.contains("bass") {
        ChannelRole::Bass
    } else if n.contains("snare") {
        ChannelRole::Snare
    } else if n.contains("hat") {
        ChannelRole::Hats
    } else if n.contains("vox") || n.contains("vocal") {
        ChannelRole::Vocal
    } else if n.contains("bv") {
        ChannelRole::VocalBv
    } else if n.contains("gtr") || n.contains("guitar") {
        ChannelRole::Guitar
    } else if n.contains("key") {
        ChannelRole::Keys
    } else {
        ChannelRole::Unknown
    }
}
