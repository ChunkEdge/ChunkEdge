use std::io::Write;

use valence_binary::{cautious_capacity, Decode, Encode, VarInt};
use valence_generated::item::ItemKind;

use crate::components::{BlockPredicate, ExactComponentMatcher, ItemComponent, Patchable};
use crate::vanilla_components::ItemKindExt;
use crate::{HashedItemStack, ItemStack, MAX_RECURSION_DEPTH, NUM_ITEM_COMPONENTS};

impl Encode for ItemStack {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        self.encode_recursive(w, false)
    }
}

impl Encode for ItemComponent {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        // Break recursion loop by erasing the type
        let w: &mut dyn Write = &mut w;

        match self {
            ItemComponent::CustomData(v) => v.encode(w),
            ItemComponent::MaxStackSize(v) => v.encode(w),
            ItemComponent::MaxDamage(v) => v.encode(w),
            ItemComponent::Damage(v) => v.encode(w),
            ItemComponent::Unbreakable => Ok(()),
            ItemComponent::CustomName(v) => v.encode(w),
            ItemComponent::ItemName(v) => v.encode(w),
            ItemComponent::ItemModel(v) => v.encode(w),
            ItemComponent::Lore(v) => v.encode(w),
            ItemComponent::Rarity(v) => v.encode(w),
            ItemComponent::Enchantments(v) => v.encode(w),
            ItemComponent::CanPlaceOn(v) => v.encode(w),
            ItemComponent::CanBreak(v) => v.encode(w),
            ItemComponent::AttributeModifiers { modifiers } => modifiers.encode(w),
            ItemComponent::CustomModelData {
                floats,
                flags,
                strings,
                colors,
            } => {
                floats.encode(&mut *w)?;
                flags.encode(&mut *w)?;
                strings.encode(&mut *w)?;
                colors.encode(w)
            }
            ItemComponent::TooltipDisplay {
                hide_tooltip,
                hidden_components,
            } => {
                hide_tooltip.encode(&mut *w)?;
                hidden_components.encode(w)
            }
            ItemComponent::RepairCost(v) => v.encode(w),
            ItemComponent::CreativeSlotLock => Ok(()),
            ItemComponent::EnchantmentGlintOverride(v) => v.encode(w),
            ItemComponent::IntangibleProjectile(v) => v.encode(w),
            ItemComponent::Food {
                nutrition,
                saturation_modifier,
                can_always_eat,
            } => {
                nutrition.encode(&mut *w)?;
                saturation_modifier.encode(&mut *w)?;
                can_always_eat.encode(w)
            }
            ItemComponent::Consumable {
                consume_seconds,
                animation,
                sound,
                has_consume_particles,
                effects,
            } => {
                consume_seconds.encode(&mut *w)?;
                animation.encode(&mut *w)?;
                sound.encode(&mut *w)?;
                has_consume_particles.encode(&mut *w)?;
                effects.encode(w)
            }
            ItemComponent::UseRemainder(v) => v.encode(w),
            ItemComponent::UseCooldown {
                seconds,
                cooldown_group,
            } => {
                seconds.encode(&mut *w)?;
                cooldown_group.encode(w)
            }
            ItemComponent::DamageResistant(v) => v.encode(w),
            ItemComponent::Tool {
                rules,
                default_mining_speed,
                damage_per_block,
                can_destroy_blocks_in_creative,
            } => {
                rules.encode(&mut *w)?;
                default_mining_speed.encode(&mut *w)?;
                damage_per_block.encode(&mut *w)?;
                can_destroy_blocks_in_creative.encode(w)
            }
            ItemComponent::Weapon {
                damage_per_attack,
                disable_blocking_for_seconds,
            } => {
                damage_per_attack.encode(&mut *w)?;
                disable_blocking_for_seconds.encode(w)
            }
            ItemComponent::Enchantable(v) => v.encode(w),
            ItemComponent::Equippable {
                slot,
                equip_sound,
                model,
                camera_overlay,
                allowed_entities,
                dispensable,
                swappable,
                damage_on_hurt,
            } => {
                slot.encode(&mut *w)?;
                equip_sound.encode(&mut *w)?;
                model.encode(&mut *w)?;
                camera_overlay.encode(&mut *w)?;
                allowed_entities.encode(&mut *w)?;
                dispensable.encode(&mut *w)?;
                swappable.encode(&mut *w)?;
                damage_on_hurt.encode(w)
            }
            ItemComponent::Repairable(v) => v.encode(w),
            ItemComponent::Glider => Ok(()),
            ItemComponent::TooltipStyle(v) => v.encode(w),
            ItemComponent::DeathProtection(v) => v.encode(w),
            ItemComponent::BlocksAttacks {
                block_delay_seconds,
                disable_cooldown_scale,
                damage_reductions,
                item_damage_threshold,
                item_damage_base,
                item_damage_factor,
                bypassed_by,
                block_sound,
                disable_sound,
            } => {
                block_delay_seconds.encode(&mut *w)?;
                disable_cooldown_scale.encode(&mut *w)?;
                damage_reductions.encode(&mut *w)?;
                item_damage_threshold.encode(&mut *w)?;
                item_damage_base.encode(&mut *w)?;
                item_damage_factor.encode(&mut *w)?;
                bypassed_by.encode(&mut *w)?;
                block_sound.encode(&mut *w)?;
                disable_sound.encode(w)
            }
            ItemComponent::StoredEnchantments {
                enchantments,
                show_in_tooltip,
            } => {
                enchantments.encode(&mut *w)?;
                show_in_tooltip.encode(w)
            }
            ItemComponent::DyedColor {
                color,
                show_in_tooltip,
            } => {
                color.encode(&mut *w)?;
                show_in_tooltip.encode(w)
            }
            ItemComponent::MapColor(v) => v.encode(w),
            ItemComponent::MapId(v) => v.encode(w),
            ItemComponent::MapDecorations(v) => v.encode(w),
            ItemComponent::MapPostProcessing(v) => v.encode(w),
            ItemComponent::ChargedProjectiles(v) => v.encode(w),
            ItemComponent::BundleContents(v) => v.encode(w),
            ItemComponent::PotionContents {
                potion_id,
                custom_color,
                custom_effects,
                custom_name,
            } => {
                potion_id.encode(&mut *w)?;
                custom_color.encode(&mut *w)?;
                custom_effects.encode(&mut *w)?;
                custom_name.encode(w)
            }
            ItemComponent::PotionDurationScale(v) => v.encode(w),
            ItemComponent::SuspiciousStewEffects(v) => v.encode(w),
            ItemComponent::WritableBookContent { pages } => pages.encode(w),
            ItemComponent::WrittenBookContent {
                raw_title,
                filtered_title,
                author,
                generation,
                pages,
                resolved,
            } => {
                raw_title.encode(&mut *w)?;
                filtered_title.encode(&mut *w)?;
                author.encode(&mut *w)?;
                generation.encode(&mut *w)?;
                pages.encode(&mut *w)?;
                resolved.encode(w)
            }
            ItemComponent::Trim {
                material,
                pattern,
                show_in_tooltip,
            } => {
                material.encode(&mut *w)?;
                pattern.encode(&mut *w)?;
                show_in_tooltip.encode(w)
            }
            ItemComponent::DebugStickState(v) => v.encode(w),
            ItemComponent::EntityData { id, data } => {
                id.encode(&mut *w)?;
                data.encode(w)
            }
            ItemComponent::BucketEntityData(v) => v.encode(w),
            ItemComponent::BlockEntityData { id, data } => {
                id.encode(&mut *w)?;
                data.encode(w)
            }
            ItemComponent::Instrument(v) => v.encode(w),
            ItemComponent::ProvidesTrimMaterial(v) => v.encode(w),
            ItemComponent::OminousBottleAmplifier(v) => v.encode(w),
            ItemComponent::JukeboxPlayable {
                song,
                show_in_tooltip,
            } => {
                song.encode(&mut *w)?;
                show_in_tooltip.encode(w)
            }
            ItemComponent::ProvidesBannerPatterns(v) => v.encode(w),
            ItemComponent::Recipes(v) => v.encode(w),
            ItemComponent::LodestoneTracker { target, tracked } => {
                target.encode(&mut *w)?;
                tracked.encode(w)
            }
            ItemComponent::FireworkExplosion(v) => v.encode(w),
            ItemComponent::Fireworks {
                flight_duration,
                explosions,
            } => {
                flight_duration.encode(&mut *w)?;
                explosions.encode(w)
            }
            ItemComponent::Profile(v) => v.encode(w),
            ItemComponent::NoteBlockSound(v) => v.encode(w),
            ItemComponent::BannerPatterns(v) => v.encode(w),
            ItemComponent::BaseColor(v) => v.encode(w),
            ItemComponent::PotDecorations(v) => v.encode(w),
            ItemComponent::Container(v) => v.encode(w),
            ItemComponent::BlockState(v) => v.encode(w),
            ItemComponent::Bees(v) => v.encode(w),
            ItemComponent::Lock(v) => v.encode(w),
            ItemComponent::ContainerLoot(v) => v.encode(w),
            ItemComponent::BreakSound(v) => v.encode(w),
            ItemComponent::VillagerVariant(v) => v.encode(w),
            ItemComponent::WolfVariant(v) => v.encode(w),
            ItemComponent::WolfSoundVariant(v) => v.encode(w),
            ItemComponent::WolfCollar(v) => v.encode(w),
            ItemComponent::FoxVariant(v) => v.encode(w),
            ItemComponent::SalmonSize(v) => v.encode(w),
            ItemComponent::ParrotVariant(v) => v.encode(w),
            ItemComponent::TropicalFishPattern(v) => v.encode(w),
            ItemComponent::TropicalFishBaseColor(v) => v.encode(w),
            ItemComponent::TropicalFishPatternColor(v) => v.encode(w),
            ItemComponent::MooshroomVariant(v) => v.encode(w),
            ItemComponent::RabbitVariant(v) => v.encode(w),
            ItemComponent::PigVariant(v) => v.encode(w),
            ItemComponent::CowVariant(v) => v.encode(w),
            ItemComponent::ChickenVariant(v) => v.encode(w),
            ItemComponent::FrogVariant(v) => v.encode(w),
            ItemComponent::HorseVariant(v) => v.encode(w),
            ItemComponent::PaintingVariant(v) => v.encode(w),
            ItemComponent::LlamaVariant(v) => v.encode(w),
            ItemComponent::AxolotlVariant(v) => v.encode(w),
            ItemComponent::CatVariant(v) => v.encode(w),
            ItemComponent::CatCollar(v) => v.encode(w),
            ItemComponent::SheepColor(v) => v.encode(w),
            ItemComponent::ShulkerColor(v) => v.encode(w),
        }
    }
}
impl<'a> Decode<'a> for ItemStack {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        decode_item_stack_recursive(r, 0, false)
    }
}

