use chunkedge_binary::{Bounded, Decode, Encode, RawBytes, VarInt};

use crate::Packet;

const MAX_PAYLOAD_SIZE: usize = 0x200000;

/// A debug subscription value update for an entity. See [`DebugBlockValueS2c`](super::debug_block_value_s2c::DebugBlockValueS2c).
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugEntityValueS2c<'a> {
    pub entity_id: VarInt,
    pub subscription: VarInt,
    pub has_payload: bool,
    pub payload: Bounded<RawBytes<'a>, MAX_PAYLOAD_SIZE>,
}
