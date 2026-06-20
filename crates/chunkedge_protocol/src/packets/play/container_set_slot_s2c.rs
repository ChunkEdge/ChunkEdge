use std::borrow::Cow;

use valence_binary::{Decode, Encode, VarInt};
use valence_item::ItemStack;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ContainerSetSlotS2c<'a> {
    pub window_id: VarInt,
    pub state_id: VarInt,
    pub slot_idx: i16,
    pub slot_data: Cow<'a, ItemStack>,
}
