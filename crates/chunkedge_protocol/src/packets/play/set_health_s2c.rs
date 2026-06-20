use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetHealthS2c {
    pub health: f32,
    pub food: VarInt,
    pub food_saturation: f32,
}
