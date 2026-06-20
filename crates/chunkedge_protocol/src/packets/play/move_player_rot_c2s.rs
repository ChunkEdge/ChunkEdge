use valence_binary::{Decode, Encode};

use crate::movement_flags::MovementFlags;
use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MovePlayerRotC2s {
    pub yaw: f32,
    pub pitch: f32,
    pub flags: MovementFlags,
}
