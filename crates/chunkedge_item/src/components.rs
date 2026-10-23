use std::io::Write;

use chunkedge_binary::{Bounded, Decode, Either, Encode, IDSet, IdOr, TextComponent, VarInt};
use chunkedge_generated::attributes::EntityAttributeOperation;
use chunkedge_generated::item_component_type as component_id;
use chunkedge_generated::registry_id::RegistryId;
use chunkedge_ident::Ident;
use chunkedge_nbt::{Compound, Value};
use uuid::Uuid;

use crate::ItemStackTemplate;

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

// Keep payload variants and their wire IDs in one declaration. The generated kind
// enum checks registry coverage; matching it exhaustively requires a decoder for
// every component, just as matching ItemComponent requires every encoder.
macro_rules! item_components {
    ($(
        $(#[$meta:meta])* $variant:ident
        $(($($ty:ty),* $(,)?))?
        $({ $($(#[$field_meta:meta])* $field:ident: $field_ty:ty),* $(,)? })?
    ),* $(,)?) => {
        #[derive(Clone, PartialEq, Debug)]
        pub enum ItemComponent {
            $($(#[$meta])* $variant $(($($ty),*))?
                $({ $($(#[$field_meta])* $field: $field_ty),* })?,)*
        }

        #[derive(Copy, Clone, chunkedge_protocol_macros::RegistryId)]
        #[packet(registry = component_id)]
        pub(crate) enum ItemComponentKind { $($variant,)* }

        impl ItemComponent {
            pub const fn id(&self) -> u32 {
                match self {
                    $(item_components!(@pattern $variant $(($($ty),*))? $({$($field),*})?)
                        => ItemComponentKind::$variant.id() as u32,)*
                }
            }
        }
    };
    (@pattern $variant:ident ($($ty:ty),*)) => { Self::$variant(..) };
    (@pattern $variant:ident {$($field:ident),*}) => { Self::$variant { .. } };
    (@pattern $variant:ident) => { Self::$variant };
}

item_components! {
    /// Arbitrary NBT data that does not fit into other structured components.
    /// Used primarily by data-driven systems and server-side plugins to store
    /// state.
    CustomData(Compound),

    /// Overrides the default maximum stack size of the item.
    /// Allowed values are between 1 and 99.
    MaxStackSize(VarInt),

    /// The total durability of the item. This is the maximum value the 'Damage'
    /// component can reach before the item breaks.
    MaxDamage(VarInt),

    /// The current wear/tear of the item. 0 represents a new item,
    /// and higher values indicate more damage.
    Damage(VarInt),

    /// If present, the item will not take durability damage when used.
    /// Mechanical equivalent to the old 'Unbreakable: 1b' NBT tag.
    Unbreakable,

    /// A custom name for the item, typically set via an anvil.
    /// Usually rendered in italics by the client.
    CustomName(TextComponent),

    /// Movement and vibration behavior while using the item.
    UseEffects(UseEffects),
    /// Minimum attack charge required to use the item as a weapon.
    MinimumAttackCharge(f32),
    /// Raw ID in the damage-type registry.
    DamageType(RegistryId),

    /// Overrides the base name of the item (e.g., "Stone").
    /// Unlike `CustomName`, this is not italicized by default.
    ItemName(TextComponent),

    /// References a specific model file in a resource pack.
    /// Allows a single Item ID to have multiple distinct visual appearances.
    ItemModel(String),

    /// Additional lines of text displayed below the item's name in the tooltip.
    Lore(Vec<TextComponent>),

    /// Determines the color of the item's name (Common/Uncommon/Rare/Epic).
    /// Also affects the default glint behavior in some contexts.
    Rarity(Rarity),

    /// A list of enchantments applied to the item and their corresponding
    /// levels.
    Enchantments(Vec<(RegistryId, VarInt)>),

    /// In Adventure mode, this restricts which blocks a player can place
    /// this specific block on.
    CanPlaceOn(Vec<BlockPredicate>),

    /// In Adventure mode, this restricts which blocks the player can break
    /// while holding this item.
    CanBreak(Vec<BlockPredicate>),

    /// Modifies the player's base attributes (like Attack Damage, Movement
    /// Speed, or Max Health) when this item is held or equipped.
    AttributeModifiers { modifiers: Vec<AttributeModifier> },

    /// Advanced visual overrides for resource packs.
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
    TooltipDisplay {
        /// If true, the entire tooltip (including name) is hidden.
        hide_tooltip: bool,
        /// A list of Component IDs that should not show their info in the
        /// tooltip.
        hidden_components: Vec<VarInt>,
    },

    /// The cumulative cost (in levels) added to anvil operations involving this
    /// item. Increases every time the item is repaired or modified.
    RepairCost(VarInt),

    /// Internal flag used for creative mode. If present, the item cannot be
    /// picked up or moved within specific creative tabs.
    CreativeSlotLock,

    /// Forces the "enchantment purple glow" to be either always on or always
    /// off, regardless of whether the item is actually enchanted.
    EnchantmentGlintOverride(bool),

    /// Used for projectiles (like arrows or tridents) to mark them as "ghost"
    /// items that cannot be picked back up by the player.
    IntangibleProjectile(Compound),

    /// Defines the nutritional value of the item when eaten.
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
    UseRemainder(Box<ItemStackTemplate>),

    /// Prevents the item from being used again for a set duration.
    UseCooldown {
        /// Duration of the cooldown in seconds.
        seconds: f32,
        /// Optional group ID. All items with the same group will share the
        /// cooldown.
        cooldown_group: Option<String>,
    },

    /// Prevents the item from being destroyed by certain damage types (e.g.,
    /// fire-resistant Netherite).
    DamageResistant(IDSet),

    /// Configures how this item mines blocks.
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
    Weapon {
        /// Base damage added to the player's attack.
        damage_per_attack: VarInt,
        /// The duration (in seconds) that blocking is disabled after an attack
        /// is landed.
        disable_blocking_for_seconds: f32,
    },

    /// Determines how many experience points the item "absorbs" in an
    /// enchanting table.
    Enchantable(VarInt),

    AttackRange(AttackRange),

    /// Logic for equipping the item.
    Equippable {
        /// Which body slot this item fits into (Head, Chest, etc.).
        slot: EquipSlot,
        /// Sound played when the item is equipped.
        equip_sound: IdOr<SoundEventDefinition>,
        /// Reference to an equipment-specific model (like 3D armor).
        model: Option<Ident<String>>,
        /// Texture used when the player's camera is "inside" the item (like a
        /// Pumpkin).
        camera_overlay: Option<Ident<String>>,
        /// Which entity types are allowed to wear this item.
        allowed_entities: Option<IDSet>,
        /// Whether a Dispenser can equip this onto an entity.
        dispensable: bool,
        /// Whether right-clicking allows swapping this with currently equipped
        /// armor.
        swappable: bool,
        /// If true, the item takes durability damage when the wearer is hurt.
        damage_on_hurt: bool,
        /// Whether interacting with an entity equips the item onto it.
        equip_on_interact: bool,
        /// Whether the item can be sheared off its wearer.
        can_be_sheared: bool,
        /// Sound played when the item is sheared off.
        shearing_sound: IdOr<SoundEventDefinition>,
    },

    /// Items that can be used in an anvil to repair this item.
    Repairable(IDSet),

    /// Enables Elytra-style flight physics when equipped.
    Glider,

    /// References a custom sprite used as the background of the item's tooltip.
    TooltipStyle(String),

    /// Replicates the "Totem of Undying" behavior.
    DeathProtection(Vec<ConsumeEffect>),

    /// Shield-specific combat logic.
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
        /// Damage types that pierce this shield's blocking logic.
        bypassed_by: Option<IDSet>,
        /// Sound played when a hit is successfully blocked.
        block_sound: Option<IdOr<SoundEventDefinition>>,
        /// Sound played when the shield is disabled (e.g., by an Axe).
        disable_sound: Option<IdOr<SoundEventDefinition>>,
    },

    /// Properties of a piercing (jab) attack.
    PiercingWeapon(PiercingWeapon),
    /// Damage, knockback and dismount conditions for a kinetic attack.
    KineticWeapon(KineticWeapon),
    /// The item's swing animation and duration in ticks.
    SwingAnimation(SwingAnimation),
    /// Additional item count required by a merchant offer.
    AdditionalTradeCost(VarInt),
    /// The dye color supplied by this item.
    Dye(DyeColor),

    /// Enchantments contained within an Enchanted Book.
    StoredEnchantments(Vec<(RegistryId, VarInt)>),

    /// RGB color for leather armor or other dyeable items.
    DyedColor {
        /// The packed RGB integer.
        color: i32,
    },

    /// The color used for markings on a Map item.
    MapColor(i32),

    /// The numerical ID associated with a filled Map.
    MapId(VarInt),

    /// NBT data defining markers, banners, and icons shown on a map.
    MapDecorations(Compound),

    /// Tracking state for map expansion or locking.
    MapPostProcessing(MapPostProcessingType),

    /// Items currently loaded into a Crossbow.
    ChargedProjectiles(Vec<ItemStackTemplate>),

    /// Items stored inside a Bundle.
    BundleContents(Vec<ItemStackTemplate>),

    /// Data for Potion items, including their base type and custom effects.
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
    PotionDurationScale(f32),

    /// Effects granted by eating Suspicious Stew.
    SuspiciousStewEffects(Vec<(RegistryId, VarInt)>),

    /// Pages and filtering information for a Book and Quill.
    WritableBookContent { pages: Vec<WritablePage> },

    /// Finalized content for a Written Book.
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
    Trim {
        material: IdOr<TrimMaterial>,
        pattern: IdOr<TrimPattern>,
    },

    /// Internal state for the Debug Stick, tracking property toggles.
    DebugStickState(Compound),

    /// NBT data used to modify an entity when it is spawned from an item (Spawn
    /// Eggs).
    EntityData { id: RegistryId, data: Compound },

    /// NBT data for entities inside a Bucket (like Fish or Axolotls).
    BucketEntityData(Compound),

    /// NBT data for the Block Entity created when this item is placed (Chests,
    /// Signs).
    BlockEntityData { id: RegistryId, data: Compound },

    /// The specific sound and duration associated with a Goat Horn.
    Instrument(IdOr<InstrumentDefinition>),

    /// Marks an item as a valid material for the Armor Trim system.
    ProvidesTrimMaterial(IdOr<TrimMaterial>),

    /// The level of Bad Omen granted by an Ominous Bottle (0-4).
    OminousBottleAmplifier(VarInt),

    /// The Jukebox Song played when this item is inserted into a Jukebox.
    JukeboxPlayable(IdOr<JukeboxSong>),

    /// Banner patterns this item unlocks in the Loom.
    ProvidesBannerPatterns(IDSet),

    /// The recipe IDs that a Knowledge Book will teach the player. Sent as an
    /// NBT list of strings, since vanilla has no network codec for it.
    Recipes(Vec<Ident<String>>),

    /// Tracking data for a Compass pointing to a specific Lodestone.
    LodestoneTracker {
        /// The dimension and coordinate of the target. None if the compass is
        /// spinning.
        target: Option<LodestoneTarget>,
        /// If true, the compass becomes a normal compass if the lodestone is
        /// destroyed.
        tracked: bool,
    },

    /// Individual explosion properties for a Firework Star.
    FireworkExplosion(FireworkExplosionData),

    /// Flight and explosion data for a Firework Rocket.
    Fireworks {
        flight_duration: VarInt,
        explosions: Vec<FireworkExplosionData>,
    },

    /// Data for a Player Head, including the skin texture and UUID.
    Profile(ResolvableProfile),

    /// The sound played by a Note Block if this player head is placed on top of
    /// it.
    NoteBlockSound(String),

    /// Visual layers for a Banner or Shield.
    BannerPatterns(Vec<BannerLayer>),

    /// The base dye color for a Banner.
    BaseColor(VarInt),

    /// The four item IDs used as patterns on a Decorated Pot.
    PotDecorations(Vec<RegistryId>),

    /// The inventory contents of a block (like a Chest or Shulker Box).
    Container(Vec<Option<ItemStackTemplate>>),

    /// Block state property overrides (e.g., "lit: true").
    BlockState(Vec<(String, String)>),

    /// Data for bees currently inside a Beehive item.
    Bees(Vec<BeeData>),

    /// The block item absorbed by a sulfur cube.
    SulfurCubeContent(Box<ItemStackTemplate>),

    /// The item predicate a key must match to open a locked container, as an
    /// NBT compound. Vanilla sends it through the persistent codec.
    Lock(Compound),

    /// Reference to a Loot Table for an unopened chest.
    ContainerLoot(Compound),

    /// Overrides the default sound played when this specific item breaks.
    BreakSound(IdOr<SoundEventDefinition>),

    /// Biome-specific variant of a Villager (e.g., Desert, Plains).
    VillagerVariant(RegistryId),

    /// Skin variant for a Wolf.
    WolfVariant(RegistryId),

    /// Determines the bark/growl sounds for a Wolf.
    WolfSoundVariant(RegistryId),

    /// Dye color of a Wolf's collar.
    WolfCollar(DyeColor),

    /// Type of Fox (Red or Snow).
    FoxVariant(FoxType),

    /// Size of a Salmon (Small, Medium, Large).
    SalmonSize(SalmonScale),

    /// Color of a Parrot.
    ParrotVariant(ParrotType),

    /// Pattern type for a Tropical Fish.
    TropicalFishPattern(TropicalFishPattern),

    /// Primary color of a Tropical Fish.
    TropicalFishBaseColor(DyeColor),

    /// Secondary color of a Tropical Fish.
    TropicalFishPatternColor(DyeColor),

    /// Type of Mooshroom (Red or Brown).
    MooshroomVariant(MooshroomType),

    /// Breed of a Rabbit.
    RabbitVariant(RabbitType),

    /// Skin variant for a Pig.
    PigVariant(RegistryId),

    /// Skin variant for a Cow.
    CowVariant(RegistryId),

    /// Skin variant for a Chicken.
    ChickenVariant(RegistryId),

    /// Biome variant for a Frog.
    FrogVariant(RegistryId),

    /// Color and marking variant for a Horse.
    HorseVariant(HorseColor),

    /// The specific painting texture and dimensions.
    PaintingVariant(IdOr<PaintingVariantDefinition>),

    /// Color variant for a Llama.
    LlamaVariant(LlamaColor),

    /// Color variant for an Axolotl.
    AxolotlVariant(AxolotlType),

    /// Breed variant for a Cat.
    CatVariant(RegistryId),

    /// Dye color of a Cat's collar.
    CatCollar(DyeColor),

    /// Natural wool color of a Sheep.
    SheepColor(DyeColor),

    /// Shell color of a Shulker.
    ShulkerColor(DyeColor),

    PigSoundVariant(RegistryId),
    CowSoundVariant(RegistryId),
    ChickenSoundVariant(RegistryId),
    ZombieNautilusVariant(RegistryId),
    CatSoundVariant(RegistryId),
}

impl ItemComponent {
    pub fn hash(&self) -> i32 {
        // TODO: implement if required
        0
    }
}

/// Movement and vibration behavior while an item is being used.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct UseEffects {
    pub can_sprint: bool,
    pub interact_vibrations: bool,
    pub speed_multiplier: f32,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct AttackRange {
    pub min_reach: f32,
    pub max_reach: f32,
    pub min_creative_reach: f32,
    pub max_creative_reach: f32,
    pub hitbox_margin: f32,
    pub mob_factor: f32,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct PiercingWeapon {
    pub deals_knockback: bool,
    pub dismounts: bool,
    pub sound: Option<IdOr<SoundEventDefinition>>,
    pub hit_sound: Option<IdOr<SoundEventDefinition>>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct KineticWeapon {
    pub contact_cooldown_ticks: VarInt,
    pub delay_ticks: VarInt,
    pub dismount_conditions: Option<KineticWeaponCondition>,
    pub knockback_conditions: Option<KineticWeaponCondition>,
    pub damage_conditions: Option<KineticWeaponCondition>,
    pub forward_movement: f32,
    pub damage_multiplier: f32,
    pub sound: Option<IdOr<SoundEventDefinition>>,
    pub hit_sound: Option<IdOr<SoundEventDefinition>>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct KineticWeaponCondition {
    pub max_duration_ticks: VarInt,
    pub min_speed: f32,
    pub min_relative_speed: f32,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct SwingAnimation {
    pub kind: SwingAnimationType,
    pub duration: VarInt,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum SwingAnimationType {
    None,
    Whack,
    Stab,
}

/// Defines a rule for matching a block in the world.
/// Used by `CanPlaceOn` and `CanBreak` in Adventure Mode.
#[derive(Clone, PartialEq, Debug)]
pub struct BlockPredicate {
    /// If None, matches any block ID.
    pub blocks: Option<IDSet>,

    /// Matches specific block state properties (e.g., `lit=true`).
    pub properties: Option<Vec<Property>>,

    /// Matches the Block Entity's NBT data.
    pub nbt: Option<Compound>,

    /// (1.21+) Matches if the block drops an item containing these EXACT
    /// components. This is a strict equality check.
    pub exact_components: Vec<ExactComponentMatcher>,

    /// (1.21+) Matches if the block drops an item containing specific NBT
    /// structures within specific components (at most 64).
    pub partial_components: Vec<PartialComponentMatcher>,
}

impl Encode for BlockPredicate {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.blocks.encode(&mut w)?;
        self.properties.encode(&mut w)?;
        self.nbt.encode(&mut w)?;
        self.exact_components.encode(&mut w)?;
        Bounded::<_, 64>(self.partial_components.as_slice()).encode(w)
    }
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
    /// An inclusive range; a missing bound is unbounded.
    MinMax {
        min: Option<String>,
        max: Option<String>,
    },
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
            let min = Option::<String>::decode(r)?;
            let max = Option::<String>::decode(r)?;
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

/// What a [`PartialComponentMatcher`] checks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComponentPredicateType {
    /// A data component predicate type (e.g. `minecraft:damage`), by its ID in
    /// the `data_component_predicate_type` registry.
    Predicate(RegistryId),
    /// Matches any value of the item component with this ID. The predicate
    /// value is then an empty compound.
    AnyValue(VarInt),
}

impl Encode for ComponentPredicateType {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        match *self {
            Self::Predicate(id) => Either::<RegistryId, VarInt>::Left(id).encode(w),
            Self::AnyValue(id) => Either::<RegistryId, VarInt>::Right(id).encode(w),
        }
    }
}

impl<'a> Decode<'a> for ComponentPredicateType {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        Ok(match Either::<RegistryId, VarInt>::decode(r)? {
            Either::Left(id) => Self::Predicate(id),
            Either::Right(id) => Self::AnyValue(id),
        })
    }
}

/// Matches a subset of data within a component using NBT.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct PartialComponentMatcher {
    /// The predicate type to apply.
    pub predicate_type: ComponentPredicateType,
    /// The predicate's value, sent as an NBT tag of any type.
    pub predicate: Value,
}

/// Modifies a player's attributes (like Strength or Speed).
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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
    /// How this modifier is displayed in the item's tooltip.
    pub display: AttributeModifierDisplay,
}

#[derive(Clone, PartialEq, Debug, Default, Encode, Decode)]
pub enum AttributeModifierDisplay {
    #[default]
    Default,
    Hidden,
    Override(TextComponent),
}

/// Defines custom mining speed logic for a tool.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct ToolRule {
    /// The blocks this rule applies to.
    pub blocks: IDSet,

    /// If present, overrides the mining speed for these blocks.
    pub speed: Option<f32>,

    /// If present and true, this tool is considered "correct" for the block
    /// (meaning the block will drop items when broken).
    pub correct_drop_for_blocks: Option<bool>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LodestoneTarget {
    /// The namespaced key of the dimension (e.g., "`minecraft:the_nether`").
    pub dimension: String,

    /// The X, Y, Z coordinates of the Lodestone block. Sent as a packed
    /// block position, so X and Z must fit in 26 bits and Y in 12 bits.
    pub position: (i32, i32, i32),
}

impl Encode for LodestoneTarget {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        self.dimension.encode(&mut w)?;
        pack_block_pos(self.position)?.encode(w)
    }
}

impl<'a> Decode<'a> for LodestoneTarget {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        Ok(Self {
            dimension: String::decode(r)?,
            position: unpack_block_pos(i64::decode(r)?),
        })
    }
}

const PACKED_HORIZONTAL_BITS: u32 = 26;
const PACKED_Y_BITS: u32 = 12;

/// Packs a block position like vanilla's `BlockPos.asLong`: X in the top 26
/// bits, then Z in 26 bits, then Y in the low 12 bits. `chunkedge_item` cannot
/// depend on `chunkedge_protocol`'s `BlockPos`.
fn pack_block_pos((x, y, z): (i32, i32, i32)) -> anyhow::Result<i64> {
    const H: i32 = 1 << (PACKED_HORIZONTAL_BITS - 1);
    const V: i32 = 1 << (PACKED_Y_BITS - 1);
    anyhow::ensure!(
        (-H..H).contains(&x) && (-V..V).contains(&y) && (-H..H).contains(&z),
        "block position ({x}, {y}, {z}) is out of range"
    );
    let horizontal_mask = (1_i64 << PACKED_HORIZONTAL_BITS) - 1;
    let y_mask = (1_i64 << PACKED_Y_BITS) - 1;
    Ok(
        ((i64::from(x) & horizontal_mask) << (PACKED_Y_BITS + PACKED_HORIZONTAL_BITS))
            | ((i64::from(z) & horizontal_mask) << PACKED_Y_BITS)
            | (i64::from(y) & y_mask),
    )
}

/// Unpacks a block position packed by [`pack_block_pos`], sign-extending each
/// coordinate.
fn unpack_block_pos(packed: i64) -> (i32, i32, i32) {
    let x = packed >> (PACKED_Y_BITS + PACKED_HORIZONTAL_BITS);
    let z =
        (packed << (64 - PACKED_Y_BITS - PACKED_HORIZONTAL_BITS)) >> (64 - PACKED_HORIZONTAL_BITS);
    let y = (packed << (64 - PACKED_Y_BITS)) >> (64 - PACKED_Y_BITS);
    (x as i32, y as i32, z as i32)
}

/// Defines a sound event, either by referencing the registry or defining it
/// on the fly.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct SoundEventDefinition {
    /// The identifier of the sound (e.g., "minecraft:entity.pig.ambient").
    pub sound: Ident<String>,

    /// A fixed range (in blocks) for the sound. If None, uses the default.
    pub range: Option<f32>,
}

/// Defines a material used to trim armor (e.g., Gold, Amethyst).
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct TrimPattern {
    /// The asset ID for the texture pattern.
    pub asset_id: String,

    /// The text displayed in the tooltip (e.g., "Vex Armor Trim").
    pub description: TextComponent,

    /// If true, the pattern is applied as a "Decal" (no color blending).
    pub decal: bool,
}

/// Defines a Goat Horn instrument.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct JukeboxSong {
    /// The sound event to play.
    pub sound_event: IdOr<SoundEventDefinition>,

    /// The song title shown in the "Now Playing" action bar.
    pub description: TextComponent,

    /// The duration of the song in seconds.
    pub length_seconds: f32,

    /// The Redstone signal strength (0-15) emitted by the Jukebox while
    /// playing.
    pub comparator_output: VarInt,
}

/// Defines a variant of a painting entity.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct PaintingVariantDefinition {
    /// Width in blocks (1 block = 16 pixels).
    pub width: VarInt,

    /// Height in blocks.
    pub height: VarInt,

    /// The path to the texture in the resource pack.
    pub asset_id: String,

    /// The painting's title shown in the tooltip.
    pub title: Option<TextComponent>,

    /// The painting's author shown in the tooltip.
    pub author: Option<TextComponent>,
}

/// Defines a single explosion in a Firework Rocket.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct BannerLayer {
    /// The pattern type (Flower, Skull, Stripe, etc.).
    pub pattern: IdOr<BannerPattern>,

    /// The dye color ID (0-15) for this layer.
    pub color: DyeColor,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct BannerPattern {
    /// The texture identifier (e.g., "minecraft:flower").
    pub asset_id: String,

    /// The translation key for the pattern name (e.g.,
    /// "block.minecraft.banner.flower").
    pub translation_key: String,
}

/// A page in a Book and Quill (Writable).
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct WritablePage {
    /// The raw text entered by the player.
    pub raw: String,

    /// If the server runs a chat filter, this is the filtered version.
    /// If None, the raw text is considered safe to display.
    pub filtered: Option<String>,
}

/// A page in a Finished Book (Written).
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct WrittenPage {
    /// The JSON text component for the page content.
    pub raw: TextComponent,

    /// Optional filtered version for chat safety settings.
    pub filtered: Option<TextComponent>,
}

/// Represents a Player's Game Profile (Skin/UUID).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ResolvableProfile {
    /// The player's username.
    pub name: Option<String>,

    /// The player's UUID.
    pub id: Option<Uuid>,

    /// Properties, primarily the "textures" property containing the skin URL.
    pub properties: Vec<ProfileProperty>,
    /// Optional client texture and model overrides.
    pub skin_patch: PlayerSkinPatch,
}

/// Optional resource textures and player-model selection for a profile.
#[derive(Clone, PartialEq, Debug, Default, Encode, Decode)]
pub struct PlayerSkinPatch {
    pub texture: Option<Ident<String>>,
    pub cape: Option<Ident<String>>,
    pub elytra: Option<Ident<String>>,
    /// `true` selects the slim model; `false` selects the wide model.
    pub slim: Option<bool>,
}

impl Encode for ResolvableProfile {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        // The right branch contains optional profile fields; the left contains a full profile.
        false.encode(&mut w)?;
        self.name.encode(&mut w)?;
        self.id.encode(&mut w)?;
        self.properties.encode(&mut w)?;
        self.skin_patch.encode(w)
    }
}

impl<'a> Decode<'a> for ResolvableProfile {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        let (name, id) = if bool::decode(r)? {
            let id = Uuid::decode(r)?;
            (Some(String::decode(r)?), Some(id))
        } else {
            (Option::<String>::decode(r)?, Option::<Uuid>::decode(r)?)
        };
        Ok(Self {
            name,
            id,
            properties: Vec::decode(r)?,
            skin_patch: PlayerSkinPatch::decode(r)?,
        })
    }
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct ProfileProperty {
    pub name: String,
    /// The base64 encoded value.
    pub value: String,
    /// The optional public key signature (Yggdrasil).
    pub signature: Option<String>,
}

/// Information about a Bee inside a Beehive.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct BeeData {
    /// The entity type of the occupant, by its `entity_type` registry ID.
    pub entity_type: RegistryId,

    /// The NBT data of the Bee entity itself (Health, Name, etc.), without
    /// its `id` field.
    pub entity_data: Compound,

    /// How many ticks the bee has been inside the hive.
    pub ticks_in_hive: VarInt,

    /// The minimum ticks required before the bee can leave.
    pub min_ticks_in_hive: VarInt,
}

/// An effect caused by consuming an item. The discriminant is the effect
/// type's ID in the `consume_effect_type` registry.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum ConsumeEffect {
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
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct PotionEffect {
    /// The ID of the effect (Speed, Jump Boost, etc.).
    pub id: RegistryId,

    /// The strength, duration and visibility of the effect.
    pub details: PotionEffectDetails,
}

/// The parameters of a [`PotionEffect`], matching vanilla's
/// `MobEffectInstance.Details`.
#[derive(Clone, PartialEq, Debug)]
pub struct PotionEffectDetails {
    /// The level of the effect (0 = Level 1, 1 = Level 2).
    pub amplifier: VarInt,

    /// Duration in ticks. -1 indicates infinite duration.
    pub duration: VarInt,

    /// If true, particles are translucent (like Beacon effects).
    pub ambient: bool,

    /// If false, no particles are shown.
    pub show_particles: bool,

    /// If true, the effect icon is displayed in the top-right of the
    /// inventory.
    pub show_icon: bool,

    /// A weaker, longer effect of the same type that resumes once this one
    /// expires.
    pub hidden_effect: Option<Box<PotionEffectDetails>>,
}

impl PotionEffectDetails {
    fn decode_recursive(r: &mut &[u8], depth: usize) -> anyhow::Result<Self> {
        anyhow::ensure!(
            depth <= crate::MAX_RECURSION_DEPTH,
            "potion effect recursion limit exceeded"
        );
        Ok(Self {
            amplifier: VarInt::decode(r)?,
            duration: VarInt::decode(r)?,
            ambient: bool::decode(r)?,
            show_particles: bool::decode(r)?,
            show_icon: bool::decode(r)?,
            hidden_effect: if bool::decode(r)? {
                Some(Box::new(Self::decode_recursive(r, depth + 1)?))
            } else {
                None
            },
        })
    }
}

impl Decode<'_> for PotionEffectDetails {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        Self::decode_recursive(r, 0)
    }
}

