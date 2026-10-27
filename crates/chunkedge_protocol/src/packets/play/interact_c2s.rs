use chunkedge_binary::{Decode, Encode, VarInt};

use crate::{Hand, LpVec3, Packet};
use chunkedge_math::DVec3;

/// Sent when a player interacts with (right-clicks) an entity.
/// Attacks use [`AttackC2s`](super::attack_c2s::AttackC2s).
///
/// The `target` is a lossily quantized vector relative to the entity.
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct InteractC2s {
    pub entity_id: VarInt,
    pub hand: Hand,
    pub target: LpVec3,
    pub sneaking: bool,
}

impl InteractC2s {
    pub fn target_vec(&self) -> DVec3 {
        DVec3::from_array(self.target.0)
    }
}
