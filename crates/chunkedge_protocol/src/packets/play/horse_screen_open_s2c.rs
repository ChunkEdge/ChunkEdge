use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct HorseScreenOpenS2c {
    pub window_id: VarInt,
    pub slot_count: VarInt,
    pub entity_id: i32,
}
