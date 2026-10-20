use chunkedge_binary::{Bounded, Decode, Encode, RawBytes, VarInt};
use chunkedge_protocol::ChunkPos;

use crate::Packet;

const MAX_PAYLOAD_SIZE: usize = 0x200000;

/// A debug subscription value update for a chunk. See [`DebugBlockValueS2c`](super::debug_block_value_s2c::DebugBlockValueS2c).
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugChunkValueS2c<'a> {
    pub chunk_pos: ChunkPos,
    pub subscription: VarInt,
    pub has_payload: bool,
    pub payload: Bounded<RawBytes<'a>, MAX_PAYLOAD_SIZE>,
}
