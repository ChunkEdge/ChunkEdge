use valence_binary::{Decode, Encode};
use valence_item::{decode_item_stack_recursive, ItemStack};

use crate::Packet;

#[derive(Clone, Debug, Packet)]
pub struct SetCreativeModeSlotC2s {
    pub slot: i16,
    pub clicked_item: ItemStack,
}

impl Decode<'_> for SetCreativeModeSlotC2s {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        Ok(Self {
            slot: i16::decode(r)?,
            clicked_item: decode_item_stack_recursive(r, 0, true)?,
        })
    }
}

impl Encode for SetCreativeModeSlotC2s {
    fn encode(&self, mut w: impl std::io::Write) -> anyhow::Result<()> {
        self.slot.encode(&mut w)?;
        self.clicked_item.encode_recursive(w, true)
    }
}
