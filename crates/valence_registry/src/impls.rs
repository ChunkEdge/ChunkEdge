use std::collections::BTreeMap;

use bevy_app::App;
use serde::{Deserialize, Serialize};
use valence_binary::id_set::IDSet;
use valence_ident::{ident, Ident};
use valence_nbt::Compound;

use crate::{RegistryItem, RegistryManagerPlugin};

pub fn add_registry_plugins(app: &mut App) {
    app.add_plugins((
        BannerPatternRegistryPlugin::default(),
        BiomeRegistryPlugin::default(),
        CatVariantRegistryPlugin::default(),
        ChatTypeRegistryPlugin::default(),
        ChickenVariantRegistryPlugin::default(),
        CowVariantRegistryPlugin::default(),
        DamageTypeRegistryPlugin::default(),
        DimensionTypeRegistryPlugin::default(),
        EnchantmentRegistryPlugin::default(),
        FrogVariantRegistryPlugin::default(),
        InstrumentRegistryPlugin::default(),
        JukeboxSongRegistryPlugin::default(),
        PaintingVariantRegistryPlugin::default(),
        PigVariantRegistryPlugin::default(),
        TestEnvironmentRegistryPlugin::default(),
        TestInstanceRegistryPlugin::default(),
        TrimMaterialRegistryPlugin::default(),
        TrimPatternRegistryPlugin::default(),
        WolfSoundVariantRegistryPlugin::default(),
        WolfVariantRegistryPlugin::default(),
    ));
}

// Type aliases for the plugins
pub type BannerPatternRegistryPlugin = RegistryManagerPlugin<BannerPattern>;
pub type BiomeRegistryPlugin = RegistryManagerPlugin<Biome>;
pub type CatVariantRegistryPlugin = RegistryManagerPlugin<CatVariant>;
pub type ChatTypeRegistryPlugin = RegistryManagerPlugin<ChatType>;
pub type ChickenVariantRegistryPlugin = RegistryManagerPlugin<ChickenVariant>;
pub type CowVariantRegistryPlugin = RegistryManagerPlugin<CowVariant>;
pub type DamageTypeRegistryPlugin = RegistryManagerPlugin<DamageType>;
pub type DimensionTypeRegistryPlugin = RegistryManagerPlugin<DimensionType>;
pub type EnchantmentRegistryPlugin = RegistryManagerPlugin<Enchantment>;
pub type FrogVariantRegistryPlugin = RegistryManagerPlugin<FrogVariant>;
pub type InstrumentRegistryPlugin = RegistryManagerPlugin<Instrument>;
pub type JukeboxSongRegistryPlugin = RegistryManagerPlugin<JukeboxSong>;
pub type PaintingVariantRegistryPlugin = RegistryManagerPlugin<PaintingVariant>;
pub type PigVariantRegistryPlugin = RegistryManagerPlugin<PigVariant>;
pub type TestEnvironmentRegistryPlugin = RegistryManagerPlugin<TestEnvironment>;
pub type TestInstanceRegistryPlugin = RegistryManagerPlugin<TestInstance>;
pub type TrimMaterialRegistryPlugin = RegistryManagerPlugin<TrimMaterial>;
pub type TrimPatternRegistryPlugin = RegistryManagerPlugin<TrimPattern>;
pub type WolfSoundVariantRegistryPlugin = RegistryManagerPlugin<WolfSoundVariant>;
pub type WolfVariantRegistryPlugin = RegistryManagerPlugin<WolfVariant>;

// --- Struct Definitions ---

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BannerPattern {
    pub asset_id: Ident<String>,
    pub translation_key: String,
}