impl Encode for PotionEffectDetails {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        // Erase the writer type so the recursive hidden effect does not
        // instantiate an unbounded chain of `&mut` writers.
        let w: &mut dyn Write = &mut w;
        self.amplifier.encode(&mut *w)?;
        self.duration.encode(&mut *w)?;
        self.ambient.encode(&mut *w)?;
        self.show_particles.encode(&mut *w)?;
        self.show_icon.encode(&mut *w)?;
        self.hidden_effect.encode(w)
    }
}

/// Shield logic for reducing damage.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum Rarity {
    Common,   // White
    Uncommon, // Yellow
    Rare,     // Aqua
    Epic,     // Purple
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum MapPostProcessingType {
    Lock,  // The map has been locked in a Cartography Table.
    Scale, // The map is being zoomed out.
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum ConsumableAnimation {
    None,
    Eat,
    Drink,
    Block, // Shield block animation
    Bow,
    Trident,
    Crossbow,
    Spyglass,
    TootHorn,
    Brush,
    Bundle,
    Spear,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum EquipSlot {
    MainHand,
    Feet,
    Legs,
    Chest,
    Head,
    OffHand,
    Body, // Horse armor / Llama carpet
    Saddle,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
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
    Saddle,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
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

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum FoxType {
    Red,
    Snow,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum SalmonScale {
    Small,
    Medium,
    Large,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum ParrotType {
    RedBlue,
    Blue,
    Green,
    YellowBlue,
    Gray,
}

/// Sent as a packed ID: the base shape (0 = small, 1 = large) in the low byte
/// and the pattern index within that shape above it.
#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum TropicalFishPattern {
    #[packet(tag = 0)]
    Kob,
    #[packet(tag = 0x100)]
    Sunstreak,
    #[packet(tag = 0x200)]
    Snooper,
    #[packet(tag = 0x300)]
    Dasher,
    #[packet(tag = 0x400)]
    Brinely,
    #[packet(tag = 0x500)]
    Spotty,
    #[packet(tag = 1)]
    Flopper,
    #[packet(tag = 0x101)]
    Stripey,
    #[packet(tag = 0x201)]
    Glitter,
    #[packet(tag = 0x301)]
    Blockfish,
    #[packet(tag = 0x401)]
    Betty,
    #[packet(tag = 0x501)]
    Clayfish,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum MooshroomType {
    Red,
    Brown,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum RabbitType {
    Brown,
    White,
    Black,
    WhiteSplotched,
    Gold,
    Salt,
    #[packet(tag = 99)]
    Evil, // "Toast"
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum HorseColor {
    White,
    Creamy,
    Chestnut,
    Brown,
    Black,
    Gray,
    DarkBrown,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum LlamaColor {
    Creamy,
    White,
    Brown,
    Gray,
}

#[derive(Clone, Copy, PartialEq, Debug, Encode, Decode)]
pub enum AxolotlType {
    Lucy, // Pink
    Wild, // Brown
    Gold,
    Cyan,
    Blue,
}
