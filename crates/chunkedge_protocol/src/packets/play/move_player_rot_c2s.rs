use chunkedge_binary::{Decode, Encode};

use crate::Packet;
use crate::movement_flags::MovementFlags;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MovePlayerRotC2s {
    pub yaw: f32,
    pub pitch: f32,
    pub flags: MovementFlags,
}
