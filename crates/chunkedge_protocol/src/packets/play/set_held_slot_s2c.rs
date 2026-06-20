use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetHeldSlotS2c {
    pub slot: VarInt,
}
