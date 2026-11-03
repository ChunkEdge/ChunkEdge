use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_math::DVec3;

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetEntityMotionS2c {
    pub entity_id: VarInt,
    /// Velocity in blocks per tick.
    #[packet(with = crate::lp_vec3)]
    pub velocity: DVec3,
}
