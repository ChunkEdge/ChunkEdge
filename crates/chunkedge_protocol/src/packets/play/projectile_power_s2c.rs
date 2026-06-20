use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ProjectilePowerS2c {
    pub entity_id: VarInt,
    pub power: f64,
}
