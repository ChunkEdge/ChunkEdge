use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_math::DVec3;

use crate::packets::play::player_position_s2c::TeleportRelativeFlags;
use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct TeleportEntityS2c {
    pub entity_id: VarInt,
    pub position: DVec3,
    pub velocity: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: TeleportRelativeFlags,
    pub on_ground: bool,
}
