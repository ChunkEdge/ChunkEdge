use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ChunkBatchFinishedS2c {
    pub batch_size: VarInt,
}
