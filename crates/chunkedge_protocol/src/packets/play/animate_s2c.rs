use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct AnimateS2c {
    pub entity_id: VarInt,
    pub animation: u8,
}
