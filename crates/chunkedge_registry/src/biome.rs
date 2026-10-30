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
use std::sync::LazyLock;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_ident::{Ident, ident};
use chunkedge_nbt::Compound;
use chunkedge_nbt::serde::ser::CompoundSerializer;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
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
                    .expect("failed to serialize biome")
                    .into(),
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
    #[serde(default, skip_serializing_if = "TemperatureModifier::is_none")]
    pub temperature_modifier: TemperatureModifier,
    pub downfall: f32,
    /// Environment attributes controlling sky/fog colors, music, ambient sounds,
    /// and particles. Color attributes such as `minecraft:visual/sky_color`
    /// accept `"#rrggbb"` strings. Values use vanilla's environment-attribute codecs.
    #[serde(default, skip_serializing_if = "Compound::is_empty")]
    pub attributes: Compound,
    pub effects: BiomeEffects,
}

/// Vanilla `Biome.TemperatureModifier`.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TemperatureModifier {
    #[default]
    None,
    /// Used by frozen oceans to add warmer patches.
    Frozen,
}

impl TemperatureModifier {
    #[expect(
        clippy::trivially_copy_pass_by_ref,
        reason = "serde skip_serializing_if passes a reference"
    )]
    fn is_none(&self) -> bool {
        *self == Self::None
    }
}

impl Default for Biome {
    /// Default will be the same as the `minecraft:plains` biome.
    fn default() -> Self {
        static DEFAULT: LazyLock<Biome> = LazyLock::new(|| {
            let codec = RegistryCodec::default();
            let plains = codec
                .registry(BiomeRegistry::KEY)
                .iter()
                .find(|value| value.name.as_str() == "minecraft:plains")
                .expect("extracted plains biome");
            Biome::deserialize(plains.element.clone()).expect("valid extracted plains biome")
        });
        DEFAULT.clone()
    }
}

/// Deserializes a color from a packed `u32` or a `"#rrggbb"` hex string.
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

/// Serializes a color as a `"#rrggbb"` hex string like vanilla `STRING_RGB_COLOR`.
/// The client reads that form as an opaque color, but it reads a plain int as is,
/// so an RGB int would arrive with alpha 0. Bits above the low 24 are ignored.
#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde serialize_with passes a reference"
)]
fn serialize_color<S>(color: &Option<u32>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match color {
        Some(color) => serialize_water_color(color, serializer),
        None => serializer.serialize_none(),
    }
}

#[expect(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde serialize_with passes a reference"
)]
fn serialize_water_color<S>(color: &u32, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("#{:06x}", color & 0xff_ffff))
}

/// Biome colors separate from environment attributes
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct BiomeEffects {
    #[serde(
        deserialize_with = "deserialize_water_color",
        serialize_with = "serialize_water_color"
    )]
    pub water_color: u32,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub foliage_color: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub grass_color: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    pub dry_foliage_color: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grass_color_modifier: Option<String>,
}

fn deserialize_water_color<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    deserialize_color(deserializer)?.ok_or_else(|| serde::de::Error::custom("missing water color"))
}

impl Default for BiomeEffects {
    /// Default colors of the `minecraft:plains` biome.
    fn default() -> Self {
        Self {
            water_color: 4159204,
            foliage_color: None,
            grass_color: None,
            dry_foliage_color: None,
            grass_color_modifier: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use chunkedge_nbt::Value;
    use chunkedge_nbt::serde::ser::CompoundSerializer;

    use super::*;

    /// Biome environment attributes must survive deserialization and serialization
    /// so clients receive the configured sky color and other visual settings.
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
        assert!(biome.attributes.contains_key("minecraft:visual/sky_color"));

        let reserialized = biome.serialize(CompoundSerializer).expect("serializes");
        let Some(Value::Compound(attrs)) = reserialized.get("attributes") else {
            panic!("attributes dropped from biome");
        };
        assert!(attrs.contains_key("minecraft:visual/sky_color"));
    }

    /// Reserializing the extracted biomes must give back the `temperature_modifier`
    /// and the hex string effect colors vanilla sends. Numbers are left out: the
    /// JSON codec loads them as doubles.
    #[test]
    fn extracted_biomes_round_trip() {
        let codec = RegistryCodec::default();

        for value in codec.registry(BiomeRegistry::KEY) {
            let biome = Biome::deserialize(value.element.clone()).expect("biome deserializes");
            let reserialized = biome.serialize(CompoundSerializer).expect("serializes");

            for key in ["temperature_modifier", "effects"] {
                assert_eq!(
                    reserialized.get(key),
                    match &value.element {
                        chunkedge_nbt::Value::Compound(element) => element.get(key),
                        _ => panic!("biome must be a compound"),
                    },
                    "{key} of {} changed",
                    value.name
                );
            }
        }

        let frozen_ocean = codec
            .registry(BiomeRegistry::KEY)
            .iter()
            .find(|value| value.name.as_str() == "minecraft:frozen_ocean")
            .expect("frozen ocean");
        let biome = Biome::deserialize(frozen_ocean.element.clone()).unwrap();
        assert_eq!(biome.temperature_modifier, TemperatureModifier::Frozen);
        assert_eq!(biome.effects.water_color, 0x3938c9);
    }
}
