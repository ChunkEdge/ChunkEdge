use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct ContainerSetDataS2c {
    pub window_id: VarInt,
    pub property: i16,
    pub value: i16,
}
