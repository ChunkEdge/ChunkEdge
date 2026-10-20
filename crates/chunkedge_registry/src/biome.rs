//! Contains biomes and the biome registry. Minecraft's default biomes are added
//! to the registry by default.
//!
//! ### **NOTE:**
//! - Modifying the biome registry after the server has started can break
//!   invariants within instances and clients! Make sure there are no instances
//!   or clients spawned before mutating.
//! - A biome named "minecraft:plains" must exist. Otherwise, vanilla clients
//!   will be disconnected.

use std::ops::{Deref, DerefMut};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_ident::{Ident, ident};
use chunkedge_nbt::Compound;
use chunkedge_nbt::serde::ser::CompoundSerializer;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::error;

use crate::codec::{RegistryCodec, RegistryValue};
use crate::{Registry, RegistryIdx, RegistrySet};

pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BiomeRegistry>()
            .add_systems(PreStartup, load_default_biomes)
            .add_systems(PostUpdate, update_biome_registry.before(RegistrySet));
    }
}

fn load_default_biomes(mut reg: ResMut<BiomeRegistry>, codec: Res<RegistryCodec>) {
    let mut helper = move || -> anyhow::Result<()> {
        for value in codec.registry(BiomeRegistry::KEY) {
            let biome = Biome::deserialize(value.element.clone())?;

            reg.insert(value.name.clone(), biome);
        }

        // Move "plains" to the front so that `BiomeId::default()` is the ID of plains.
        reg.swap_to_front(ident!("plains"));

        Ok(())
    };

    if let Err(e) = helper() {
        error!("failed to load default biomes from registry codec: {e:#}");
    }
}

fn update_biome_registry(reg: Res<BiomeRegistry>, mut codec: ResMut<RegistryCodec>) {
    if reg.is_changed() {
        let biomes = codec.registry_mut(BiomeRegistry::KEY);

        biomes.clear();

        biomes.extend(reg.iter().map(|(_, name, biome)| {
            RegistryValue {
                name: name.into(),
                element: biome
                    .serialize(CompoundSerializer)
                    .expect("failed to serialize biome"),
            }
        }));
    }
}

#[derive(Resource, Default, Debug)]
pub struct BiomeRegistry {
    reg: Registry<BiomeId, Biome>,
}

impl BiomeRegistry {
    pub const KEY: Ident<&'static str> = ident!("worldgen/biome");
}

impl Deref for BiomeRegistry {
    type Target = Registry<BiomeId, Biome>;

    fn deref(&self) -> &Self::Target {
        &self.reg
    }
}

impl DerefMut for BiomeRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reg
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct BiomeId(u32);

impl BiomeId {
    pub const DEFAULT: Self = BiomeId(0);
}

impl RegistryIdx for BiomeId {
    const MAX: usize = u32::MAX as usize;

    #[inline]
    fn to_index(self) -> usize {
        self.0 as usize
    }

    #[inline]
    fn from_index(idx: usize) -> Self {
        Self(idx as u32)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Biome {
    pub has_precipitation: bool,
    pub temperature: f32,
    pub downfall: f32,
    /// Environment attributes (e.g. `minecraft:visual/sky_color`, moved out
    /// of `effects` in 26.1). Forwarded opaquely: without these the client
    /// falls back to defaults such as a black sky.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Compound>,
    pub effects: BiomeEffects,
}

impl Default for Biome {
    /// Default will be the same as the `minecraft:plains` biome.
    fn default() -> Self {
        Self {
            has_precipitation: true,
            temperature: 0.8,
            downfall: 0.4,
            attributes: None,
            effects: BiomeEffects::default(),
        }
    }
}

/// Deserializes a color that is either a packed `u32` (legacy) or a
/// `"#rrggbb"` hex string (26.1+).
fn deserialize_color<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrInt {
        Int(u32),
        Str(String),
    }

    let helper = Option::<StringOrInt>::deserialize(deserializer)?;
    match helper {
        None => Ok(None),
        Some(StringOrInt::Int(i)) => Ok(Some(i)),
        Some(StringOrInt::Str(s)) => {
            let hex = s.strip_prefix('#').unwrap_or(&s);
            u32::from_str_radix(hex, 16)
                .map(Some)
                .map_err(serde::de::Error::custom)
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
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub sky_color: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub foliage_color: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub grass_color: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub fog_color: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub water_color: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub water_fog_color: Option<u32>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        deserialize_with = "deserialize_color"
    )]
    pub dry_foliage_color: Option<u32>,
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
            sky_color: Some(7907327),
            fog_color: Some(12638463),
            water_color: Some(4159204),
            water_fog_color: Some(329011),
            dry_foliage_color: None,
            additions_sound: None,
            music: Vec::new(),
            particle: None,
            foliage_color: None,
            grass_color: None,
            grass_color_modifier: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use chunkedge_nbt::Value;
    use chunkedge_nbt::serde::ser::CompoundSerializer;

    use super::*;

    /// Same as the dimension-type case: 26.x biome `attributes` (sky color,
    /// ...) must survive the round-trip instead of being dropped.
    #[test]
    fn biome_attributes_round_trip() {
        let codec = include_bytes!("../extracted/registry_codec.json");
        let compound = serde_json::from_slice::<Compound>(codec).expect("valid registry codec");

        let Value::Compound(biomes) = compound
            .get("minecraft:worldgen/biome")
            .expect("biome registry")
            .clone()
        else {
            panic!("expected compound");
        };
        let Value::Compound(plains) = biomes
            .get("minecraft:plains")
            .expect("plains entry")
            .clone()
        else {
            panic!("expected compound");
        };

        let biome = Biome::deserialize(plains).expect("plains deserializes");
        assert!(biome.attributes.is_some());

        let reserialized = biome.serialize(CompoundSerializer).expect("serializes");
        let Some(Value::Compound(attrs)) = reserialized.get("attributes") else {
            panic!("attributes dropped from biome");
        };
        assert!(attrs.contains_key("minecraft:visual/sky_color"));
    }
}
