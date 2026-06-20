use std::borrow::Cow;

use valence_binary::{Decode, Encode, VarInt};
use valence_item::ItemStack;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetPlayerInventoryS2c<'a> {
    pub slot: VarInt,
    pub slot_data: Cow<'a, ItemStack>,
}
