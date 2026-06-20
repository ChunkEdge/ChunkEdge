use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ChunkBatchReceivedC2s {
    pub chunks_per_tick: f32,
}
