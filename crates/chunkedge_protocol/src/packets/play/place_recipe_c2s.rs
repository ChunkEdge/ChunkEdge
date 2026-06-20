use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct PlaceRecipeC2s {
    pub window_id: VarInt,
    pub recipe: VarInt,
    pub make_all: bool,
}
