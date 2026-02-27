use std::io::Write;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use valence_binary::registry_id::RegistryId;
use valence_binary::{Decode, Encode, IDSet, IdOr, TextComponent, VarInt};
use valence_generated::attributes::EntityAttributeOperation;
use valence_generated::block::BlockKind;
use valence_generated::item::ItemKind;
use valence_ident::Ident;
use valence_nbt::Compound;
use valence_text::Text;

use crate::stack::ItemStack;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Enchantment {
    pub description: Text,
    pub supported_items: IDSet<ItemKind>,
    #[serde(default)]
    pub primary_items: Option<IDSet<ItemKind>>,
    pub weight: i32,
    pub max_level: i32,
    pub min_cost: EnchantmentCost,
    pub max_cost: EnchantmentCost,
    pub anvil_cost: i32,
    pub slots: Vec<EquipmentSlot>,
    pub effects: Compound, // TODO
    #[serde(default)]
    pub exclusive_set: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EnchantmentCost {
    pub base: i32,
    pub per_level_above_first: i32,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[rename_all = "snake_case"]
pub enum EquipmentSlot {
    MainHand = 0,
    OffHand = 1,
    Boots = 2,
    Leggings = 3,
    Chestplate = 4,
    Helmet = 5,
    Body = 6,
    Saddle = 7,
}

impl EquipmentSlot {
    pub const fn number_of_members() -> usize {
        // Please update if number changes!!!
        8
    }
}

impl From<u8> for EquipmentSlot {
    fn from(value: u8) -> Self {
        match value {
            0 => EquipmentSlot::MainHand,
            1 => EquipmentSlot::OffHand,
            2 => EquipmentSlot::Boots,
            3 => EquipmentSlot::Leggings,
            4 => EquipmentSlot::Chestplate,
            5 => EquipmentSlot::Helmet,
            6 => EquipmentSlot::Body,
            7 => EquipmentSlot::Saddle,
            _ => panic!("Invalid equipment slot value: {value}"),
        }
    }
}

impl From<i8> for EquipmentSlot {
    fn from(value: i8) -> Self {
        match value {
            0 => EquipmentSlot::MainHand,
            1 => EquipmentSlot::OffHand,
            2 => EquipmentSlot::Boots,
            3 => EquipmentSlot::Leggings,
            4 => EquipmentSlot::Chestplate,
            5 => EquipmentSlot::Helmet,
            6 => EquipmentSlot::Body,
            7 => EquipmentSlot::Saddle,
            _ => panic!("Invalid equipment slot value: {value}"),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub(crate) enum Patchable<T> {
    #[allow(dead_code)]
    Default(T),
    /// `T`, `crc32c hash`
    Added((T, i32)),
    Removed,
    None,
}
impl<T> Patchable<T> {
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_option(self) -> Option<T> {
        match self {
            Patchable::Default(t) => Some(t),
            Patchable::Added((t, _)) => Some(t),
            _ => None,
        }
    }

    pub(crate) fn as_option(&self) -> Option<&T> {
        match self {
            Patchable::Default(t) => Some(t),
            Patchable::Added((t, _)) => Some(t),
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum DynamicRegistryPlaceholder {
    // FIXME: We can only handle static registries for now
    String(String),
    Id(VarInt),
}

impl Encode for DynamicRegistryPlaceholder {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            DynamicRegistryPlaceholder::String(s) => VarInt(0).encode(&mut w),
            DynamicRegistryPlaceholder::Id(id) => id.encode(&mut w),
        }
    }
}

impl<'a> Decode<'a> for DynamicRegistryPlaceholder {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        // always decode as num.
        let s = VarInt::decode(r)?;
        Ok(DynamicRegistryPlaceholder::Id(s))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    Many(Vec<T>),
    One(T),
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    fn from(item: OneOrMany<T>) -> Self {
        match item {
            OneOrMany::Many(vec) => vec,
            OneOrMany::One(val) => vec![val],
        }
    }
}

fn deserialize_one_or_many<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    let helper = OneOrMany::<T>::deserialize(deserializer)?;
    Ok(helper.into())
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
pub enum ItemComponent {
    /// Arbitrary NBT data that does not fit into other structured components.
    /// Used primarily by data-driven systems and server-side plugins to store
    /// state.
    #[serde(rename = "minecraft:custom_data")]
    CustomData(Compound),

    /// Overrides the default maximum stack size of the item.
    /// Allowed values are between 1 and 99.
    #[serde(rename = "minecraft:max_stack_size")]
    MaxStackSize(VarInt),

    /// The total durability of the item. This is the maximum value the 'Damage'
    /// component can reach before the item breaks.
    #[serde(rename = "minecraft:max_damage")]
    MaxDamage(VarInt),

    /// The current wear/tear of the item. 0 represents a new item,
    /// and higher values indicate more damage.
    #[serde(rename = "minecraft:damage")]
    Damage(VarInt),

    /// If present, the item will not take durability damage when used.
    /// Mechanical equivalent to the old 'Unbreakable: 1b' NBT tag.
    #[serde(rename = "minecraft:unbreakable")]
    Unbreakable,

    /// A custom name for the item, typically set via an anvil.
    /// Usually rendered in italics by the client.
    #[serde(rename = "minecraft:custom_name")]
    CustomName(TextComponent),

    /// Overrides the base name of the item (e.g., "Stone").
    /// Unlike `CustomName`, this is not italicized by default.
    #[serde(rename = "minecraft:item_name")]
    ItemName(TextComponent),

    /// References a specific model file in a resource pack.
    /// Allows a single Item ID to have multiple distinct visual appearances.
    #[serde(rename = "minecraft:item_model")]
    ItemModel(String),

    /// Additional lines of text displayed below the item's name in the tooltip.
    #[serde(rename = "minecraft:lore")]
    Lore(Vec<TextComponent>),

    /// Determines the color of the item's name (Common/Uncommon/Rare/Epic).
    /// Also affects the default glint behavior in some contexts.
    #[serde(rename = "minecraft:rarity")]
    Rarity(Rarity),

    /// A list of enchantments applied to the item and their corresponding
    /// levels.
    #[serde(rename = "minecraft:enchantments")]
    Enchantments(Vec<(DynamicRegistryPlaceholder, VarInt)>), // TODO we cant handle dynamic registries here yet

    /// In Adventure mode, this restricts which blocks a player can place
    /// this specific block on.
    #[serde(rename = "minecraft:can_place_on")]
    #[serde(deserialize_with = "deserialize_one_or_many")]
    CanPlaceOn(Vec<BlockPredicate>),

    /// In Adventure mode, this restricts which blocks the player can break
    /// while holding this item.
    #[serde(rename = "minecraft:can_break")]
    CanBreak(Vec<BlockPredicate>),

    /// Modifies the player's base attributes (like Attack Damage, Movement
    /// Speed, or Max Health) when this item is held or equipped.
    #[serde(rename = "minecraft:attribute_modifiers")]
    AttributeModifiers { modifiers: Vec<AttributeModifier> },

    /// Advanced visual overrides for resource packs.
    #[serde(rename = "minecraft:custom_model_data")]
    CustomModelData {
        /// Generic floating point values used by shaders or model predicates.
        floats: Vec<f32>,
        /// Boolean flags for toggling model parts.
        flags: Vec<bool>,
        /// String identifiers for selecting textures or sub-models.
        strings: Vec<String>,
        /// RGB integer colors for tinting specific model layers.
        colors: Vec<i32>,
    },

    /// Controls the visibility of the item's details.
    #[serde(rename = "minecraft:tooltip_display")]
    TooltipDisplay {
        /// If true, the entire tooltip (including name) is hidden.
        hide_tooltip: bool,
        /// A list of Component IDs that should not show their info in the
        /// tooltip.
        hidden_components: Vec<VarInt>,
    },

    /// The cumulative cost (in levels) added to anvil operations involving this
    /// item. Increases every time the item is repaired or modified.
    #[serde(rename = "minecraft:repair_cost")]
    RepairCost(VarInt),

    /// Internal flag used for creative mode. If present, the item cannot be
    /// picked up or moved within specific creative tabs.
    #[serde(rename = "minecraft:creative_slot_lock")]
    CreativeSlotLock,

    /// Forces the "enchantment purple glow" to be either always on or always
    /// off, regardless of whether the item is actually enchanted.
    #[serde(rename = "minecraft:enchantment_glint_override")]
    EnchantmentGlintOverride(bool),

    /// Used for projectiles (like arrows or tridents) to mark them as "ghost"
    /// items that cannot be picked back up by the player.
    #[serde(rename = "minecraft:intangible_projectile")]
    IntangibleProjectile(Compound),

    /// Defines the nutritional value of the item when eaten.
    #[serde(rename = "minecraft:food")]
    Food {
        /// How many hunger points (half-shanks) are restored.
        nutrition: VarInt,
        /// The multiplier applied to the nutrition to determine saturation.
        saturation_modifier: f32,
        /// If true, the player can eat this even if their hunger bar is full.
        can_always_eat: bool,
    },

    /// Defines how the item is used/consumed (e.g., eating, drinking, or using
    /// a bow).
    #[serde(rename = "minecraft:consumable")]
    Consumable {
        /// The time in seconds required to finish using the item.
        consume_seconds: f32,
        /// The visual pose the player takes (Eat, Drink, Block, etc.).
        animation: ConsumableAnimation,
        /// The sound played during and after consumption.
        sound: IdOr<SoundEventDefinition>,
        /// Whether to spawn particle effects (like food crumbs) while using.
        has_consume_particles: bool,
        /// Status effects (like Poison or Speed) applied when consumption
        /// finishes.
        effects: Vec<ConsumeEffect>,
    },

    /// Defines an item that is returned to the inventory after this one is
    /// used. Example: Eating Mushroom Stew returns an empty Bowl.
    #[serde(rename = "minecraft:use_remainder")]
    UseRemainder(Box<ItemStack>),

    /// Prevents the item from being used again for a set duration.
    #[serde(rename = "minecraft:use_cooldown")]
    UseCooldown {
        /// Duration of the cooldown in seconds.
        seconds: f32,
        /// Optional group ID. All items with the same group will share the
        /// cooldown.
        cooldown_group: Option<String>,
    },

    /// Prevents the item from being destroyed by certain damage types (e.g.,
    /// fire-resistant Netherite).
    #[serde(rename = "minecraft:damage_resistant")]
    DamageResistant(String),

    /// Configures how this item mines blocks.
    #[serde(rename = "minecraft:tool")]
    Tool {
        /// Specific rules for block sets (e.g., "Pickaxes mine stones fast").
        rules: Vec<ToolRule>,
        /// The mining speed used if no specific rule matches.
        default_mining_speed: f32,
        /// Durability lost per block broken.
        damage_per_block: VarInt,
        /// If false, this tool cannot break blocks in Creative mode.
        can_destroy_blocks_in_creative: bool,
    },

    /// Statistics for attacking.
    #[serde(rename = "minecraft:weapon")]
    Weapon {
        /// Base damage added to the player's attack.
        damage_per_attack: VarInt,
        /// The duration (in seconds) that blocking is disabled after an attack
        /// is landed.
        disable_blocking_for_seconds: f32,
    },

    /// Determines how many experience points the item "absorbs" in an
    /// enchanting table.
    #[serde(rename = "minecraft:enchantable")]
    Enchantable(VarInt),

    /// Logic for equipping the item.
    #[serde(rename = "minecraft:equippable")]
    Equippable {
        /// Which body slot this item fits into (Head, Chest, etc.).
        slot: EquipSlot,
        /// Sound played when the item is equipped.
        equip_sound: IdOr<SoundEventDefinition>,
        /// Reference to an equipment-specific model (like 3D armor).
        model: Option<String>,
        /// Texture used when the player's camera is "inside" the item (like a
        /// Pumpkin).
        camera_overlay: Option<String>,
        /// Which entity types are allowed to wear this item.
        allowed_entities: Option<IDSet>,
        /// Whether a Dispenser can equip this onto an entity.
        dispensable: bool,
        /// Whether right-clicking allows swapping this with currently equipped
        /// armor.
        swappable: bool,
        /// If true, the item takes durability damage when the wearer is hurt.
        damage_on_hurt: bool,
    },

    /// Items that can be used in an anvil to repair this item.
    #[serde(rename = "minecraft:repairable")]
    Repairable(IDSet),

    /// Enables Elytra-style flight physics when equipped.
    #[serde(rename = "minecraft:glider")]
    Glider,

    /// References a custom sprite used as the background of the item's tooltip.
    #[serde(rename = "minecraft:tooltip_style")]
    TooltipStyle(String),

    /// Replicates the "Totem of Undying" behavior.
    #[serde(rename = "minecraft:death_protection")]
    DeathProtection(Vec<ConsumeEffect>),

    /// Shield-specific combat logic.
    #[serde(rename = "minecraft:blocks_attacks")]
    BlocksAttacks {
        /// Delay in seconds before blocking becomes active.
        block_delay_seconds: f32,
        /// Scale factor for cooldowns when blocking.
        disable_cooldown_scale: f32,
        /// How much damage is absorbed from specific sources.
        damage_reductions: Vec<DamageReduction>,
        /// Minimum damage required for the shield to take durability loss.
        item_damage_threshold: f32,
        /// Flat durability loss per block.
        item_damage_base: f32,
        /// Multiplier for durability loss based on damage blocked.
        item_damage_factor: f32,
        /// Damage type tag that pierces this shield's blocking logic.
        bypassed_by: Option<String>,
        /// Sound played when a hit is successfully blocked.
        block_sound: Option<IdOr<SoundEventDefinition>>,
        /// Sound played when the shield is disabled (e.g., by an Axe).
        disable_sound: Option<IdOr<SoundEventDefinition>>,
    },

    /// Enchantments contained within an Enchanted Book.
    #[serde(rename = "minecraft:stored_enchantments")]
    StoredEnchantments {
        enchantments: Vec<(RegistryId, VarInt)>,
        show_in_tooltip: bool,
    },

    /// RGB color for leather armor or other dyeable items.
    #[serde(rename = "minecraft:dyed_color")]
    DyedColor {
        /// The packed RGB integer.
        color: i32,
        /// Whether the "Dyed" text is visible in the tooltip.
        show_in_tooltip: bool,
    },

    /// The color used for markings on a Map item.
    #[serde(rename = "minecraft:map_color")]
    MapColor(i32),

    /// The numerical ID associated with a filled Map.
    #[serde(rename = "minecraft:map_id")]
    MapId(VarInt),

    /// NBT data defining markers, banners, and icons shown on a map.
    #[serde(rename = "minecraft:map_decorations")]
    MapDecorations(Compound),

    /// Tracking state for map expansion or locking.
    #[serde(rename = "minecraft:map_post_processing")]
    MapPostProcessing(MapPostProcessingType),

    /// Items currently loaded into a Crossbow.
    #[serde(rename = "minecraft:charged_projectiles")]
    ChargedProjectiles(Vec<ItemStack>),

    /// Items stored inside a Bundle.
    #[serde(rename = "minecraft:bundle_contents")]
    BundleContents(Vec<ItemStack>),

    /// Data for Potion items, including their base type and custom effects.
    #[serde(rename = "minecraft:potion_contents")]
    PotionContents {
        /// The base potion type (e.g., "Invisibility").
        potion_id: Option<RegistryId>,
        /// Custom color for the liquid, overrides the potion's default.
        custom_color: Option<i32>,
        /// Additional status effects not included in the base potion type.
        custom_effects: Vec<PotionEffect>,
        /// An optional name for the specific potion mixture.
        custom_name: Option<String>,
    },

    /// Multiplier for the duration of effects applied by this potion.
    #[serde(rename = "minecraft:potion_duration_scale")]
    PotionDurationScale(f32),

    /// Effects granted by eating Suspicious Stew.
    #[serde(rename = "minecraft:suspicious_stew_effects")]
    SuspiciousStewEffects(Vec<(RegistryId, VarInt)>),

    /// Pages and filtering information for a Book and Quill.
    #[serde(rename = "minecraft:writable_book_content")]
    WritableBookContent { pages: Vec<WritablePage> },

    /// Finalized content for a Written Book.
    #[serde(rename = "minecraft:written_book_content")]
    WrittenBookContent {
        /// The displayed title.
        raw_title: String,
        /// The title after passing through server-side chat filters.
        filtered_title: Option<String>,
        /// The username of the player who signed the book.
        author: String,
        /// How many times the book has been copied (Original, Copy of Original,
        /// etc.).
        generation: VarInt,
        /// Page contents (Rich Text).
        pages: Vec<WrittenPage>,
        /// Whether entity selectors (like @p) have been resolved.
        resolved: bool,
    },
    /// Visual armor customization (Pattern and Material).
    #[serde(rename = "minecraft:trim")]
    Trim {
        material: IdOr<TrimMaterial>,
        pattern: IdOr<TrimPattern>,
        /// Whether the "Armor Trim" lines show in the tooltip.
        show_in_tooltip: bool,
    },

    /// Internal state for the Debug Stick, tracking property toggles.
    #[serde(rename = "minecraft:debug_stick_state")]
    DebugStickState(Compound),

    /// NBT data used to modify an entity when it is spawned from an item (Spawn
    /// Eggs).
    #[serde(rename = "minecraft:entity_data")]
    EntityData { id: RegistryId, data: Compound },

    /// NBT data for entities inside a Bucket (like Fish or Axolotls).
    #[serde(rename = "minecraft:bucket_entity_data")]
    BucketEntityData(Compound),

    /// NBT data for the Block Entity created when this item is placed (Chests,
    /// Signs).
    #[serde(rename = "minecraft:block_entity_data")]
    BlockEntityData { id: RegistryId, data: Compound },

    /// The specific sound and duration associated with a Goat Horn.
    #[serde(rename = "minecraft:instrument")]
    Instrument(IdOr<InstrumentDefinition>),

    /// Marks an item as a valid material for the Armor Trim system.
    #[serde(rename = "minecraft:provides_trim_material")]
    ProvidesTrimMaterial(ModePair<String, IdOr<TrimMaterial>>),

    /// The level of Bad Omen granted by an Ominous Bottle (0-4).
    #[serde(rename = "minecraft:ominous_bottle_amplifier")]
    OminousBottleAmplifier(VarInt),

    /// Configuration for items that can be played in a Jukebox.
    #[serde(rename = "minecraft:jukebox_playable")]
    JukeboxPlayable {
        /// Reference to a Jukebox Song.
        song: ModePair<String, IdOr<JukeboxSong>>,
        show_in_tooltip: bool,
    },

    /// Marks an item as a valid pattern for the Loom (Banner Patterns).
    #[serde(rename = "minecraft:provides_banner_patterns")]
    ProvidesBannerPatterns(String),

    /// A list of recipe IDs that a Knowledge Book will teach the player.
    #[serde(rename = "minecraft:recipes")]
    Recipes(Compound),

    /// Tracking data for a Compass pointing to a specific Lodestone.
    #[serde(rename = "minecraft:lodestone_tracker")]
    LodestoneTracker {
        /// The dimension and coordinate of the target. None if the compass is
        /// spinning.
        target: Option<LodestoneTarget>,
        /// If true, the compass becomes a normal compass if the lodestone is
        /// destroyed.
        tracked: bool,
    },

    /// Individual explosion properties for a Firework Star.
    #[serde(rename = "minecraft:firework_explosion")]
    FireworkExplosion(FireworkExplosionData),

    /// Flight and explosion data for a Firework Rocket.
    #[serde(rename = "minecraft:fireworks")]
    Fireworks {
        flight_duration: VarInt,
        explosions: Vec<FireworkExplosionData>,
    },

    /// Data for a Player Head, including the skin texture and UUID.
    #[serde(rename = "minecraft:profile")]
    Profile(ResolvableProfile),

    /// The sound played by a Note Block if this player head is placed on top of
    /// it.
    #[serde(rename = "minecraft:note_block_sound")]
    NoteBlockSound(String),

    /// Visual layers for a Banner or Shield.
    #[serde(rename = "minecraft:banner_patterns")]
    BannerPatterns(Vec<BannerLayer>),

    /// The base dye color for a Banner.
    #[serde(rename = "minecraft:base_color")]
    BaseColor(VarInt),

    /// The four item IDs used as patterns on a Decorated Pot.
    #[serde(rename = "minecraft:pot_decorations")]
    PotDecorations(Vec<RegistryId>),

    /// The inventory contents of a block (like a Chest or Shulker Box).
    #[serde(rename = "minecraft:container")]
    Container(Vec<ItemStack>),

    /// Block state property overrides (e.g., "lit: true").
    #[serde(rename = "minecraft:block_state")]
    BlockState(Vec<(String, String)>),

    /// Data for bees currently inside a Beehive item.
    #[serde(rename = "minecraft:bees")]
    Bees(Vec<BeeData>),

    /// The "Key" name required to open a container if it has a Lock component.
    #[serde(rename = "minecraft:lock")]
    Lock(String),

    /// Reference to a Loot Table for an unopened chest.
    #[serde(rename = "minecraft:container_loot")]
    ContainerLoot(Compound),

    /// Overrides the default sound played when this specific item breaks.
    #[serde(rename = "minecraft:break_sound")]
    BreakSound(IdOr<SoundEventDefinition>),

    /// Biome-specific variant of a Villager (e.g., Desert, Plains).
    #[serde(rename = "minecraft:villager_variant")]
    VillagerVariant(RegistryId),

    /// Skin variant for a Wolf.
    #[serde(rename = "minecraft:wolf_variant")]
    WolfVariant(RegistryId),

    /// Determines the bark/growl sounds for a Wolf.
    #[serde(rename = "minecraft:wolf_sound_variant")]
    WolfSoundVariant(RegistryId),

    /// Dye color of a Wolf's collar.
    #[serde(rename = "minecraft:wolf_collar")]
    WolfCollar(DyeColor),

    /// Type of Fox (Red or Snow).
    #[serde(rename = "minecraft:fox_variant")]
    FoxVariant(FoxType),

    /// Size of a Salmon (Small, Medium, Large).
    #[serde(rename = "minecraft:salmon_size")]
    SalmonSize(SalmonScale),

    /// Color of a Parrot.
    #[serde(rename = "minecraft:parrot_variant")]
    ParrotVariant(ParrotType),

    /// Pattern type for a Tropical Fish.
    #[serde(rename = "minecraft:tropical_fish_pattern")]
    TropicalFishPattern(TropicalFishPattern),

    /// Primary color of a Tropical Fish.
    #[serde(rename = "minecraft:tropical_fish_base_color")]
    TropicalFishBaseColor(DyeColor),

    /// Secondary color of a Tropical Fish.
    #[serde(rename = "minecraft:tropical_fish_pattern_color")]
    TropicalFishPatternColor(DyeColor),

    /// Type of Mooshroom (Red or Brown).
    #[serde(rename = "minecraft:mooshroom_variant")]
    MooshroomVariant(MooshroomType),

    /// Breed of a Rabbit.
    #[serde(rename = "minecraft:rabbit_variant")]
    RabbitVariant(RabbitType),

    /// Skin variant for a Pig.
    #[serde(rename = "minecraft:pig_variant")]
    PigVariant(RegistryId),

    /// Skin variant for a Cow.
    #[serde(rename = "minecraft:cow_variant")]
    CowVariant(RegistryId),

    /// Skin variant for a Chicken.
    #[serde(rename = "minecraft:chicken_variant")]
    ChickenVariant(ModePair<String, RegistryId>),

    /// Biome variant for a Frog.
    #[serde(rename = "minecraft:frog_variant")]
    FrogVariant(RegistryId),

    /// Color and marking variant for a Horse.
    #[serde(rename = "minecraft:horse_variant")]
    HorseVariant(HorseColor),

    /// The specific painting texture and dimensions.
    #[serde(rename = "minecraft:painting_variant")]
    PaintingVariant(IdOr<PaintingVariantDefinition>),

    /// Color variant for a Llama.
    #[serde(rename = "minecraft:llama_variant")]
    LlamaVariant(LlamaColor),

    /// Color variant for an Axolotl.
    #[serde(rename = "minecraft:axolotl_variant")]
    AxolotlVariant(AxolotlType),

    /// Breed variant for a Cat.
    #[serde(rename = "minecraft:cat_variant")]
    CatVariant(RegistryId),

    /// Dye color of a Cat's collar.
    #[serde(rename = "minecraft:cat_collar")]
    CatCollar(DyeColor),

    /// Natural wool color of a Sheep.
    #[serde(rename = "minecraft:sheep_color")]
    SheepColor(DyeColor),

    /// Shell color of a Shulker.
    #[serde(rename = "minecraft:shulker_color")]
    ShulkerColor(DyeColor),
}

impl ItemComponent {
    pub fn id(&self) -> u32 {
        match self {
            ItemComponent::CustomData { .. } => 0,
            ItemComponent::MaxStackSize { .. } => 1,
            ItemComponent::MaxDamage { .. } => 2,
            ItemComponent::Damage { .. } => 3,
            ItemComponent::Unbreakable => 4,
            ItemComponent::CustomName { .. } => 5,
            ItemComponent::ItemName { .. } => 6,
            ItemComponent::ItemModel { .. } => 7,
            ItemComponent::Lore { .. } => 8,
            ItemComponent::Rarity { .. } => 9,
            ItemComponent::Enchantments { .. } => 10,
            ItemComponent::CanPlaceOn { .. } => 11,
            ItemComponent::CanBreak { .. } => 12,
            ItemComponent::AttributeModifiers { .. } => 13,
            ItemComponent::CustomModelData { .. } => 14,
            ItemComponent::TooltipDisplay { .. } => 15,
            ItemComponent::RepairCost { .. } => 16,
            ItemComponent::CreativeSlotLock => 17,
            ItemComponent::EnchantmentGlintOverride { .. } => 18,
            ItemComponent::IntangibleProjectile { .. } => 19,
            ItemComponent::Food { .. } => 20,
            ItemComponent::Consumable { .. } => 21,
            ItemComponent::UseRemainder { .. } => 22,
            ItemComponent::UseCooldown { .. } => 23,
            ItemComponent::DamageResistant { .. } => 24,
            ItemComponent::Tool { .. } => 25,
            ItemComponent::Weapon { .. } => 26,
            ItemComponent::Enchantable { .. } => 27,
            ItemComponent::Equippable { .. } => 28,
            ItemComponent::Repairable { .. } => 29,
            ItemComponent::Glider => 30,
            ItemComponent::TooltipStyle { .. } => 31,
            ItemComponent::DeathProtection { .. } => 32,
            ItemComponent::BlocksAttacks { .. } => 33,
            ItemComponent::StoredEnchantments { .. } => 34,
            ItemComponent::DyedColor { .. } => 35,
            ItemComponent::MapColor { .. } => 36,
            ItemComponent::MapId { .. } => 37,
            ItemComponent::MapDecorations { .. } => 38,
            ItemComponent::MapPostProcessing { .. } => 39,
            ItemComponent::ChargedProjectiles { .. } => 40,
            ItemComponent::BundleContents { .. } => 41,
            ItemComponent::PotionContents { .. } => 42,
            ItemComponent::PotionDurationScale { .. } => 43,
            ItemComponent::SuspiciousStewEffects { .. } => 44,
            ItemComponent::WritableBookContent { .. } => 45,
            ItemComponent::WrittenBookContent { .. } => 46,
            ItemComponent::Trim { .. } => 47,
            ItemComponent::DebugStickState { .. } => 48,
            ItemComponent::EntityData { .. } => 49,
            ItemComponent::BucketEntityData { .. } => 50,
            ItemComponent::BlockEntityData { .. } => 51,
            ItemComponent::Instrument { .. } => 52,
            ItemComponent::ProvidesTrimMaterial { .. } => 53,
            ItemComponent::OminousBottleAmplifier { .. } => 54,
            ItemComponent::JukeboxPlayable { .. } => 55,
            ItemComponent::ProvidesBannerPatterns { .. } => 56,
            ItemComponent::Recipes { .. } => 57,
            ItemComponent::LodestoneTracker { .. } => 58,
            ItemComponent::FireworkExplosion { .. } => 59,
            ItemComponent::Fireworks { .. } => 60,
            ItemComponent::Profile { .. } => 61,
            ItemComponent::NoteBlockSound { .. } => 62,
            ItemComponent::BannerPatterns { .. } => 63,
            ItemComponent::BaseColor { .. } => 64,
            ItemComponent::PotDecorations { .. } => 65,
            ItemComponent::Container { .. } => 66,
            ItemComponent::BlockState { .. } => 67,
            ItemComponent::Bees { .. } => 68,
            ItemComponent::Lock { .. } => 69,
            ItemComponent::ContainerLoot { .. } => 70,
            ItemComponent::BreakSound { .. } => 71,
            ItemComponent::VillagerVariant { .. } => 72,
            ItemComponent::WolfVariant { .. } => 73,
            ItemComponent::WolfSoundVariant { .. } => 74,
            ItemComponent::WolfCollar { .. } => 75,
            ItemComponent::FoxVariant { .. } => 76,
            ItemComponent::SalmonSize { .. } => 77,
            ItemComponent::ParrotVariant { .. } => 78,
            ItemComponent::TropicalFishPattern { .. } => 79,
            ItemComponent::TropicalFishBaseColor { .. } => 80,
            ItemComponent::TropicalFishPatternColor { .. } => 81,
            ItemComponent::MooshroomVariant { .. } => 82,
            ItemComponent::RabbitVariant { .. } => 83,
            ItemComponent::PigVariant { .. } => 84,
            ItemComponent::CowVariant { .. } => 85,
            ItemComponent::ChickenVariant { .. } => 86,
            ItemComponent::FrogVariant { .. } => 87,
            ItemComponent::HorseVariant { .. } => 88,
            ItemComponent::PaintingVariant { .. } => 89,
            ItemComponent::LlamaVariant { .. } => 90,
            ItemComponent::AxolotlVariant { .. } => 91,
            ItemComponent::CatVariant { .. } => 92,
            ItemComponent::CatCollar { .. } => 93,
            ItemComponent::SheepColor { .. } => 94,
            ItemComponent::ShulkerColor { .. } => 95,
        }
    }

    pub fn hash(&self) -> i32 {
        // TODO: implement if required
        0
    }
}

/// A helper struct for protocol fields that start with a "Mode" byte.
///
/// In 1.21, several components (like Jukebox Songs or Trim Materials) are
/// encoded as:
/// - Byte `0`: Followed by Type A (usually a String Identifier).
/// - Byte `1`: Followed by Type B (usually an ID or Inline Definition).
#[derive(Clone, PartialEq, Debug)]
pub enum ModePair<A, B> {
    /// Mode 0: Usually references a registry key by name.
    Mode0(A),
    /// Mode 1: Usually references a registry ID or defines the data inline.
    Mode1(B),
}

impl<A: Encode, B: Encode> Encode for ModePair<A, B> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            ModePair::Mode0(a) => {
                0_u8.encode(&mut w)?;
                a.encode(w)
            }
            ModePair::Mode1(b) => {
                1_u8.encode(&mut w)?;
                b.encode(w)
            }
        }
    }
}
impl<'a, A: Decode<'a>, B: Decode<'a>> Decode<'a> for ModePair<A, B> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let mode = u8::decode(r)?;
        match mode {
            0 => Ok(ModePair::Mode0(A::decode(r)?)),
            1 => Ok(ModePair::Mode1(B::decode(r)?)),
            _ => anyhow::bail!("Invalid ModePair byte: {mode}"),
        }
    }
}

/// Defines a rule for matching a block in the world.
/// Used by `CanPlaceOn` and `CanBreak` in Adventure Mode.
#[derive(Clone, PartialEq, Debug, Encode)]
pub struct BlockPredicate {
    /// If None, matches any block ID.
    pub blocks: Option<IDSet<BlockKind>>,

    /// Matches specific block state properties (e.g., `lit=true`).
    pub properties: Option<Vec<Property>>,

    /// Matches the Block Entity's NBT data.
    pub nbt: Option<Compound>,

    /// (1.21+) Matches if the block drops an item containing these EXACT
    /// components. This is a strict equality check.
    pub exact_components: Vec<ExactComponentMatcher>,

    /// (1.21+) Matches if the block drops an item containing specific NBT
    /// structures within specific components.
    pub partial_components: Vec<PartialComponentMatcher>,
}

// A specific Block State property requirement.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct Property {
    /// The name of the property (e.g., "facing", "waterlogged").
    pub name: String,

    /// Either an exact value or a min/max range for the property.
    pub value: PropertyValue,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PropertyValue {
    /// An exact string value.
    Exact(String),
    /// A min max string value.
    MinMax { min: String, max: String },
}

// encoded as bool followed by one if true or min and max if false
impl Encode for PropertyValue {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            PropertyValue::Exact(v) => {
                true.encode(&mut w)?;
                v.encode(w)
            }
            PropertyValue::MinMax { min, max } => {
                false.encode(&mut w)?;
                min.encode(&mut w)?;
                max.encode(w)
            }
        }
    }
}
impl<'a> Decode<'a> for PropertyValue {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let is_exact = bool::decode(r)?;
        if is_exact {
            let v = String::decode(r)?;
            Ok(PropertyValue::Exact(v))
        } else {
            let min = String::decode(r)?;
            let max = String::decode(r)?;
            Ok(PropertyValue::MinMax { min, max })
        }
    }
}

