use std::io::Write;

use chunkedge_binary::{Bounded, Decode, Encode, VarInt, cautious_capacity};
use chunkedge_generated::item::ItemKind;
use chunkedge_ident::Ident;
use chunkedge_nbt::{List, Value};

use crate::components::{
    BlockPredicate, ExactComponentMatcher, ItemComponent, ItemComponentKind, Patchable,
};
use crate::vanilla_components::ItemKindExt;
use crate::{
    HashedItemStack, ItemStack, ItemStackTemplate, MAX_RECURSION_DEPTH, NUM_ITEM_COMPONENTS,
};

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
            ItemComponent::UseEffects(v) => v.encode(w),
            ItemComponent::MinimumAttackCharge(v) => v.encode(w),
            ItemComponent::DamageType(v) => v.encode(w),
            ItemComponent::AttackRange(v) => v.encode(w),
            ItemComponent::PiercingWeapon(v) => v.encode(w),
            ItemComponent::KineticWeapon(v) => v.encode(w),
            ItemComponent::BlockTransformer(v) => v.encode(w),
            ItemComponent::VillagerFood(v) => v.encode(w),
            ItemComponent::Compostable(v) => v.encode(w),
            ItemComponent::CookingFuel(v) => v.encode(w),
            ItemComponent::BrewingFuel(v) => v.encode(w),
            ItemComponent::MobVisibility(v) => v.encode(w),
            ItemComponent::ProvidesPotteryPattern(v) => v.encode(w),
            ItemComponent::SignTextFront(v) => v.encode(w),
            ItemComponent::SignTextBack(v) => v.encode(w),
            ItemComponent::CushionColor(v) => v.encode(w),
            ItemComponent::Waxed => Ok(()),
            ItemComponent::AttackAnimation(v) | ItemComponent::InteractAnimation(v) => v.encode(w),
            ItemComponent::AdditionalTradeCost(v) => v.encode(w),
            ItemComponent::Dye(v) => v.encode(w),
            ItemComponent::PigSoundVariant(v) => v.encode(w),
            ItemComponent::CowSoundVariant(v) => v.encode(w),
            ItemComponent::ChickenSoundVariant(v) => v.encode(w),
            ItemComponent::ZombieNautilusVariant(v) => v.encode(w),
            ItemComponent::CatSoundVariant(v) => v.encode(w),
            ItemComponent::SulfurCubeContent(v) => v.encode(w),
            ItemComponent::MaxStackSize(v) => v.encode(w),
            ItemComponent::MaxDamage(v) => v.encode(w),
            ItemComponent::Damage(v) => v.encode(w),
            ItemComponent::Unbreakable => Ok(()),
            ItemComponent::CustomName(v) => v.encode(w),
            ItemComponent::ItemName(v) => v.encode(w),
            ItemComponent::ItemModel(v) => v.encode(w),
            ItemComponent::Lore(v) => Bounded::<_, 256>(v.as_slice()).encode(w),
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
                equip_on_interact,
                can_be_sheared,
                shearing_sound,
            } => {
                slot.encode(&mut *w)?;
                equip_sound.encode(&mut *w)?;
                model.encode(&mut *w)?;
                camera_overlay.encode(&mut *w)?;
                allowed_entities.encode(&mut *w)?;
                dispensable.encode(&mut *w)?;
                swappable.encode(&mut *w)?;
                damage_on_hurt.encode(&mut *w)?;
                equip_on_interact.encode(&mut *w)?;
                can_be_sheared.encode(&mut *w)?;
                shearing_sound.encode(w)
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
            ItemComponent::StoredEnchantments(v) => v.encode(w),
            ItemComponent::DyedColor { color } => color.encode(w),
            ItemComponent::MapId(v) => v.encode(w),
            ItemComponent::MapDecorations(v) => v.encode(w),
            ItemComponent::MapPostProcessing(v) => v.encode(w),
            ItemComponent::ChargedProjectiles(v) => Bounded::<_, 1024>(v.as_slice()).encode(w),
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
            ItemComponent::WritableBookContent { pages } => {
                Bounded::<_, 100>(pages.as_slice()).encode(w)
            }
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
            ItemComponent::Trim { material, pattern } => {
                material.encode(&mut *w)?;
                pattern.encode(w)
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
            ItemComponent::JukeboxPlayable(v) => v.encode(w),
            ItemComponent::ProvidesBannerPatterns(v) => v.encode(w),
            ItemComponent::Recipes(v) => recipes_to_nbt(v).encode(w),
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
                Bounded::<_, 256>(explosions.as_slice()).encode(w)
            }
            ItemComponent::Profile(v) => v.encode(w),
            ItemComponent::NoteBlockSound(v) => v.encode(w),
            ItemComponent::BannerPatterns(v) => v.encode(w),
            ItemComponent::BaseColor(v) => v.encode(w),
            ItemComponent::PotDecorations(v) => v.encode(w),
            ItemComponent::Container(v) => Bounded::<_, 256>(v.as_slice()).encode(w),
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

    decode_stack_components(r, item, count as i8, depth, prefixed)
}

