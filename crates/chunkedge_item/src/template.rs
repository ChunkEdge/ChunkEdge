use std::borrow::Borrow;
use std::io::Write;

use chunkedge_binary::{Decode, Encode};

use crate::ItemStack;

/// A non-empty item stack encoded as an item ID, count, and component patch.
/// Accepts an owned or borrowed [`ItemStack`].
#[derive(Clone, PartialEq, Debug)]
pub struct ItemStackTemplate<I = ItemStack>(pub I);

impl<I: Borrow<ItemStack>> Encode for ItemStackTemplate<I> {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        self.0.borrow().encode_template(w)
    }
}

impl<'a> Decode<'a> for ItemStackTemplate {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        crate::impls::decode_item_stack_template(r, 0).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use chunkedge_binary::VarInt;

    use super::*;
    use crate::{ItemComponent, ItemKind};

    #[test]
    fn template_wire_order_and_components() {
        let mut stack = ItemStack::new(ItemKind::IronSword, 1);
        stack.insert_component(ItemComponent::Damage(VarInt(7)));
        stack.remove_component(ItemComponent::MapId(VarInt(0)).id() as usize);

        let mut bytes = Vec::new();
        ItemStackTemplate(&stack).encode(&mut bytes).unwrap();

        // Read the fields in vanilla's ItemStackTemplate.STREAM_CODEC order.
        let mut r = bytes.as_slice();
        assert_eq!(
            VarInt::decode(&mut r).unwrap().0,
            i32::from(stack.item.to_raw())
        );
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 1);
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 1); // added components
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 1); // removed components
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 3); // damage
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 7);
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 48); // map_id in 26.3
        assert!(r.is_empty());

        let mut r = bytes.as_slice();
        assert_eq!(ItemStackTemplate::decode(&mut r).unwrap().0, stack);
        assert!(r.is_empty());

        let mut regular_bytes = Vec::new();
        stack.encode(&mut regular_bytes).unwrap();
        let mut r = regular_bytes.as_slice();
        assert_eq!(VarInt::decode(&mut r).unwrap().0, 1);
        assert_eq!(
            VarInt::decode(&mut r).unwrap().0,
            i32::from(stack.item.to_raw())
        );
        assert_eq!(r, &bytes[bytes.len() - r.len()..]);
        assert_eq!(
            ItemStack::decode(&mut regular_bytes.as_slice()).unwrap(),
            stack
        );
    }

    #[test]
    fn template_rejects_empty_stacks() {
        for stack in [ItemStack::EMPTY, ItemStack::new(ItemKind::Stone, 0)] {
            assert!(ItemStackTemplate(stack).encode(Vec::new()).is_err());
        }
        for (item, count) in [(ItemKind::Air, 1), (ItemKind::Stone, 0)] {
            let mut bytes = Vec::new();
            item.encode(&mut bytes).unwrap();
            VarInt(count).encode(&mut bytes).unwrap();
            assert!(ItemStackTemplate::decode(&mut bytes.as_slice()).is_err());
        }
    }
}
