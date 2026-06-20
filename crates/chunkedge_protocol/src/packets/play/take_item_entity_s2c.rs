use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct TakeItemEntityS2c {
    pub collected_entity_id: VarInt,
    pub collector_entity_id: VarInt,
    pub pickup_item_count: VarInt,
}
