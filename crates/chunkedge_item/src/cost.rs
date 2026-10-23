use std::io::Write;

use chunkedge_binary::{Decode, Encode, VarInt, cautious_capacity};

use crate::{ItemComponent, ItemKind, impls::decode_item_component};

/// An item and exact component predicates required by a merchant offer.
/// Unlike an `ItemStack`, the wire format starts with the item ID and contains
/// typed component values, with no removed-component list.
#[derive(Clone, PartialEq, Debug)]
pub struct ItemCost {
    pub item: ItemKind,
    pub count: VarInt,
    pub components: Vec<ItemComponent>,
}

impl Encode for ItemCost {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.item.encode(&mut w)?;
        self.count.encode(&mut w)?;
        let len = i32::try_from(self.components.len())?;
        VarInt(len).encode(&mut w)?;
        for component in &self.components {
            VarInt(component.id() as i32).encode(&mut w)?;
            component.encode(&mut w)?;
        }
        Ok(())
    }
}

impl Decode<'_> for ItemCost {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        let item = ItemKind::decode(r)?;
        let count = VarInt::decode(r)?;
        let len = VarInt::decode(r)?.0;
        anyhow::ensure!(len >= 0, "negative item cost component count");
        let mut components = Vec::with_capacity(cautious_capacity::<ItemComponent>(len as usize));
        for _ in 0..len {
            let id = VarInt::decode(r)?.0;
            anyhow::ensure!(id >= 0, "negative item cost component ID");
            components.push(decode_item_component(r, id as usize, 0)?);
        }
        Ok(Self {
            item,
            count,
            components,
        })
    }
}
