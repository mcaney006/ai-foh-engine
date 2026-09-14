//! Analysis, diagnosis, and verification for live multichannel mixes.
//!
//! This crate never talks to a console. It consumes PCM, emits measurements
//! and bounded mix proposals, then scores a before/after pair.

pub mod analyze;
pub mod fft;
pub mod audio;
pub mod bands;
pub mod chanmap;
pub mod corr;
pub mod diagnose;
pub mod metrics;
pub mod propose;
pub mod session;
pub mod verify;

pub use analyze::{analyze_buffer, analyze_session, ChannelReport, MixReport};
pub use audio::{AudioError, Buffer, ChannelRole};
pub use diagnose::{Diagnosis, Severity};
pub use metrics::ChannelMetrics;
pub use propose::{MixMove, MoveKind, Proposal};
pub use session::{NamedStem, Session};
pub use verify::{VerifyDecision, VerifyReport};

pub const DEFAULT_SR: u32 = 48_000;
pub const OCTAVE_CENTERS_HZ: [f32; 10] =
    [31.5, 63.0, 125.0, 250.0, 500.0, 1_000.0, 2_000.0, 4_000.0, 8_000.0, 16_000.0];

pub use chanmap::{ChannelMap, MapEntry, parse_role};
