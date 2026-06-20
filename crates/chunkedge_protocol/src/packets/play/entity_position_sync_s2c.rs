use valence_binary::{Decode, Encode, VarInt};
use valence_math::DVec3;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct EntityPositionSyncS2c {
    pub entity_id: VarInt,
    pub position: DVec3,
    pub velocity: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