/// Matches a component exactly.
#[derive(Clone, PartialEq, Debug, Encode)]
pub struct ExactComponentMatcher {
    /// The ID of the component to check.
    pub component_type: VarInt,
    /// The encoded data of that component.
    pub component_data: ItemComponent,
}

/// Matches a subset of data within a component using NBT.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct PartialComponentMatcher {
    /// The ID of the component to check.
    pub component_type: VarInt,
    /// An NBT matcher to apply to that component's internal data.
    pub predicate: Compound,
}

/// Modifies a player's attributes (like Strength or Speed).
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AttributeModifier {
    /// The ID of the attribute to modify in the registry.
    pub attribute_id: RegistryId,

    /// A unique identifier for this modifier instance.
    /// Used to prevent stacking the same modifier multiple times from different
    /// sources.
    pub modifier_id: Ident<String>,

    /// The numerical amount to change the attribute by.
    pub value: f64,

    /// How the math is applied.
    /// (Add): X = X + Value
    /// (Multiply Base): X = X + (`BaseValue` * Value)
    /// (Multiply Total): X = X * (1 + Value)
    pub operation: EntityAttributeOperation,

    /// Which slot the item must be in for this to work.
    pub slot: AttributeSlot,
}

/// Defines custom mining speed logic for a tool.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ToolRule {
    /// The blocks this rule applies to.
    pub blocks: IDSet,

    /// If present, overrides the mining speed for these blocks.
    pub speed: Option<f32>,

    /// If present and true, this tool is considered "correct" for the block
    /// (meaning the block will drop items when broken).
    pub correct_drop_for_blocks: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LodestoneTarget {
    /// The namespaced key of the dimension (e.g., "`minecraft:the_nether`").
    pub dimension: String,

    /// The precise X, Y, Z coordinates of the Lodestone block.
    pub position: (VarInt, VarInt, VarInt),
}

