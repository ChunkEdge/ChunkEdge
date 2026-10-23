//! Contains dimension types and the dimension type registry. Minecraft's
//! default dimensions are added to the registry by default.
//!
//! ### **NOTE:**
//! - Modifying the dimension type registry after the server has started can
//!   break invariants within instances and clients! Make sure there are no
//!   instances or clients spawned before mutating.

use std::ops::{Deref, DerefMut};
use std::sync::LazyLock;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_ident::{Ident, ident};
use chunkedge_nbt::serde::ser::CompoundSerializer;
use chunkedge_nbt::{Compound, Value};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::codec::{RegistryCodec, RegistryValue};
use crate::{Registry, RegistryIdx, RegistrySet};
pub struct DimensionTypePlugin;

impl Plugin for DimensionTypePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DimensionTypeRegistry>()
            .add_systems(PreStartup, load_default_dimension_types)
            .add_systems(
                PostUpdate,
                update_dimension_type_registry.before(RegistrySet),
            );
    }
}

/// Loads the default dimension types from the registry codec.
fn load_default_dimension_types(mut reg: ResMut<DimensionTypeRegistry>, codec: Res<RegistryCodec>) {
    let mut helper = move || -> anyhow::Result<()> {
        for value in codec.registry(DimensionTypeRegistry::KEY) {
            let mut dimension_type = DimensionType::deserialize(value.element.clone())?;

            dimension_type.make_full_bright();

            reg.insert(value.name.clone(), dimension_type);
        }

        Ok(())
    };

    if let Err(e) = helper() {
        error!("failed to load default dimension types from registry codec: {e:#}");
    }
}

/// Updates the registry codec as the dimension type registry is modified by
/// users.
fn update_dimension_type_registry(
    reg: Res<DimensionTypeRegistry>,
    mut codec: ResMut<RegistryCodec>,
) {
    if reg.is_changed() {
        let dimension_types = codec.registry_mut(DimensionTypeRegistry::KEY);

        dimension_types.clear();

        dimension_types.extend(reg.iter().map(|(_, name, dim)| {
            RegistryValue {
                name: name.into(),
                element: dim
                    .serialize(CompoundSerializer)
                    .expect("failed to serialize dimension type"),
            }
        }));
    }
}

#[derive(Resource, Default, Debug)]
pub struct DimensionTypeRegistry {
    reg: Registry<DimensionTypeId, DimensionType>,
}

impl DimensionTypeRegistry {
    pub const KEY: Ident<&'static str> = ident!("dimension_type");
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
pub struct DimensionTypeId(u16);

impl DimensionTypeId {
    pub fn new(value: u16) -> Self {
        DimensionTypeId(value)
    }
    pub fn get_value(&self) -> u16 {
        self.0
    }
}

impl RegistryIdx for DimensionTypeId {
    const MAX: usize = u16::MAX as usize;

    fn to_index(self) -> usize {
        self.0 as usize
    }

    fn from_index(idx: usize) -> Self {
        Self(idx as u16)
    }
}

impl Deref for DimensionTypeRegistry {
    type Target = Registry<DimensionTypeId, DimensionType>;

    fn deref(&self) -> &Self::Target {
        &self.reg
    }
}

impl DerefMut for DimensionTypeRegistry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reg
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct DimensionType {
    pub ambient_light: f32,
    pub coordinate_scale: f64,
    #[serde(default)]
    pub has_fixed_time: bool,
    pub has_skylight: bool,
    pub has_ceiling: bool,
    #[serde(default)]
    pub has_ender_dragon_fight: bool,
    pub height: i32,
    pub infiniburn: String,
    pub logical_height: i32,
    pub min_y: i32,
    pub monster_spawn_block_light_limit: i32,
    pub monster_spawn_light_level: MonsterSpawnLightLevel,
    /// Environment attributes (sky/fog colors, light factors, music, ...).
    /// Forwarded opaquely so the client receives the dimension's visual and audio settings.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Compound>,
    /// Timelines (world clocks) reference, e.g. `"#minecraft:in_overworld"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timelines: Option<String>,
    /// Default world clock, e.g. `"minecraft:overworld"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_clock: Option<String>,
    /// Skybox override, e.g. `"end"` or `"none"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skybox: Option<String>,
    /// Cardinal lighting override, e.g. `"nether"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cardinal_light: Option<String>,
}

impl DimensionType {
    fn make_full_bright(&mut self) {
        // HACK: We don't have a lighting engine implemented. To avoid shrouding the
        // world in darkness, give all dimensions the max ambient light.
        // This allows even zero-light chunks to be visible.
        self.ambient_light = 1.0;
        self.attributes.get_or_insert_with(Compound::new).insert(
            "minecraft:visual/ambient_light_color",
            Value::String("#ffffff".into()),
        );
    }
}

impl Default for DimensionType {
    /// Extracted overworld settings from the registry codec, with full bright hack applied.
    fn default() -> Self {
        static DEFAULT: LazyLock<DimensionType> = LazyLock::new(|| {
            let codec = RegistryCodec::default();
            let overworld = codec
                .registry(DimensionTypeRegistry::KEY)
                .iter()
                .find(|value| value.name.as_str() == "minecraft:overworld")
                .expect("extracted overworld dimension");
            let mut dimension = DimensionType::deserialize(overworld.element.clone())
                .expect("valid extracted overworld dimension");
            dimension.make_full_bright();
            dimension
        });
        DEFAULT.clone()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Dimension environment attributes must survive deserialization and serialization
    /// so clients receive the configured sky colors, light factors, and fog.
    #[test]
    fn dimension_attributes_round_trip() {
        let codec = include_bytes!("../extracted/registry_codec.json");
        let compound = serde_json::from_slice::<Compound>(codec).expect("valid registry codec");

        let Value::Compound(dimensions) = compound
            .get("minecraft:dimension_type")
            .expect("dimension_type registry")
            .clone()
        else {
            panic!("expected compound");
        };
        let Value::Compound(overworld) = dimensions
            .get("minecraft:overworld")
            .expect("overworld entry")
            .clone()
        else {
            panic!("expected compound");
        };

        let dimension_type = DimensionType::deserialize(overworld).expect("overworld deserializes");
        assert!(dimension_type.attributes.is_some());

        let reserialized = dimension_type
            .serialize(CompoundSerializer)
            .expect("serializes");
        let Some(Value::Compound(attrs)) = reserialized.get("attributes") else {
            panic!("attributes dropped from dimension type");
        };
        assert!(attrs.contains_key("minecraft:visual/sky_color"));
    }
}