pub(crate) fn decode_item_stack_template(r: &mut &[u8], depth: usize) -> anyhow::Result<ItemStack> {
    anyhow::ensure!(
        depth <= MAX_RECURSION_DEPTH,
        "ItemStack recursion limit exceeded"
    );
    let item = ItemKind::decode(r)?;
    let count = VarInt::decode(r)?.0;
    anyhow::ensure!(
        item != ItemKind::Air && (1..=i32::from(i8::MAX)).contains(&count),
        "Item stack template must have a non-air item and a count between 1 and 127"
    );
    decode_stack_components(r, item, count as i8, depth, false)
}

fn decode_stack_components(
    r: &mut &[u8],
    item: ItemKind,
    count: i8,
    depth: usize,
    prefixed: bool,
) -> anyhow::Result<ItemStack> {
    let mut components = item.default_components();

    // Decode counts
    let added_count = VarInt::decode(r)?.0;
    let removed_count = VarInt::decode(r)?.0;

    anyhow::ensure!(
        added_count >= 0 && removed_count >= 0,
        "negative component count"
    );

    // Decode Added Components
    for _ in 0..added_count {
        let id = VarInt::decode(r)?.0 as usize;
        if id >= NUM_ITEM_COMPONENTS {
            return Err(anyhow::anyhow!("Invalid item component ID: {id}"));
        }

        let component = if prefixed {
            let len = VarInt::decode(r)?.0;
            anyhow::ensure!(
                len >= 0 && len as usize <= r.len(),
                "invalid component length"
            );
            let (payload, rest) = r.split_at(len as usize);
            *r = rest;
            let mut payload = payload;
            let component = decode_item_component(&mut payload, id, depth)?;
            anyhow::ensure!(payload.is_empty(), "trailing component bytes");
            component
        } else {
            decode_item_component(r, id, depth)?
        };

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
        count,
        components,
    })
}

/// Encodes knowledge book recipe keys the way vanilla's persistent codec
/// does: an NBT list of strings.
fn recipes_to_nbt(recipes: &[Ident<String>]) -> Value {
    Value::List(List::String(
        recipes.iter().map(|id| id.as_str().to_owned()).collect(),
    ))
}

fn recipes_from_nbt(value: Value) -> anyhow::Result<Vec<Ident<String>>> {
    match value {
        Value::List(List::End) => Ok(Vec::new()),
        Value::List(List::String(keys)) => keys
            .into_iter()
            .map(|key| Ident::<String>::try_from(key).map_err(Into::into))
            .collect(),
        other => anyhow::bail!("expected a list of recipe keys, got {:?}", other.tag()),
    }
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
        partial_components: Bounded::<_, 64>::decode(r)?.0,
    })
}

