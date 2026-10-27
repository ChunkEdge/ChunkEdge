use chunkedge_binary::{Decode, Encode, VarInt};

use crate::{LpVec3, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetEntityMotionS2c {
    pub entity_id: VarInt,
    pub velocity: LpVec3,
}
