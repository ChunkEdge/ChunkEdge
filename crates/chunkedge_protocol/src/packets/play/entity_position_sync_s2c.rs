use crate::entity_movement::PositionPath;
use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct EntityPositionSyncS2c {
    pub entity_id: VarInt,
    pub position: PositionPath,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
