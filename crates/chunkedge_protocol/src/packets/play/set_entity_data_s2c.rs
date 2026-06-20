use valence_binary::{Decode, Encode, RawBytes, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetEntityDataS2c<'a> {
    pub entity_id: VarInt,
    pub tracked_values: RawBytes<'a>,
}