pub fn decode_item_stack_recursive(
    r: &mut &[u8],
    depth: usize,
    prefixed: bool,
) -> anyhow::Result<ItemStack> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(anyhow::anyhow!("ItemStack recursion limit exceeded"));
    }

    let count = VarInt::decode(r)?.0;
    if count <= 0 {
        return Ok(ItemStack::EMPTY);
    }
    let item = ItemKind::decode(r)?;

    let mut components = item.default_components();

    // Decode counts
    let added_count = VarInt::decode(r)?.0;
    let removed_count = VarInt::decode(r)?.0;

    // Decode Added Components
    for _ in 0..added_count {
        let id = VarInt::decode(r)?.0 as usize;
        if id >= NUM_ITEM_COMPONENTS {
            return Err(anyhow::anyhow!("Invalid item component ID: {id}"));
        }

        let _prefix = if prefixed {
            Some(VarInt::decode(r)?)
        } else {
            None
        }; // TODO: Use prefix?

        let component = decode_item_component(r, id, depth)?;
        let hash = component.hash();
        components[id] = Patchable::Added((Box::new(component), hash));
    }

    // Decode Removed Components
    for _ in 0..removed_count {
        let id = VarInt::decode(r)?.0 as usize;
        if id >= NUM_ITEM_COMPONENTS {
            return Err(anyhow::anyhow!("Invalid item component ID: {id}"));
        }
        components[id] = Patchable::Removed;
    }

    Ok(ItemStack {
        item,
        count: count as i8,
        components,
    })
}

