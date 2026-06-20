use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ContainerSlotStateChangedC2s {
    pub slot_id: VarInt,
    pub window_id: VarInt,
    pub state: bool,
}