impl RegistryItem for BannerPattern {
    const KEY: Ident<&'static str> = ident!("banner_pattern");
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Biome {
    pub has_precipitation: bool,
    pub temperature: f32,
    pub downfall: f32,
    pub effects: BiomeEffects,
}

impl RegistryItem for Biome {
    const KEY: Ident<&'static str> = ident!("worldgen/biome");
}

impl Default for Biome {
    /// Default will be the same as the `minecraft:plains` biome.
    fn default() -> Self {
        Self {
            has_precipitation: true,
            temperature: 0.8,
            downfall: 0.4,
            effects: BiomeEffects::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeEffects {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mood_sound: Option<BiomeMoodSound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additions_sound: Option<BiomeAdditionsSound>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub music: Vec<BiomeMusic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub music_volume: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub particle: Option<BiomeParticle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sky_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foliage_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fog_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub water_fog_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color_modifier: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeMoodSound {
    pub sound: Ident<String>,
    pub tick_delay: u32,
    pub block_search_extent: u32,
    pub offset: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeMusic {
    pub data: BiomeMusicData,
    pub weight: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeMusicData {
    pub sound: Ident<String>,
    pub min_delay: u32,
    pub max_delay: u32,
    pub replace_current_music: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeAdditionsSound {
    pub sound: Ident<String>,
    pub tick_chance: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeParticle {
    pub options: BiomeParticleOptions,
    pub probability: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BiomeParticleOptions {
    #[serde(rename = "type")]
    pub kind: Ident<String>,
}

impl Default for BiomeEffects {
    /// Default will be the same as the `minecraft:plains` biome.
    fn default() -> Self {
        Self {
            mood_sound: Some(BiomeMoodSound {
                sound: ident!("minecraft:ambient.cave").into(),
                tick_delay: 6000,
                block_search_extent: 8,
                offset: 2.0,
            }),
            music_volume: Some(1.0),
            sky_color: Some(0x78A7FF),
            fog_color: Some(0xC0D8FF),
            water_color: Some(0x3F76E4),
            water_fog_color: Some(0x50533),
            additions_sound: None,
            music: Vec::new(),
            particle: None,
            foliage_color: None,
            grass_color: None,
            grass_color_modifier: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CatVariant {
    pub asset_id: Ident<String>,
}

impl RegistryItem for CatVariant {
    const KEY: Ident<&'static str> = ident!("cat_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatType {
    pub chat: ChatTypeDecoration,
    pub narration: ChatTypeDecoration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatTypeDecoration {
    pub translation_key: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<Compound>, // TODO: : handle correctly as TextStyle
}

impl RegistryItem for ChatType {
    const KEY: Ident<&'static str> = ident!("chat_type");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChickenVariant {
    pub asset_id: Ident<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ChickenModel>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ChickenModel {
    #[default]
    Normal,
    Cold,
}

impl RegistryItem for ChickenVariant {
    const KEY: Ident<&'static str> = ident!("chicken_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CowVariant {
    pub asset_id: Ident<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<CowModel>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum CowModel {
    #[default]
    Normal,
    Cold,
    Warm,
}

impl RegistryItem for CowVariant {
    const KEY: Ident<&'static str> = ident!("cow_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DamageType {
    pub message_id: String,
    pub scaling: DamageScaling,
    pub exhaustion: f32,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<DamageEffects>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_message_type: Option<DamageDeathMessageType>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DamageScaling {
    Never,
    Always,
    WhenCausedByLivingNonPlayer,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DamageEffects {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DamageDeathMessageType {
    #[default]
    Default,
    FallVariants,
    IntentionalGameDesign,
}

impl RegistryItem for DamageType {
    const KEY: Ident<&'static str> = ident!("damage_type");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct DimensionType {
    pub ambient_light: f32,
    pub bed_works: bool,
    pub coordinate_scale: f64,
    pub effects: DimensionEffects,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_time: Option<i32>,
    pub has_ceiling: bool,
    pub has_raids: bool,
    pub has_skylight: bool,
    pub height: i32,
    pub infiniburn: IdSet<BlockKind>,
    pub logical_height: i32,
    pub min_y: i32,
    pub monster_spawn_block_light_limit: i32,
    pub monster_spawn_light_level: MonsterSpawnLightLevel,
    pub natural: bool,
    pub piglin_safe: bool,
    pub respawn_anchor_works: bool,
    pub ultrawarm: bool,
}

impl Default for DimensionType {
    fn default() -> Self {
        Self {
            ambient_light: 0.0,
            bed_works: true,
            coordinate_scale: 1.0,
            effects: DimensionEffects::default(),
            fixed_time: None,
            has_ceiling: false,
            has_raids: true,
            has_skylight: true,
            height: 384,
            infiniburn: "#minecraft:infiniburn_overworld".into(),
            logical_height: 384,
            min_y: -64,
            monster_spawn_block_light_limit: 0,
            monster_spawn_light_level: MonsterSpawnLightLevel::Int(7),
            natural: true,
            piglin_safe: false,
            respawn_anchor_works: false,
            ultrawarm: false,
        }
    }
}

/// Determines what skybox/fog effects to use in dimensions.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum DimensionEffects {
    #[serde(rename = "minecraft:overworld")]
    #[default]
    Overworld,
    #[serde(rename = "minecraft:the_nether")]
    TheNether,
    #[serde(rename = "minecraft:the_end")]
    TheEnd,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MonsterSpawnLightLevel {
    Int(i32),
    Tagged(MonsterSpawnLightLevelTagged),
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonsterSpawnLightLevelTagged {
    #[serde(rename = "minecraft:uniform")]
    Uniform {
        min_inclusive: i32,
        max_inclusive: i32,
    },
}

impl From<i32> for MonsterSpawnLightLevel {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl RegistryItem for Enchantment {
    const KEY: Ident<&'static str> = ident!("enchantment");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FrogVariant {
    pub asset_id: Ident<String>,
}

impl RegistryItem for FrogVariant {
    const KEY: Ident<&'static str> = ident!("frog_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Instrument {
    pub sound_event: Ident<String>,
    pub use_duration: f32,
    pub range: f32,
    pub description: Compound, // Text component
}

impl RegistryItem for Instrument {
    const KEY: Ident<&'static str> = ident!("instrument");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JukeboxSong {
    pub sound_event: Ident<String>,
    pub description: Compound, // Text component
    pub length_in_seconds: f32,
    pub comparator_output: i32,
}

impl RegistryItem for JukeboxSong {
    const KEY: Ident<&'static str> = ident!("jukebox_song");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PaintingVariant {
    pub asset_id: Ident<String>,
    pub width: i32,
    pub height: i32,
    pub title: Compound, // Text component
    #[serde(default)]
    pub author: Option<Compound>, // Text component
}

impl RegistryItem for PaintingVariant {
    const KEY: Ident<&'static str> = ident!("painting_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PigVariant {
    pub asset_id: Ident<String>,
    #[serde(default)]
    pub model: Option<String>,
}

impl RegistryItem for PigVariant {
    const KEY: Ident<&'static str> = ident!("pig_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestEnvironment {
    // Structure depends on test framework
    #[serde(default)]
    pub definitions: Vec<Compound>,
}

impl RegistryItem for TestEnvironment {
    const KEY: Ident<&'static str> = ident!("test_environment");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestInstance {
    // Structure depends on test framework
    pub function: String,
    pub max_ticks: i32,
    pub setup_ticks: i32,
    pub required: bool,
    pub environment: String,
    pub structure: String,
}

impl RegistryItem for TestInstance {
    const KEY: Ident<&'static str> = ident!("test_instance");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrimMaterial {
    pub asset_name: String,
    pub description: Text,
    #[serde(default)]
    pub override_armor_assets: Option<BTreeMap<String, String>>,
}

impl RegistryItem for TrimMaterial {
    const KEY: Ident<&'static str> = ident!("trim_material");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrimPattern {
    pub asset_id: Ident<String>,
    pub description: Text,
    pub decal: bool,
    pub template_item: Ident<String>,
}

impl RegistryItem for TrimPattern {
    const KEY: Ident<&'static str> = ident!("trim_pattern");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WolfSoundVariant {
    pub hurt_sound: Ident<String>,
    pub pant_sound: Ident<String>,
    pub whine_sound: Ident<String>,
    pub ambient_sound: Ident<String>,
    pub death_sound: Ident<String>,
    pub growl_sound: Ident<String>,
}

impl RegistryItem for WolfSoundVariant {
    const KEY: Ident<&'static str> = ident!("wolf_sound_variant");
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WolfVariant {
    assets: WolfVariantAssets,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WolfVariantAssets {
    pub wild: String,
    pub tame: String,
    pub angry: String,
}

impl RegistryItem for WolfVariant {
    const KEY: Ident<&'static str> = ident!("wolf_variant");
}
