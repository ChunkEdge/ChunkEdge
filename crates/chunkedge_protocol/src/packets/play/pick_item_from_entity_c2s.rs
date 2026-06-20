use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct PickItemFromEntityC2s {
    pub entity_id: VarInt,
    pub include_data: bool,
}
