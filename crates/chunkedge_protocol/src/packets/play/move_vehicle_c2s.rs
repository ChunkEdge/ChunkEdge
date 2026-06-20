use chunkedge_binary::{Decode, Encode};
use chunkedge_math::DVec3;

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MoveVehicleC2s {
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