/// Defines a sound event, either by referencing the registry or defining it
/// on the fly.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SoundEventDefinition {
    /// The identifier of the sound (e.g., "minecraft:entity.pig.ambient").
    /// In 1.21, this can be a direct String or a Registry ID.
    pub sound: ModePair<String, RegistryId>,

    /// A fixed range (in blocks) for the sound. If None, uses the default.
    pub range: Option<f32>,
}

/// Defines a material used to trim armor (e.g., Gold, Amethyst).
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrimMaterial {
    /// Corresponds to "Suffix" in the Wiki.
    /// This string is appended to the texture path (e.g., "amethyst" ->
    /// "`trims/items/leggings_trim_amethyst`").
    pub asset_name: String,

    /// Allows specific armor materials to use a different texture suffix.
    ///
    /// Structure:
    /// - Key: Armor Material Model Name (Identifier, e.g.,
    ///   "minecraft:netherite")
    /// - Value: Overridden Asset Name (String, e.g., "`amethyst_darker`")
    pub overrides: Vec<(Ident<String>, String)>,

    /// Corresponds to "Description" in the Wiki.
    /// The text displayed in the item tooltip (e.g., "Amethyst Material").
    pub description: TextComponent,
}

/// Defines the shape/pattern of the armor trim (e.g., Vex, Coast).
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TrimPattern {
    /// The asset ID for the texture pattern.
    pub asset_id: String,

    /// The Smithing Template item required to apply this pattern.
    pub template_item: RegistryId,

    /// The text displayed in the tooltip (e.g., "Vex Armor Trim").
    pub description: TextComponent,

    /// If true, the pattern is applied as a "Decal" (no color blending).
    pub decal: bool,
}

