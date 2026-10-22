use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Sent when a player attacks an entity.
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct AttackC2s {
    pub entity_id: VarInt,
}
