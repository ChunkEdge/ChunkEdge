use valence_binary::{Decode, Encode, VarInt};
use valence_nbt::Compound;

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TagQueryS2c {
    pub transaction_id: VarInt,
    pub nbt: Compound,
}