/// Defines a Goat Horn instrument.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstrumentDefinition {
    /// The sound played when the horn is used.
    pub sound_event: IdOr<SoundEventDefinition>,

    /// How long the horn plays (in seconds).
    pub use_duration: f32,

    /// The audible range of the horn.
    pub range: f32,

    /// The description shown in the tooltip (e.g., "Ponder").
    pub description: TextComponent,
}

/// Defines a Music Disc song.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JukeboxSong {
    /// The sound event to play.
    pub sound_event: IdOr<SoundEventDefinition>,

    /// The song title shown in the "Now Playing" action bar.
    pub description: Text,

    /// The duration of the song in seconds.
    pub length_seconds: f32,

    /// The Redstone signal strength (0-15) emitted by the Jukebox while
    /// playing.
    pub comparator_output: VarInt,
}

/// Defines a variant of a painting entity.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PaintingVariantDefinition {
    /// The path to the texture in the resource pack.
    pub asset_id: String,

    /// Width in blocks (1 block = 16 pixels).
    pub width: VarInt,

    /// Height in blocks.
    pub height: VarInt,
}

/// Defines a single explosion in a Firework Rocket.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FireworkExplosionData {
    /// The shape (Small Ball, Large Ball, Star, Creeper, Burst).
    pub shape: VarInt,

    /// List of RGB integers for the initial explosion colors.
    pub colors: Vec<i32>,

    /// List of RGB integers that the particles fade into.
    pub fade_colors: Vec<i32>,

    /// If true, particles leave a trail behind them.
    pub has_trail: bool,

    /// If true, particles crackle/flash after the explosion.
    pub has_twinkle: bool,
}