pub(crate) fn decode_item_component(
    r: &mut &[u8],
    id: usize,
    depth: usize,
) -> anyhow::Result<ItemComponent> {
    anyhow::ensure!(
        depth <= MAX_RECURSION_DEPTH,
        "ItemStack recursion limit exceeded"
    );
    let kind = ItemComponentKind::from_id(id)
        .ok_or_else(|| anyhow::anyhow!("Unknown ItemComponent ID: {id}"))?;
    Ok(match kind {
        ItemComponentKind::CustomData => ItemComponent::CustomData(Decode::decode(r)?),
        ItemComponentKind::UseEffects => ItemComponent::UseEffects(Decode::decode(r)?),
        ItemComponentKind::MinimumAttackCharge => {
            ItemComponent::MinimumAttackCharge(Decode::decode(r)?)
        }
        ItemComponentKind::DamageType => ItemComponent::DamageType(Decode::decode(r)?),
        ItemComponentKind::AttackRange => ItemComponent::AttackRange(Decode::decode(r)?),
        ItemComponentKind::PiercingWeapon => ItemComponent::PiercingWeapon(Decode::decode(r)?),
        ItemComponentKind::KineticWeapon => ItemComponent::KineticWeapon(Decode::decode(r)?),
        ItemComponentKind::BlockTransformer => ItemComponent::BlockTransformer(Decode::decode(r)?),
        ItemComponentKind::VillagerFood => ItemComponent::VillagerFood(Decode::decode(r)?),
        ItemComponentKind::Compostable => ItemComponent::Compostable(Decode::decode(r)?),
        ItemComponentKind::CookingFuel => ItemComponent::CookingFuel(Decode::decode(r)?),
        ItemComponentKind::BrewingFuel => ItemComponent::BrewingFuel(Decode::decode(r)?),
        ItemComponentKind::MobVisibility => ItemComponent::MobVisibility(Decode::decode(r)?),
        ItemComponentKind::ProvidesPotteryPattern => {
            ItemComponent::ProvidesPotteryPattern(Decode::decode(r)?)
        }
        ItemComponentKind::SignTextFront => ItemComponent::SignTextFront(Decode::decode(r)?),
        ItemComponentKind::SignTextBack => ItemComponent::SignTextBack(Decode::decode(r)?),
        ItemComponentKind::CushionColor => ItemComponent::CushionColor(Decode::decode(r)?),
        ItemComponentKind::Waxed => ItemComponent::Waxed,
        ItemComponentKind::AttackAnimation => ItemComponent::AttackAnimation(Decode::decode(r)?),
        ItemComponentKind::InteractAnimation => {
            ItemComponent::InteractAnimation(Decode::decode(r)?)
        }
        ItemComponentKind::AdditionalTradeCost => {
            ItemComponent::AdditionalTradeCost(Decode::decode(r)?)
        }
        ItemComponentKind::Dye => ItemComponent::Dye(Decode::decode(r)?),
        ItemComponentKind::PigSoundVariant => ItemComponent::PigSoundVariant(Decode::decode(r)?),
        ItemComponentKind::CowSoundVariant => ItemComponent::CowSoundVariant(Decode::decode(r)?),
        ItemComponentKind::ChickenSoundVariant => {
            ItemComponent::ChickenSoundVariant(Decode::decode(r)?)
        }
        ItemComponentKind::ZombieNautilusVariant => {
            ItemComponent::ZombieNautilusVariant(Decode::decode(r)?)
        }
        ItemComponentKind::CatSoundVariant => ItemComponent::CatSoundVariant(Decode::decode(r)?),
        ItemComponentKind::SulfurCubeContent => ItemComponent::SulfurCubeContent(Box::new(
            ItemStackTemplate(decode_item_stack_template(r, depth + 1)?),
        )),
        ItemComponentKind::MaxStackSize => ItemComponent::MaxStackSize(Decode::decode(r)?),
        ItemComponentKind::MaxDamage => ItemComponent::MaxDamage(Decode::decode(r)?),
        ItemComponentKind::Damage => ItemComponent::Damage(Decode::decode(r)?),
        ItemComponentKind::Unbreakable => ItemComponent::Unbreakable,
        ItemComponentKind::CustomName => ItemComponent::CustomName(Decode::decode(r)?),
        ItemComponentKind::ItemName => ItemComponent::ItemName(Decode::decode(r)?),
        ItemComponentKind::ItemModel => ItemComponent::ItemModel(Decode::decode(r)?),
        ItemComponentKind::Lore => ItemComponent::Lore(Bounded::<_, 256>::decode(r)?.0),
        ItemComponentKind::Rarity => ItemComponent::Rarity(Decode::decode(r)?),
        ItemComponentKind::Enchantments => ItemComponent::Enchantments(Decode::decode(r)?),
        ItemComponentKind::CanPlaceOn => ItemComponent::CanPlaceOn({
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<BlockPredicate>(count as usize));
            for _ in 0..count {
                items.push(decode_block_predicate(r, depth)?);
            }
            items
        }),
        ItemComponentKind::CanBreak => ItemComponent::CanBreak({
            let count = VarInt::decode(r)?.0;
            let mut items = Vec::with_capacity(cautious_capacity::<BlockPredicate>(count as usize));
            for _ in 0..count {
                items.push(decode_block_predicate(r, depth)?);
            }
            items
        }),
        ItemComponentKind::AttributeModifiers => ItemComponent::AttributeModifiers {
            modifiers: Decode::decode(r)?,
        },
        ItemComponentKind::CustomModelData => ItemComponent::CustomModelData {
            floats: Decode::decode(r)?,
            flags: Decode::decode(r)?,
            strings: Decode::decode(r)?,
            colors: Decode::decode(r)?,
        },
        ItemComponentKind::TooltipDisplay => ItemComponent::TooltipDisplay {
            hide_tooltip: Decode::decode(r)?,
            hidden_components: Decode::decode(r)?,
        },
        ItemComponentKind::RepairCost => ItemComponent::RepairCost(Decode::decode(r)?),
        ItemComponentKind::CreativeSlotLock => ItemComponent::CreativeSlotLock,
        ItemComponentKind::EnchantmentGlintOverride => {
            ItemComponent::EnchantmentGlintOverride(Decode::decode(r)?)
        }
        ItemComponentKind::IntangibleProjectile => {
            ItemComponent::IntangibleProjectile(Decode::decode(r)?)
        }
        ItemComponentKind::Food => ItemComponent::Food {
            nutrition: Decode::decode(r)?,
            saturation_modifier: Decode::decode(r)?,
            can_always_eat: Decode::decode(r)?,
        },
        ItemComponentKind::Consumable => ItemComponent::Consumable {
            consume_seconds: Decode::decode(r)?,
            animation: Decode::decode(r)?,
            sound: Decode::decode(r)?,
            has_consume_particles: Decode::decode(r)?,
            effects: Decode::decode(r)?,
        },
        ItemComponentKind::UseRemainder => ItemComponent::UseRemainder(Box::new(
            ItemStackTemplate(decode_item_stack_template(r, depth + 1)?),
        )),
        ItemComponentKind::UseCooldown => ItemComponent::UseCooldown {
            seconds: Decode::decode(r)?,
            cooldown_group: Decode::decode(r)?,
        },
        ItemComponentKind::DamageResistant => ItemComponent::DamageResistant(Decode::decode(r)?),
        ItemComponentKind::Tool => ItemComponent::Tool {
            rules: Decode::decode(r)?,
            default_mining_speed: Decode::decode(r)?,
            damage_per_block: Decode::decode(r)?,
            can_destroy_blocks_in_creative: Decode::decode(r)?,
        },
        ItemComponentKind::Weapon => ItemComponent::Weapon {
            damage_per_attack: Decode::decode(r)?,
            disable_blocking_for_seconds: Decode::decode(r)?,
        },
        ItemComponentKind::Enchantable => ItemComponent::Enchantable(Decode::decode(r)?),
        ItemComponentKind::Equippable => ItemComponent::Equippable {
            slot: Decode::decode(r)?,
            equip_sound: Decode::decode(r)?,
            model: Decode::decode(r)?,
            camera_overlay: Decode::decode(r)?,
            allowed_entities: Decode::decode(r)?,
            dispensable: Decode::decode(r)?,
            swappable: Decode::decode(r)?,
            damage_on_hurt: Decode::decode(r)?,
            equip_on_interact: Decode::decode(r)?,
            can_be_sheared: Decode::decode(r)?,
            shearing_sound: Decode::decode(r)?,
        },
        ItemComponentKind::Repairable => ItemComponent::Repairable(Decode::decode(r)?),
        ItemComponentKind::Glider => ItemComponent::Glider,
        ItemComponentKind::TooltipStyle => ItemComponent::TooltipStyle(Decode::decode(r)?),
        ItemComponentKind::DeathProtection => ItemComponent::DeathProtection(Decode::decode(r)?),
        ItemComponentKind::BlocksAttacks => ItemComponent::BlocksAttacks {
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
        ItemComponentKind::StoredEnchantments => {
            ItemComponent::StoredEnchantments(Decode::decode(r)?)
        }
        ItemComponentKind::DyedColor => ItemComponent::DyedColor {
            color: Decode::decode(r)?,
        },
        ItemComponentKind::MapId => ItemComponent::MapId(Decode::decode(r)?),
        ItemComponentKind::MapDecorations => ItemComponent::MapDecorations(Decode::decode(r)?),
        ItemComponentKind::MapPostProcessing => {
            ItemComponent::MapPostProcessing(Decode::decode(r)?)
        }
        ItemComponentKind::ChargedProjectiles => {
            let count = VarInt::decode(r)?.0;
            anyhow::ensure!(
                (0..=1024).contains(&count),
                "invalid ChargedProjectiles length"
            );
            let mut items =
                Vec::with_capacity(cautious_capacity::<ItemStackTemplate>(count as usize));
            for _ in 0..count {
                items.push(ItemStackTemplate(decode_item_stack_template(r, depth + 1)?));
            }
            ItemComponent::ChargedProjectiles(items)
        }
        ItemComponentKind::BundleContents => {
            let count = VarInt::decode(r)?.0;
            anyhow::ensure!(
                (0..=i32::MAX).contains(&count),
                "invalid BundleContents length"
            );
            let mut items =
                Vec::with_capacity(cautious_capacity::<ItemStackTemplate>(count as usize));
            for _ in 0..count {
                items.push(ItemStackTemplate(decode_item_stack_template(r, depth + 1)?));
            }
            ItemComponent::BundleContents(items)
        }
        ItemComponentKind::PotionContents => ItemComponent::PotionContents {
            potion_id: Decode::decode(r)?,
            custom_color: Decode::decode(r)?,
            custom_effects: Decode::decode(r)?,
            custom_name: Decode::decode(r)?,
        },
        ItemComponentKind::PotionDurationScale => {
            ItemComponent::PotionDurationScale(Decode::decode(r)?)
        }
        ItemComponentKind::SuspiciousStewEffects => {
            ItemComponent::SuspiciousStewEffects(Decode::decode(r)?)
        }
        ItemComponentKind::WritableBookContent => ItemComponent::WritableBookContent {
            pages: Bounded::<_, 100>::decode(r)?.0,
        },
        ItemComponentKind::WrittenBookContent => ItemComponent::WrittenBookContent {
            raw_title: Decode::decode(r)?,
            filtered_title: Decode::decode(r)?,
            author: Decode::decode(r)?,
            generation: Decode::decode(r)?,
            pages: Decode::decode(r)?,
            resolved: Decode::decode(r)?,
        },
        ItemComponentKind::Trim => ItemComponent::Trim {
            material: Decode::decode(r)?,
            pattern: Decode::decode(r)?,
        },
        ItemComponentKind::DebugStickState => ItemComponent::DebugStickState(Decode::decode(r)?),
        ItemComponentKind::EntityData => ItemComponent::EntityData {
            id: Decode::decode(r)?,
            data: Decode::decode(r)?,
        },
        ItemComponentKind::BucketEntityData => ItemComponent::BucketEntityData(Decode::decode(r)?),
        ItemComponentKind::BlockEntityData => ItemComponent::BlockEntityData {
            id: Decode::decode(r)?,
            data: Decode::decode(r)?,
        },
        ItemComponentKind::Instrument => ItemComponent::Instrument(Decode::decode(r)?),
        ItemComponentKind::ProvidesTrimMaterial => {
            ItemComponent::ProvidesTrimMaterial(Decode::decode(r)?)
        }
        ItemComponentKind::OminousBottleAmplifier => {
            ItemComponent::OminousBottleAmplifier(Decode::decode(r)?)
        }
        ItemComponentKind::JukeboxPlayable => ItemComponent::JukeboxPlayable(Decode::decode(r)?),
        ItemComponentKind::ProvidesBannerPatterns => {
            ItemComponent::ProvidesBannerPatterns(Decode::decode(r)?)
        }
        ItemComponentKind::Recipes => ItemComponent::Recipes(recipes_from_nbt(Value::decode(r)?)?),
        ItemComponentKind::LodestoneTracker => ItemComponent::LodestoneTracker {
            target: Decode::decode(r)?,
            tracked: Decode::decode(r)?,
        },
        ItemComponentKind::FireworkExplosion => {
            ItemComponent::FireworkExplosion(Decode::decode(r)?)
        }
        ItemComponentKind::Fireworks => ItemComponent::Fireworks {
            flight_duration: Decode::decode(r)?,
            explosions: Bounded::<_, 256>::decode(r)?.0,
        },
        ItemComponentKind::Profile => ItemComponent::Profile(Decode::decode(r)?),
        ItemComponentKind::NoteBlockSound => ItemComponent::NoteBlockSound(Decode::decode(r)?),
        ItemComponentKind::BannerPatterns => ItemComponent::BannerPatterns(Decode::decode(r)?),
        ItemComponentKind::BaseColor => ItemComponent::BaseColor(Decode::decode(r)?),
        ItemComponentKind::PotDecorations => ItemComponent::PotDecorations(Box::new({
            let mut sides = [const { None }; 4];
            for side in &mut sides {
                if bool::decode(r)? {
                    *side = Some(ItemStackTemplate(decode_item_stack_template(r, depth + 1)?));
                }
            }
            sides
        })),
        ItemComponentKind::Container => {
            let count = VarInt::decode(r)?.0;
            anyhow::ensure!((0..=256).contains(&count), "invalid Container length");
            let mut items = Vec::with_capacity(cautious_capacity::<Option<ItemStackTemplate>>(
                count as usize,
            ));
            for _ in 0..count {
                items.push(if bool::decode(r)? {
                    Some(ItemStackTemplate(decode_item_stack_template(r, depth + 1)?))
                } else {
                    None
                });
            }
            ItemComponent::Container(items)
        }
        ItemComponentKind::BlockState => ItemComponent::BlockState(Decode::decode(r)?),
        ItemComponentKind::Bees => ItemComponent::Bees(Decode::decode(r)?),
        ItemComponentKind::Lock => ItemComponent::Lock(Decode::decode(r)?),
        ItemComponentKind::ContainerLoot => ItemComponent::ContainerLoot(Decode::decode(r)?),
        ItemComponentKind::BreakSound => ItemComponent::BreakSound(Decode::decode(r)?),
        ItemComponentKind::VillagerVariant => ItemComponent::VillagerVariant(Decode::decode(r)?),
        ItemComponentKind::WolfVariant => ItemComponent::WolfVariant(Decode::decode(r)?),
        ItemComponentKind::WolfSoundVariant => ItemComponent::WolfSoundVariant(Decode::decode(r)?),
        ItemComponentKind::WolfCollar => ItemComponent::WolfCollar(Decode::decode(r)?),
        ItemComponentKind::FoxVariant => ItemComponent::FoxVariant(Decode::decode(r)?),
        ItemComponentKind::SalmonSize => ItemComponent::SalmonSize(Decode::decode(r)?),
        ItemComponentKind::ParrotVariant => ItemComponent::ParrotVariant(Decode::decode(r)?),
        ItemComponentKind::TropicalFishPattern => {
            ItemComponent::TropicalFishPattern(Decode::decode(r)?)
        }
        ItemComponentKind::TropicalFishBaseColor => {
            ItemComponent::TropicalFishBaseColor(Decode::decode(r)?)
        }
        ItemComponentKind::TropicalFishPatternColor => {
            ItemComponent::TropicalFishPatternColor(Decode::decode(r)?)
        }
        ItemComponentKind::MooshroomVariant => ItemComponent::MooshroomVariant(Decode::decode(r)?),
        ItemComponentKind::RabbitVariant => ItemComponent::RabbitVariant(Decode::decode(r)?),
        ItemComponentKind::PigVariant => ItemComponent::PigVariant(Decode::decode(r)?),
        ItemComponentKind::CowVariant => ItemComponent::CowVariant(Decode::decode(r)?),
        ItemComponentKind::ChickenVariant => ItemComponent::ChickenVariant(Decode::decode(r)?),
        ItemComponentKind::FrogVariant => ItemComponent::FrogVariant(Decode::decode(r)?),
        ItemComponentKind::HorseVariant => ItemComponent::HorseVariant(Decode::decode(r)?),
        ItemComponentKind::PaintingVariant => ItemComponent::PaintingVariant(Decode::decode(r)?),
        ItemComponentKind::LlamaVariant => ItemComponent::LlamaVariant(Decode::decode(r)?),
        ItemComponentKind::AxolotlVariant => ItemComponent::AxolotlVariant(Decode::decode(r)?),
        ItemComponentKind::CatVariant => ItemComponent::CatVariant(Decode::decode(r)?),
        ItemComponentKind::CatCollar => ItemComponent::CatCollar(Decode::decode(r)?),
        ItemComponentKind::SheepColor => ItemComponent::SheepColor(Decode::decode(r)?),
        ItemComponentKind::ShulkerColor => ItemComponent::ShulkerColor(Decode::decode(r)?),
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
