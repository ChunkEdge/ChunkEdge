use chunkedge_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct PlayerRotationS2c {
    pub yaw: f32,
    pub relative_yaw: bool,
    pub pitch: f32,
    pub relative_pitch: bool,
}
