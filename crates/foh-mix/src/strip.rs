use foh_core::propose::Proposal;
use foh_core::session::Session;
use crate::{apply_moves, MoveApplication};

#[derive(Debug, Clone, Default)]
pub struct ChannelStrip {
    pub fader_db: f32,
}

pub struct ProcessedSession {
    pub session: Session,
    pub application: MoveApplication,
}

pub fn apply_proposal(session: &Session, proposal: &Proposal) -> ProcessedSession {
    let (session, application) = apply_moves(session, &proposal.moves);
    ProcessedSession { session, application }
}
