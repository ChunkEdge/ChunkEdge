use chunkedge_binary::{Bounded, Decode, Encode, RawBytes, VarInt};
use chunkedge_protocol::BlockPos;

use crate::Packet;

const MAX_PAYLOAD_SIZE: usize = 0x200000;

/// A debug subscription value update for a block position. The `payload` is
/// the subscription-specific value encoding; see the vanilla
/// `DebugSubscription` registry for per-type shapes.
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugBlockValueS2c<'a> {
    pub block_pos: BlockPos,
    pub subscription: VarInt,
    pub has_payload: bool,
    pub payload: Bounded<RawBytes<'a>, MAX_PAYLOAD_SIZE>,
}
