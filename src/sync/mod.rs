use datachannel::{IceCandidate, SessionDescription};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerMsg {
    pub dest_id: Uuid,
    pub kind: MsgKind,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MsgKind {
    Description(SessionDescription),
    Candidate(IceCandidate),
}
