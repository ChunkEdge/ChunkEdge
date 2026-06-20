use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub enum ClientCommandC2s {
    PerformRespawn,
    RequestStats,
}