fn decode_block_predicate(r: &mut &[u8], depth: usize) -> anyhow::Result<BlockPredicate> {
    Ok(BlockPredicate {
        blocks: Decode::decode(r)?,
        properties: Decode::decode(r)?,
        nbt: Decode::decode(r)?,
        exact_components: {
            // Vec = |len|item*len|
            let length = VarInt::decode(r)?.0 as usize;
            let mut vec = Vec::with_capacity(cautious_capacity::<ExactComponentMatcher>(length));
            for _ in 0..length {
                let component_type = VarInt::decode(r)?;
                let component_data =
                    decode_item_component(r, component_type.0 as usize, depth + 1)?;
                vec.push(ExactComponentMatcher {
                    component_type,
                    component_data,
                });
            }
            vec
        },
        partial_components: Decode::decode(r)?,
    })
}

fn decode_item_component(r: &mut &[u8], id: usize, depth: usize) -> anyhow::Result<ItemComponent> {
    Ok(match id {
        0 => ItemComponent::CustomData(Decode::decode(r)?),
        1 => ItemComponent::MaxStackSize(Decode::decode(r)?),
        2 => ItemComponent::MaxDamage(Decode::decode(r)?),
        3 => ItemComponent::Damage(Decode::decode(r)?),
        4 => ItemComponent::Unbreakable,
        5 => ItemComponent::CustomName(Decode::decode(r)?),
        6 => ItemComponent::ItemName(Decode::decode(r)?),
        7 => ItemComponent::ItemModel(Decode::decode(r)?),
        8 => ItemComponent::Lore(Decode::decode(r)?),
        9 => ItemComponent::Rarity(Decode::decode(r)?),
        10 => ItemComponent::Enchantments(Decode::decode(r)?),
        11 => ItemComponent::CanPlaceOn({
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<BlockPredicate>(count as usize));
            for _ in 0..count {
                items.push(decode_block_predicate(r, depth)?);
            }
            items
        }),
        12 => ItemComponent::CanBreak({
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<BlockPredicate>(count as usize));
            for _ in 0..count {
                items.push(decode_block_predicate(r, depth)?);
            }
            items
        }),
        13 => ItemComponent::AttributeModifiers {
            modifiers: Decode::decode(r)?,
        },
        14 => ItemComponent::CustomModelData {
            floats: Decode::decode(r)?,
            flags: Decode::decode(r)?,
            strings: Decode::decode(r)?,
            colors: Decode::decode(r)?,
        },
        15 => ItemComponent::TooltipDisplay {
            hide_tooltip: Decode::decode(r)?,
            hidden_components: Decode::decode(r)?,
        },
        16 => ItemComponent::RepairCost(Decode::decode(r)?),
        17 => ItemComponent::CreativeSlotLock,
        18 => ItemComponent::EnchantmentGlintOverride(Decode::decode(r)?),
        19 => ItemComponent::IntangibleProjectile(Decode::decode(r)?),
        20 => ItemComponent::Food {
            nutrition: Decode::decode(r)?,
            saturation_modifier: Decode::decode(r)?,
            can_always_eat: Decode::decode(r)?,
        },
        21 => ItemComponent::Consumable {
            consume_seconds: Decode::decode(r)?,
            animation: Decode::decode(r)?,
            sound: Decode::decode(r)?,
            has_consume_particles: Decode::decode(r)?,
            effects: Decode::decode(r)?,
        },
        22 => {
            ItemComponent::UseRemainder(Box::new(decode_item_stack_recursive(r, depth + 1, false)?))
        }
        23 => ItemComponent::UseCooldown {
            seconds: Decode::decode(r)?,
            cooldown_group: Decode::decode(r)?,
        },
        24 => ItemComponent::DamageResistant(Decode::decode(r)?),
        25 => ItemComponent::Tool {
            rules: Decode::decode(r)?,
            default_mining_speed: Decode::decode(r)?,
            damage_per_block: Decode::decode(r)?,
            can_destroy_blocks_in_creative: Decode::decode(r)?,
        },
        26 => ItemComponent::Weapon {
            damage_per_attack: Decode::decode(r)?,
            disable_blocking_for_seconds: Decode::decode(r)?,
        },
        27 => ItemComponent::Enchantable(Decode::decode(r)?),
        28 => ItemComponent::Equippable {
            slot: Decode::decode(r)?,
            equip_sound: Decode::decode(r)?,
            model: Decode::decode(r)?,
            camera_overlay: Decode::decode(r)?,
            allowed_entities: Decode::decode(r)?,
            dispensable: Decode::decode(r)?,
            swappable: Decode::decode(r)?,
            damage_on_hurt: Decode::decode(r)?,
        },
        29 => ItemComponent::Repairable(Decode::decode(r)?),
        30 => ItemComponent::Glider,
        31 => ItemComponent::TooltipStyle(Decode::decode(r)?),
        32 => ItemComponent::DeathProtection(Decode::decode(r)?),
        33 => ItemComponent::BlocksAttacks {
            block_delay_seconds: Decode::decode(r)?,
            disable_cooldown_scale: Decode::decode(r)?,
            damage_reductions: Decode::decode(r)?,
            item_damage_threshold: Decode::decode(r)?,
            item_damage_base: Decode::decode(r)?,
            item_damage_factor: Decode::decode(r)?,
            bypassed_by: Decode::decode(r)?,
            block_sound: Decode::decode(r)?,
            disable_sound: Decode::decode(r)?,
        },
        34 => ItemComponent::StoredEnchantments {
            enchantments: Decode::decode(r)?,
            show_in_tooltip: Decode::decode(r)?,
        },
        35 => ItemComponent::DyedColor {
            color: Decode::decode(r)?,
            show_in_tooltip: Decode::decode(r)?,
        },
        36 => ItemComponent::MapColor(Decode::decode(r)?),
        37 => ItemComponent::MapId(Decode::decode(r)?),
        38 => ItemComponent::MapDecorations(Decode::decode(r)?),
        39 => ItemComponent::MapPostProcessing(Decode::decode(r)?),
        40 => {
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<ItemStack>(count as usize));
            for _ in 0..count {
                items.push(decode_item_stack_recursive(r, depth + 1, false)?);
            }
            ItemComponent::ChargedProjectiles(items)
        }
        41 => {
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<ItemStack>(count as usize));
            for _ in 0..count {
                items.push(decode_item_stack_recursive(r, depth + 1, false)?);
            }
            ItemComponent::BundleContents(items)
        }
        42 => ItemComponent::PotionContents {
            potion_id: Decode::decode(r)?,
            custom_color: Decode::decode(r)?,
            custom_effects: Decode::decode(r)?,
            custom_name: Decode::decode(r)?,
        },
        43 => ItemComponent::PotionDurationScale(Decode::decode(r)?),
        44 => ItemComponent::SuspiciousStewEffects(Decode::decode(r)?),
        45 => ItemComponent::WritableBookContent {
            pages: Decode::decode(r)?,
        },
        46 => ItemComponent::WrittenBookContent {
            raw_title: Decode::decode(r)?,
            filtered_title: Decode::decode(r)?,
            author: Decode::decode(r)?,
            generation: Decode::decode(r)?,
            pages: Decode::decode(r)?,
            resolved: Decode::decode(r)?,
        },
        47 => ItemComponent::Trim {
            material: Decode::decode(r)?,
            pattern: Decode::decode(r)?,
            show_in_tooltip: Decode::decode(r)?,
        },
        48 => ItemComponent::DebugStickState(Decode::decode(r)?),
        49 => ItemComponent::EntityData {
            id: Decode::decode(r)?,
            data: Decode::decode(r)?,
        },
        50 => ItemComponent::BucketEntityData(Decode::decode(r)?),
        51 => ItemComponent::BlockEntityData {
            id: Decode::decode(r)?,
            data: Decode::decode(r)?,
        },
        52 => ItemComponent::Instrument(Decode::decode(r)?),
        53 => ItemComponent::ProvidesTrimMaterial(Decode::decode(r)?),
        54 => ItemComponent::OminousBottleAmplifier(Decode::decode(r)?),
        55 => ItemComponent::JukeboxPlayable {
            song: Decode::decode(r)?,
            show_in_tooltip: Decode::decode(r)?,
        },
        56 => ItemComponent::ProvidesBannerPatterns(Decode::decode(r)?),
        57 => ItemComponent::Recipes(Decode::decode(r)?),
        58 => ItemComponent::LodestoneTracker {
            target: Decode::decode(r)?,
            tracked: Decode::decode(r)?,
        },
        59 => ItemComponent::FireworkExplosion(Decode::decode(r)?),
        60 => ItemComponent::Fireworks {
            flight_duration: Decode::decode(r)?,
            explosions: Decode::decode(r)?,
        },
        61 => ItemComponent::Profile(Decode::decode(r)?),
        62 => ItemComponent::NoteBlockSound(Decode::decode(r)?),
        63 => ItemComponent::BannerPatterns(Decode::decode(r)?),
        64 => ItemComponent::BaseColor(Decode::decode(r)?),
        65 => ItemComponent::PotDecorations(Decode::decode(r)?),
        66 => {
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<ItemStack>(count as usize));
            for _ in 0..count {
                items.push(decode_item_stack_recursive(r, depth + 1, false)?);
            }
            ItemComponent::Container(items)
        }
        67 => ItemComponent::BlockState(Decode::decode(r)?),
        68 => ItemComponent::Bees(Decode::decode(r)?),
        69 => ItemComponent::Lock(Decode::decode(r)?),
        70 => ItemComponent::ContainerLoot(Decode::decode(r)?),
        71 => ItemComponent::BreakSound(Decode::decode(r)?),
        72 => ItemComponent::VillagerVariant(Decode::decode(r)?),
        73 => ItemComponent::WolfVariant(Decode::decode(r)?),
        74 => ItemComponent::WolfSoundVariant(Decode::decode(r)?),
        75 => ItemComponent::WolfCollar(Decode::decode(r)?),
        76 => ItemComponent::FoxVariant(Decode::decode(r)?),
        77 => ItemComponent::SalmonSize(Decode::decode(r)?),
        78 => ItemComponent::ParrotVariant(Decode::decode(r)?),
        79 => ItemComponent::TropicalFishPattern(Decode::decode(r)?),
        80 => ItemComponent::TropicalFishBaseColor(Decode::decode(r)?),
        81 => ItemComponent::TropicalFishPatternColor(Decode::decode(r)?),
        82 => ItemComponent::MooshroomVariant(Decode::decode(r)?),
        83 => ItemComponent::RabbitVariant(Decode::decode(r)?),
        84 => ItemComponent::PigVariant(Decode::decode(r)?),
        85 => ItemComponent::CowVariant(Decode::decode(r)?),
        86 => ItemComponent::ChickenVariant(Decode::decode(r)?),
        87 => ItemComponent::FrogVariant(Decode::decode(r)?),
        88 => ItemComponent::HorseVariant(Decode::decode(r)?),
        89 => ItemComponent::PaintingVariant(Decode::decode(r)?),
        90 => ItemComponent::LlamaVariant(Decode::decode(r)?),
        91 => ItemComponent::AxolotlVariant(Decode::decode(r)?),
        92 => ItemComponent::CatVariant(Decode::decode(r)?),
        93 => ItemComponent::CatCollar(Decode::decode(r)?),
        94 => ItemComponent::SheepColor(Decode::decode(r)?),
        95 => ItemComponent::ShulkerColor(Decode::decode(r)?),
        _ => return Err(anyhow::anyhow!("Unknown ItemComponent ID: {id}")),
    })
}

