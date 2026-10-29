use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_math::DVec3;

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct AcceptTeleportationC2s {
    pub teleport_id: VarInt,
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
}