/// Defines a layer on a Banner.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BannerLayer {
    /// The pattern type (Flower, Skull, Stripe, etc.).
    pub pattern: IdOr<BannerPattern>,

    /// The dye color ID (0-15) for this layer.
    pub color: DyeColor,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BannerPattern {
    /// The texture identifier (e.g., "minecraft:flower").
    pub asset_id: String,

    /// The translation key for the pattern name (e.g.,
    /// "block.minecraft.banner.flower").
    pub translation_key: String,
}

/// A page in a Book and Quill (Writable).
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WritablePage {
    /// The raw text entered by the player.
    pub raw: String,

    /// If the server runs a chat filter, this is the filtered version.
    /// If None, the raw text is considered safe to display.
    pub filtered: Option<String>,
}

/// A page in a Finished Book (Written).
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WrittenPage {
    /// The JSON text component for the page content.
    pub raw: TextComponent,

    /// Optional filtered version for chat safety settings.
    pub filtered: Option<TextComponent>,
}

/// Represents a Player's Game Profile (Skin/UUID).
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolvableProfile {
    /// The player's username.
    pub name: Option<String>,

    /// The player's UUID.
    pub id: Option<Uuid>,

    /// Properties, primarily the "textures" property containing the skin URL.
    pub properties: Vec<ProfileProperty>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProfileProperty {
    pub name: String,
    /// The base64 encoded value.
    pub value: String,
    /// The optional public key signature (Yggdrasil).
    pub signature: Option<String>,
}

/// Information about a Bee inside a Beehive.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BeeData {
    /// The NBT data of the Bee entity itself (Health, Name, etc.).
    pub entity_data: Compound,

    /// How many ticks the bee has been inside the hive.
    pub ticks_in_hive: VarInt,

    /// The minimum ticks required before the bee can leave.
    pub min_ticks_in_hive: VarInt,
}

