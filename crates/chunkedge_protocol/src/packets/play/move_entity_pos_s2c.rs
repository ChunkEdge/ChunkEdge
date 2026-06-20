use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MoveEntityPosS2c {
    pub entity_id: VarInt,
    pub delta: [i16; 3],
    pub on_ground: bool,
}
