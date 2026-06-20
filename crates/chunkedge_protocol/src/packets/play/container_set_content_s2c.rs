use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_item::ItemStack;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ContainerSetContentS2c<'a> {
    pub window_id: VarInt,
    pub state_id: VarInt,
    pub slots: Cow<'a, [ItemStack]>,
    pub carried_item: Cow<'a, ItemStack>,
}