/// A wrapper for the various effects caused by consuming an item.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ConsumeEffect {
    pub data: ConsumeEffectData,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
// This is a "registry" but im not making a
// RegistryId impl for it cuase this is its only use
// #[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsumeEffectData {
    /// Type 0: Apply Effects
    ApplyEffects {
        /// List of potion effects to apply.
        effects: Vec<PotionEffect>,
        /// Chance (0.0 to 1.0) that these effects are applied.
        probability: f32,
    },
    /// Type 1: Remove Effects
    RemoveEffects(IDSet), // Set of Potion IDs to cure/remove.
    /// Type 2: Clear All Effects
    ClearAllEffects,
    /// Type 3: Teleport Randomly (Chorus Fruit behavior)
    TeleportRandomly {
        /// The horizontal radius to search for a safe spot.
        diameter: f32,
    },
    /// Type 4: Play Sound
    PlaySound(IdOr<SoundEventDefinition>),
}

/// A standard Potion Effect.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PotionEffect {
    /// The ID of the effect (Speed, Jump Boost, etc.).
    pub id: RegistryId, // TODO

    /// The level of the effect (0 = Level 1, 1 = Level 2).
    pub amplifier: VarInt,

    /// Duration in ticks. -1 indicates infinite duration.
    pub duration: VarInt,

    /// If true, particles are translucent (like Beacon effects).
    pub ambient: bool,

    /// If false, no particles are shown.
    pub show_particles: bool,

    /// If true, the effect icon is displayed in the top-right of theinventory.
    pub show_icon: bool,
}

