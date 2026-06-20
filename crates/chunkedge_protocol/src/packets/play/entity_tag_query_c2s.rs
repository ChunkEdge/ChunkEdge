use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct EntityTagQueryC2s {
    pub transaction_id: VarInt,
    pub entity_id: VarInt,
}
