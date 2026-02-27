mod components;
mod impls;
mod stack;
mod vanilla_components;

pub(crate) const NUM_ITEM_COMPONENTS: usize = 96;
/// Controls the maximum recursion depth for encoding and decoding item
/// components.
pub(crate) const MAX_RECURSION_DEPTH: usize = 16;

pub use valence_generated::item::ItemKind;

pub use crate::components::ItemComponent;
pub use crate::impls::decode_item_stack_recursive;
pub use crate::stack::{HashedItemStack, ItemStack};

#[cfg(test)]
mod tests {
    use valence_binary::{Decode, Encode, VarInt};
    use valence_generated::attributes::EntityAttributeOperation;
    use valence_generated::item::ItemKind;
    use valence_generated::registry_id::RegistryId;
    use valence_ident::ident;
    use valence_nbt::Compound;
    use valence_text::Text;

    use super::*;
    use crate::components::{
        AttributeModifier, AttributeSlot, DyeColor, ModePair, Patchable, PropertyValue, Rarity,
    };

    // --- Helpers ---

    fn create_test_stack(item: ItemKind, count: i8) -> ItemStack {
        ItemStack::new(item, count)
    }

    fn roundtrip<T: Encode + for<'a> Decode<'a> + PartialEq + std::fmt::Debug>(val: &T) {
        let mut buf = Vec::new();
        val.encode(&mut buf).expect("Failed to encode");
        let mut slice = buf.as_slice();
        let decoded = T::decode(&mut slice).expect("Failed to decode");
        assert_eq!(val, &decoded, "Roundtrip failed equality check");
        assert!(slice.is_empty(), "Buffer not fully consumed");
    }

    // --- Patchable Tests ---

    #[test]
    fn test_patchable_logic() {
        let p_added = Patchable::Added((Box::new(10), 123));
        let p_default = Patchable::Default(Box::new(5));
        let p_removed: Patchable<Box<i32>> = Patchable::Removed;
        let p_none: Patchable<Box<i32>> = Patchable::None;

        assert_eq!(p_added.as_option().map(|v| **v), Some(10));
        assert_eq!(p_default.as_option().map(|v| **v), Some(5));
        assert_eq!(p_removed.as_option(), None);
        assert_eq!(p_none.as_option(), None);
    }

    // --- ItemStack Logical Tests ---

    #[test]
    fn test_item_stack_empty() {
        let empty = ItemStack::EMPTY;
        assert!(empty.is_empty());

        let air = ItemStack::new(ItemKind::Air, 1);
        assert!(air.is_empty());

        let stack = ItemStack::new(ItemKind::Diamond, 0);
        assert!(stack.is_empty());

        let stack = ItemStack::new(ItemKind::Diamond, -1);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_component_insertion_and_removal() {
        let mut stack = create_test_stack(ItemKind::Diamond, 1);

        // Test Insert
        let custom_name = ItemComponent::CustomName(Text::from("Test Item").into());
        stack.insert_component(custom_name.clone());

        assert_eq!(stack.get_component(5_usize), Some(&custom_name));
        assert_eq!(stack.components().len(), 1);

        // Test Remove
        let removed = stack.remove_component(5_usize);
        assert_eq!(removed, Some(custom_name));
        assert_eq!(stack.get_component(5_usize), None);

        // Ensure "Removed" patch is applied (important for serialization)
        assert!(matches!(stack.components[5], Patchable::Removed));
    }

    // --- Serialization Roundtrips ---

    #[test]
    fn test_serialization_basic_stack() {
        let stack = ItemStack::new(ItemKind::Stone, 32);
        roundtrip(&stack);
    }

    #[test]
    fn test_serialization_with_complex_components() {
        let mut stack = ItemStack::new(ItemKind::DiamondSword, 1);

        // Add multiple types of components
        stack.insert_component(ItemComponent::Damage(VarInt(50)));
        stack.insert_component(ItemComponent::Rarity(Rarity::Epic));

        // Test vec-based components (Enchantments)
        stack.insert_component(ItemComponent::Enchantments(vec![(
            RegistryId::new(1),
            VarInt(5),
        )]));

        roundtrip(&stack);
    }

    #[test]
    fn test_serialization_removed_components() {
        // Start with a stack that has a component, then remove it
        let mut stack = ItemStack::new(ItemKind::Diamond, 1);
        stack.insert_component(ItemComponent::Unbreakable);
        stack.remove_component(ItemComponent::Unbreakable.id() as usize);

        roundtrip(&stack);
    }

    #[test]
    fn test_mode_pair_serialization() {
        let m0 = ModePair::<String, RegistryId>::Mode0("minecraft:standard".to_owned());
        let m1 = ModePair::<String, RegistryId>::Mode1(RegistryId::new(1));

        roundtrip(&m0);
        roundtrip(&m1);
    }

    #[test]
    fn test_property_value_serialization() {
        let exact = PropertyValue::Exact("true".into());
        let min_max = PropertyValue::MinMax {
            min: "1".into(),
            max: "5".into(),
        };

        roundtrip(&exact);
        roundtrip(&min_max);
    }

    // --- Recursion and Nested Components ---

    #[test]
    fn test_nested_container_serialization() {
        let mut inner_stack = ItemStack::new(ItemKind::Apple, 1);
        inner_stack.insert_component(ItemComponent::ItemName(Text::from("Inner").into()));

        let mut outer_stack = ItemStack::new(ItemKind::Chest, 1);
        outer_stack.insert_component(ItemComponent::Container(vec![inner_stack]));

        roundtrip(&outer_stack);
    }

    #[test]
    fn test_recursion_limit() {
        let mut buf = Vec::new();

        // Helper to write a recursive bundle structure manually
        fn write_recursive_bundle(w: &mut Vec<u8>, depth: usize) {
            VarInt(1).encode(&mut *w).unwrap(); // Count
            ItemKind::Bundle.encode(&mut *w).unwrap(); // Item

            VarInt(1).encode(&mut *w).unwrap(); // Added components count
            VarInt(0).encode(&mut *w).unwrap(); // Removed components count

            VarInt(41).encode(&mut *w).unwrap(); // Component ID: BundleContents

            if depth > 0 {
                VarInt(1).encode(&mut *w).unwrap(); // Nested list length
                write_recursive_bundle(w, depth - 1);
            } else {
                VarInt(0).encode(&mut *w).unwrap(); // Empty nested list
            }
        }

        write_recursive_bundle(&mut buf, 20); // 20 > 16

        let mut slice = buf.as_slice();
        let result = ItemStack::decode(&mut slice);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("recursion limit exceeded"));
    }