/// Shield logic for reducing damage.
#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DamageReduction {
    /// The angle (in degrees) in front of the player that is blocked.
    pub horizontal_blocking_angle: f32,

    /// Specific damage types this reduction applies to. None = All.
    pub damage_type: Option<IDSet>,

    /// Flat amount of damage removed.
    pub base: f32,

    /// Percentage of remaining damage removed (0.0 to 1.0).
    pub factor: f32,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Common,   // White
    Uncommon, // Yellow
    Rare,     // Aqua
    Epic,     // Purple
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapPostProcessingType {
    Lock,  // The map has been locked in a Cartography Table.
    Scale, // The map is being zoomed out.
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsumableAnimation {
    None,
    Eat,
    Drink,
    Block, // Shield block animation
    Bow,
    Spear, // Trident throw
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EquipSlot {
    MainHand,
    Feet,
    Legs,
    Chest,
    Head,
    OffHand,
    Body, // Horse armor / Llama carpet
}

#[derive(Clone, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributeSlot {
    Any,
    MainHand,
    OffHand,
    Hand, // MainHand or OffHand
    Feet,
    Legs,
    Chest,
    Head,
    Armor, // Any armor slot
    Body,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DyeColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoxType {
    Red,
    Snow,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SalmonScale {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParrotType {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TropicalFishPattern {
    Kob,
    Sunstreak,
    Snooper,
    Dasher,
    Brinely,
    Spotty,
    Flopper,
    Stripey,
    Glitter,
    Blockfish,
    Betty,
    Clayfish,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MooshroomType {
    Red,
    Brown,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RabbitType {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    Evil, // "Toast"
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorseColor {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlamaColor {
    Creamy,
    White,
    Brown,
    Gray,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxolotlType {
    Lucy, // Pink
    Wild, // Brown
    Gold,
    Cyan,
    Blue,
}
