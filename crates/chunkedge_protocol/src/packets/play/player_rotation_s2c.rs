use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct PlayerRotationS2c {
    pub yaw: f32,
    pub pitch: f32,
}