    // --- HashedItemStack Tests ---

    #[test]
    fn test_hashed_item_stack_roundtrip() {
        let mut hashed = HashedItemStack::EMPTY;
        hashed.item = ItemKind::IronIngot;
        hashed.count = 10;
        // In real use, these would be crc hashes
        hashed.components[1] = Patchable::Added(((), 123456));

        roundtrip(&hashed);
    }

    #[test]
    fn test_hashed_item_stack_empty() {
        let hashed = HashedItemStack::EMPTY;
        let mut buf = Vec::new();
        hashed.encode(&mut buf).unwrap();

        let mut slice = buf.as_slice();
        let decoded = HashedItemStack::decode(&mut slice).unwrap();
        assert!(decoded.is_empty());
    }

    // --- Edge Cases ---

    #[test]
    fn test_invalid_component_id() {
        let mut buf = Vec::new();
        VarInt(1).encode(&mut buf).unwrap(); // Count
        ItemKind::Stone.encode(&mut buf).unwrap(); // Item
        VarInt(1).encode(&mut buf).unwrap(); // Added count
        VarInt(999).encode(&mut buf).unwrap(); // INVALID ID

        let mut slice = buf.as_slice();
        let result = ItemStack::decode(&mut slice);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_item_component_ids() {
        let mut ids = std::collections::HashSet::new();
        let components = vec![
            ItemComponent::CustomData(Compound::default()),
            ItemComponent::MaxStackSize(VarInt(64)),
            ItemComponent::Unbreakable,
            ItemComponent::Glider,
            ItemComponent::ShulkerColor(DyeColor::Black),
        ];

        for comp in components {
            let id = comp.id();
            assert!(id < NUM_ITEM_COMPONENTS as u32);
            ids.insert(id);
        }

        assert!(ids.contains(&4));
        assert!(ids.contains(&30));
    }

    #[test]
    fn test_food_component_serialization() {
        let food = ItemComponent::Food {
            nutrition: VarInt(4),
            saturation_modifier: 0.5,
            can_always_eat: true,
        };
        let mut stack = ItemStack::new(ItemKind::Apple, 1);
        stack.insert_component(food);
        roundtrip(&stack);
    }

    #[test]
    fn test_attribute_modifiers_serialization() {
        let modifier = AttributeModifier {
            attribute_id: RegistryId::new(0),
            modifier_id: ident!("test_mod").into(),
            value: 5.0,
            operation: EntityAttributeOperation::Add,
            slot: AttributeSlot::MainHand,
        };

        let comp = ItemComponent::AttributeModifiers {
            modifiers: vec![modifier],
        };

        let mut stack = ItemStack::new(ItemKind::NetheriteSword, 1);
        stack.insert_component(comp);
        roundtrip(&stack);
    }
}
