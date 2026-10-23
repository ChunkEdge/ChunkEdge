use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_nbt::Compound;

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TagQueryS2c {
    pub transaction_id: VarInt,
    /// Null is encoded as `TAG_End`, without an `Option` boolean prefix.
    #[packet(with = chunkedge_binary::nullable_compound)]
    pub nbt: Option<Compound>,
}
