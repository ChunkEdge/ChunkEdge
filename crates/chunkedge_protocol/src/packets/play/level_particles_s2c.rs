use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_generated::block::BlockState;
use chunkedge_item::ItemStack;
use chunkedge_math::{DVec3, Vec3};

use crate::{BlockPos, Packet};

#[derive(Clone, Debug, Packet, Encode, Decode)]
pub struct LevelParticlesS2c {
    pub long_distance: bool,
    pub always_visible: bool,
    pub position: DVec3,
    pub offset: Vec3,
    pub max_speed: f32,
    pub count: i32,
    pub particle: Particle,
}

// The format of the enum below currently needs to be manually maintained.
// This means that when we update MC versions, we need to check if the
// particle types below are still correct. Either with the wiki:
// https://minecraft.wiki/w/Java_Edition_protocol/Particles
// Or with the MC source code.
#[derive(Clone, PartialEq, Debug, Encode, Decode, chunkedge_protocol_macros::RegistryId)]
#[packet(registry = chunkedge_generated::particle_type)]
pub enum Particle {
    AngryVillager,
    Block(BlockState),
    BlockMarker(BlockState),
    Bubble,
    Cloud,
    Crit,
    DamageIndicator,
    DragonBreath,
    DrippingLava,
    FallingLava,
    LandingLava,
    DrippingWater,
    FallingWater,
    Dust {
        /// The color, encoded as 0xRRGGBB; top bits are ignored.
        color: i32,
        //// The scale, will be clamped between 0.01 and 4.
        scale: f32,
    },
    DustColorTransition {
        /// The color to transition from, encoded as 0xRRGGBB; top bits are
        /// ignored.
        from_color: i32,
        /// The color to transition from, encoded as 0xRRGGBB; top bits are
        /// ignored.
        to_color: i32,
        /// The scale, will be clamped between 0.01 and 4.
        scale: f32,
    },
    Effect,
    ElderGuardian,
    EnchantedHit,
    Enchant,
    EndRod,
    EntityEffect {
        /// The ARGB components of the color encoded as an Int
        color: i32,
    },
    ExplosionEmitter,
    Explosion,
    Gust,
    SmallGust,
    GustEmitterLarge,
    GustEmitterSmall,
    SonicBoom,
    FallingDust(BlockState),
    Firework,
    Fishing,
    Flame,
    Infested,
    CherryLeaves,
    PaleOakLeaves,
    TintedLeaves {
        /// The ARGB components of the color encoded as an Int
        color: i32,
    },
    SculkSoul,
    SculkCharge {
        /// How much the particle will be rotated when displayed.
        roll: f32,
    },
    SculkChargePop,
    SoulFireFlame,
    Soul,
    Flash,
    HappyVillager,
    Composter,
    Heart,
    InstantEffect,
    /// If the item is air or its count is 0, the client might crash.
    Item(Box<ItemStack>),
    Vibration {
        /// The type of the vibration source defined by the
        /// `minecraft:position_source_type` builtin registry.
        source: VibrationSourceType,
        /// The amount of ticks it takes for the vibration to travel from its
        /// source to its destination.
        ticks: VarInt,
    },
    Trail {
        position: DVec3,
        /// The trail color, encoded as 0xRRGGBB; top bits are ignored.
        color: i32,
        /// Life time in ticks
        duration: VarInt,
    },
    ItemSlime,
    ItemCobweb,
    ItemSnowball,
    LargeSmoke,
    Lava,
    Mycelium,
    Note,
    Poof,
    Portal,
    Rain,
    Smoke,
    WhiteSmoke,
    Sneeze,
    Spit,
    SquidInk,
    SweepAttack,
    TotemOfUndying,
    Underwater,
    Splash,
    Witch,
    BubblePop,
    CurrentDown,
    BubbleColumnUp,
    Nautilus,
    Dolphin,
    CampfireCosySmoke,
    CampfireSignalSmoke,
    DrippingHoney,
    FallingHoney,
    LandingHoney,
    FallingNectar,
    FallingSporeBlossom,
    Ash,
    CrimsonSpore,
    WarpedSpore,
    SporeBlossomAir,
    DrippingObsidianTear,
    FallingObsidianTear,
    LandingObsidianTear,
    ReversePortal,
    WhiteAsh,
    SmallFlame,
    Snowflake,
    DrippingDripstoneLava,
    FallingDripstoneLava,
    DrippingDripstoneWater,
    FallingDripstoneWater,
    GlowSquidInk,
    Glow,
    WaxOn,
    WaxOff,
    ElectricSpark,
    Scrape,
    Shriek {
        /// The time in ticks before the particle is displayed
        delay: VarInt,
    },
    EggCrack,
    DustPlume,
    TrialSpawnerDetection,
    TrialSpawnerDetectionOminous,
    VaultConnection,
    DustPillar(BlockState),
    OminousSpawning,
    RaidOmen,
    TrialOmen,
    BlockCrumble(BlockState),
    Firefly,
    SulfurBubbles,
    NoxiousGas,
    NoxiousGasCloud,
    Geyser {
        water_blocks: i32,
    },
    GeyserBase {
        water_blocks: i32,
        burst_impulse_base: f32,
    },
    GeyserPoof {
        water_blocks: i32,
        burst_impulse_base: f32,
    },
    GeyserPlume {
        water_blocks: i32,
    },
    CopperFireFlame,
    PauseMobGrowth,
    ResetMobGrowth,
    SulfurCubeGoo,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum VibrationSourceType {
    Block {
        /// The position of the block the vibration originated from.
        block_pos: BlockPos,
    },
    Entity {
        /// The ID of the entity the vibration originated from.
        ///
        /// See `EntityId`.
        id: VarInt,
        /// Entity eye height.
        eye_height: f32,
    },
}
