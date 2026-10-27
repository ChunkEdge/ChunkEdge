mod components;
mod cost;
mod impls;
mod stack;
mod template;
mod vanilla_components;

pub(crate) const NUM_ITEM_COMPONENTS: usize = chunkedge_generated::item_component_type::COUNT;
/// Controls the maximum recursion depth for encoding and decoding item
/// components.
pub(crate) const MAX_RECURSION_DEPTH: usize = 16;

pub use chunkedge_generated::item::ItemKind;

pub use crate::components::*;
pub use crate::cost::ItemCost;
pub use crate::impls::decode_item_stack_recursive;
pub use crate::stack::{HashedItemStack, ItemStack};
pub use crate::template::ItemStackTemplate;

#[cfg(test)]
mod tests {
    use chunkedge_binary::{Decode, Encode, VarInt};
    use chunkedge_generated::attributes::EntityAttributeOperation;
    use chunkedge_generated::item::ItemKind;
    use chunkedge_generated::registry_id::RegistryId;
    use chunkedge_ident::ident;
    use chunkedge_nbt::Compound;
    use chunkedge_text::Text;

    use super::*;
    use crate::components::{
        AttributeModifier, AttributeSlot, DyeColor, Patchable, PropertyValue, Rarity,
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
        let custom_name_id = custom_name.id() as usize;
        stack.insert_component(custom_name.clone());

        assert_eq!(stack.get_component(custom_name_id), Some(&custom_name));
        assert_eq!(stack.components().len(), 1);

        // Test Remove
        let removed = stack.remove_component(custom_name_id);
        assert_eq!(removed, Some(custom_name));
        assert_eq!(stack.get_component(custom_name_id), None);

        // Ensure "Removed" patch is applied (important for serialization)
        assert!(matches!(
            stack.components[custom_name_id],
            Patchable::Removed
        ));
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
    fn test_property_value_serialization() {
        let exact = PropertyValue::Exact("true".into());
        let min_max = PropertyValue::MinMax {
            min: Some("1".into()),
            max: None,
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
        outer_stack.insert_component(ItemComponent::Container(vec![Some(ItemStackTemplate(
            inner_stack,
        ))]));

        roundtrip(&outer_stack);
    }

    #[test]
    fn test_recursion_limit() {
        let mut buf = Vec::new();

        // Helper to write a recursive bundle structure manually
        fn write_recursive_bundle(w: &mut Vec<u8>, depth: usize, template: bool) {
            if template {
                ItemKind::Bundle.encode(&mut *w).unwrap();
                VarInt(1).encode(&mut *w).unwrap();
            } else {
                VarInt(1).encode(&mut *w).unwrap();
                ItemKind::Bundle.encode(&mut *w).unwrap();
            }

            VarInt(1).encode(&mut *w).unwrap(); // Added components count
            VarInt(0).encode(&mut *w).unwrap(); // Removed components count

            VarInt(chunkedge_generated::item_component_type::BUNDLE_CONTENTS as i32)
                .encode(&mut *w)
                .unwrap();

            if depth > 0 {
                VarInt(1).encode(&mut *w).unwrap(); // Nested list length
                write_recursive_bundle(w, depth - 1, true);
            } else {
                VarInt(0).encode(&mut *w).unwrap(); // Empty nested list
            }
        }

        write_recursive_bundle(&mut buf, 20, false); // 20 > 16

        let mut slice = buf.as_slice();
        let result = ItemStack::decode(&mut slice);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("recursion limit exceeded")
        );
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
        assert!(ids.contains(&34));
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
            display: AttributeModifierDisplay::Default,
        };

        let comp = ItemComponent::AttributeModifiers {
            modifiers: vec![modifier],
        };

        let mut stack = ItemStack::new(ItemKind::NetheriteSword, 1);
        stack.insert_component(comp);
        roundtrip(&stack);
    }

