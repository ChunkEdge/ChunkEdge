use chunkedge_binary::{Decode, Encode};
use chunkedge_item::ItemStack;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetCursorItemS2c {
    item: ItemStack,
}