// Encode for HashedItemStack as described in "Hashed Format"
impl Encode for HashedItemStack {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        if self.is_empty() {
            false.encode(&mut w)
        } else {
            true.encode(&mut w)?;
            self.item.encode(&mut w)?;
            VarInt(i32::from(self.count)).encode(&mut w)?;

            let mut added = Vec::new();
            let mut removed = Vec::new();

            for (i, c) in self.components.iter().enumerate() {
                match c {
                    Patchable::Added(((), hash)) => added.push((i, hash)),
                    Patchable::Removed => removed.push(i),
                    _ => {}
                }
            }

            VarInt(added.len() as i32).encode(&mut w)?;
            for (id, hash) in added {
                VarInt(id as i32).encode(&mut w)?;
                hash.encode(&mut w)?;
            }

            VarInt(removed.len() as i32).encode(&mut w)?;
            for id in removed {
                VarInt(id as i32).encode(&mut w)?;
            }

            Ok(())
        }
    }
}
impl Decode<'_> for HashedItemStack {
    fn decode(r: &mut &'_ [u8]) -> anyhow::Result<Self> {
        let has_item = bool::decode(r)?;
        if !has_item {
            Ok(Self::EMPTY)
        } else {
            let item = ItemKind::decode(r)?;
            let item_count = VarInt::decode(r)?;

            let mut components = [Patchable::None; NUM_ITEM_COMPONENTS];

            let components_added: Vec<(VarInt, i32)> = Vec::decode(r)?;
            let components_removed: Vec<VarInt> = Vec::decode(r)?;

            for (id, hash) in components_added {
                let id = id.0 as usize;
                if id >= NUM_ITEM_COMPONENTS {
                    return Err(anyhow::anyhow!("Invalid item component ID: {id}"));
                }
                components[id] = Patchable::Added(((), hash));
            }

            for id in components_removed {
                let id = id.0 as usize;
                if id >= NUM_ITEM_COMPONENTS {
                    return Err(anyhow::anyhow!("Invalid item component ID: {id}"));
                }
                components[id] = Patchable::Removed;
            }

            Ok(Self {
                item,
                count: item_count.0 as i8,
                components,
            })
        }
    }
}
