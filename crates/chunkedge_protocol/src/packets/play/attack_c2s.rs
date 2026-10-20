use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Sent when a player attacks an entity in creative-ish contexts. Split out
/// of [`InteractC2s`](super::interact_c2s::InteractC2s) in 26.1.
///
/// wiki: [Attack](https://minecraft.wiki/w/Java_Edition_protocol#Attack)
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct AttackC2s {
    pub entity_id: VarInt,
}