    /// Each component encodes to the bytes of vanilla 26.2's network codec
    /// and decodes back from them.
    #[test]
    fn test_components_match_vanilla_wire_format() {
        use chunkedge_binary::{IDSet, IdOr, IntoTextComponent};
        use chunkedge_ident::Ident;
        use chunkedge_nbt::Value;

        let details = |amplifier, duration, hidden_effect| PotionEffectDetails {
            amplifier: VarInt(amplifier),
            duration: VarInt(duration),
            ambient: false,
            show_particles: true,
            show_icon: true,
            hidden_effect,
        };
        let cases: Vec<(ItemComponent, Vec<u8>)> = vec![
            (
                ItemComponent::StoredEnchantments(vec![(RegistryId::new(3), VarInt(2))]),
                vec![1, 3, 2],
            ),
            (
                ItemComponent::LodestoneTracker {
                    target: Some(LodestoneTarget {
                        dimension: "a:b".into(),
                        position: (1, 64, -1),
                    }),
                    tracked: true,
                },
                vec![
                    1, 3, b'a', b':', b'b', 0, 0, 0, 0x7f, 0xff, 0xff, 0xf0, 0x40, 1,
                ],
            ),
            (
                ItemComponent::PotionContents {
                    potion_id: None,
                    custom_color: None,
                    custom_effects: vec![PotionEffect {
                        id: RegistryId::new(1),
                        details: details(0, 200, Some(Box::new(details(1, 100, None)))),
                    }],
                    custom_name: None,
                },
                vec![0, 0, 1, 1, 0, 0xc8, 0x01, 0, 1, 1, 1, 1, 100, 0, 1, 1, 0, 0],
            ),
            (
                ItemComponent::ProvidesBannerPatterns(IDSet::AdHocSet(vec![RegistryId::new(4)])),
                vec![2, 4],
            ),
            (
                ItemComponent::Trim {
                    material: IdOr::Id(RegistryId::new(0)),
                    pattern: IdOr::Inline(TrimPattern {
                        asset_id: "a:b".into(),
                        description: "x".into_text_component(),
                        decal: true,
                    }),
                },
                vec![1, 0, 3, b'a', b':', b'b', 8, 0, 1, b'x', 1],
            ),
            (
                ItemComponent::JukeboxPlayable(IdOr::Inline(JukeboxSong {
                    sound_event: IdOr::Id(RegistryId::new(0)),
                    description: "s".into_text_component(),
                    length_seconds: 1.0,
                    comparator_output: VarInt(15),
                })),
                vec![0, 1, 8, 0, 1, b's', 0x3f, 0x80, 0, 0, 15],
            ),
            (ItemComponent::ChickenVariant(RegistryId::new(2)), vec![2]),
            (
                ItemComponent::DeathProtection(vec![ConsumeEffect::TeleportRandomly {
                    diameter: 16.0,
                }]),
                vec![1, 3, 0x41, 0x80, 0, 0],
            ),
            (
                ItemComponent::Recipes(vec![Ident::new("a:b").unwrap().to_string_ident()]),
                vec![9, 8, 0, 0, 0, 1, 0, 3, b'a', b':', b'b'],
            ),
            (ItemComponent::Lock(Compound::new()), vec![10, 0]),
            (
                ItemComponent::PaintingVariant(IdOr::Inline(PaintingVariantDefinition {
                    width: VarInt(2),
                    height: VarInt(1),
                    asset_id: "a:b".into(),
                    title: Some("t".into_text_component()),
                    author: None,
                })),
                vec![0, 2, 1, 3, b'a', b':', b'b', 1, 8, 0, 1, b't', 0],
            ),
            (
                ItemComponent::CanBreak(vec![BlockPredicate {
                    blocks: None,
                    properties: Some(vec![Property {
                        name: "n".into(),
                        value: PropertyValue::MinMax {
                            min: None,
                            max: Some("5".into()),
                        },
                    }]),
                    nbt: None,
                    exact_components: Vec::new(),
                    partial_components: vec![PartialComponentMatcher {
                        predicate_type: ComponentPredicateType::Predicate(RegistryId::new(1)),
                        predicate: Value::Int(7),
                    }],
                }]),
                vec![
                    1, 0, 1, 1, 1, b'n', 0, 0, 1, 1, b'5', 0, 0, 1, 1, 1, 3, 0, 0, 0, 7,
                ],
            ),
            (ItemComponent::RabbitVariant(RabbitType::Evil), vec![99]),
            (
                ItemComponent::TropicalFishPattern(TropicalFishPattern::Clayfish),
                vec![0x81, 0x0a],
            ),
            (
                ItemComponent::Consumable {
                    consume_seconds: 1.0,
                    animation: ConsumableAnimation::Spear,
                    sound: IdOr::Id(RegistryId::new(0)),
                    has_consume_particles: false,
                    effects: Vec::new(),
                },
                vec![0x3f, 0x80, 0, 0, 11, 1, 0, 0],
            ),
            (
                ItemComponent::Bees(vec![BeeData {
                    entity_type: RegistryId::new(5),
                    entity_data: Compound::new(),
                    ticks_in_hive: VarInt(10),
                    min_ticks_in_hive: VarInt(20),
                }]),
                vec![1, 5, 10, 0, 10, 20],
            ),
        ];

        for (component, expected) in cases {
            let mut bytes = Vec::new();
            component.encode(&mut bytes).unwrap();
            assert_eq!(bytes, expected, "encoding {component:?}");

            let mut r = bytes.as_slice();
            let decoded = impls::decode_item_component(&mut r, component.id() as usize, 0)
                .unwrap_or_else(|e| panic!("decoding {component:?}: {e:#}"));
            assert_eq!(decoded, component);
            assert!(r.is_empty(), "{component:?} left {} bytes", r.len());
        }
    }
}
