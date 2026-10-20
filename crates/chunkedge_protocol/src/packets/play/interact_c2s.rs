use chunkedge_binary::{Decode, Encode, VarInt};

use crate::{Hand, Packet, Velocity};
use chunkedge_math::DVec3;

/// Sent when a player interacts with (right-clicks) an entity. In 26.1 the
/// attack action was split out into [`AttackC2s`](super::attack_c2s::AttackC2s),
/// so this packet is now always an interaction.
///
/// The `target` is a lossily quantized vector relative to the entity. It
/// shares [`Velocity`]'s wire encoding..
///
/// wiki: [Interact](https://minecraft.wiki/w/Java_Edition_protocol#Interact)
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct InteractC2s {
    pub entity_id: VarInt,
    pub hand: Hand,
    pub target: Velocity,
    pub sneaking: bool,
}

impl InteractC2s {
    pub fn target_vec(&self) -> DVec3 {
        DVec3::from_array(self.target.0.map(|v| f64::from(v) / 8000.0))
    }
}
