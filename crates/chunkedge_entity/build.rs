use std::collections::BTreeMap;

use anyhow::Context;
use chunkedge_build_utils::{ident, rerun_if_changed, write_generated_file};
use heck::{ToPascalCase, ToShoutySnakeCase, ToSnakeCase};
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Serialize};

#[path = "build/entity_enums.rs"]
mod entity_enums;

#[derive(Deserialize, Clone, Debug)]
struct Entity {
    #[serde(rename = "type")]
    typ: Option<String>,
    translation_key: Option<String>,
    fields: Vec<Field>,
    attributes: Option<Vec<Attribute>>,
    parent: Option<String>,
    default_bounding_box: Option<BoundingBox>,
}

#[derive(Deserialize, Clone, Debug)]
struct BoundingBox {
    size_x: f64,
    size_y: f64,
    size_z: f64,
}

#[derive(Deserialize, Clone, Debug)]
struct EntityTypes {
    entity_type: BTreeMap<String, i32>,
}

#[derive(Deserialize)]
struct TrackedDataHandlers {
    tracked_data_handler: BTreeMap<String, u8>,
}

#[derive(Deserialize)]
struct PaintingVariants {
    painting_variant: BTreeMap<String, i32>,
}

#[derive(Deserialize)]
struct PaintingDimensions {
    width: u32,
    height: u32,
}

#[derive(Deserialize, Clone, Debug)]
struct Field {
    name: String,
    index: u8,
    #[serde(flatten)]
    default_value: Value,
}

#[derive(Deserialize, Clone, Debug)]
struct Attribute {
    name: String,
    base_value: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
enum PaintingVariantValue {
    Identifier(String),
    Inline(PaintingVariantInline),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct PaintingVariantInline {
    width: i32,
    height: i32,
    asset_id: String,
    title: Option<String>,
    author: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type", content = "default_value", rename_all = "snake_case")]
#[allow(dead_code)]
enum Value {
    Byte(i8),
    Integer(i32),
    Long(i64),
    Float(f32),
    String(String),
    TextComponent(String),
    OptionalTextComponent(Option<String>),
    ItemStack(String),
    Boolean(bool),
    Rotation {
        pitch: f32,
        yaw: f32,
        roll: f32,
    },
    BlockPos(BlockPos),
    OptionalBlockPos(Option<BlockPos>),
    Facing(String),
    LazyEntityReference(Option<()>), // TODO
    BlockState(String),
    OptionalBlockState(Option<String>),
    Particle(String),
    ParticleList(Vec<String>),
    VillagerData {
        #[serde(rename = "type")]
        typ: String,
        profession: String,
        level: i32,
    },
    OptionalInt(Option<i32>),
    EntityPose(String),
    CatVariant(String),
    CatSoundVariant(String),
    CowVariant(String),
    CowSoundVariant(String),
    WolfVariant(String),
    WolfSoundVariant(String),
    FrogVariant(String),
    PigVariant(String),
    PigSoundVariant(String),
    ChickenVariant(String),
    ChickenSoundVariant(String),
    ZombieNautilusVariant(String),
    OptionalGlobalPos(Option<()>), // TODO
    PaintingVariant(PaintingVariantValue),
    SnifferState(String),
    ArmadilloState(String),
    CopperGolemState(String),
    WeatheringCopperState(String),
    Vector3f {
        x: f32,
        y: f32,
        z: f32,
    },
    Quaternionf {
        x: f32,
        y: f32,
        z: f32,
        w: f32,
    },
    ResolvableProfile(String),
    HumanoidArm(String),
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
struct BlockPos {
    x: i32,
    y: i32,
    z: i32,
}

impl Value {
    fn type_id(&self, handlers: &BTreeMap<String, u8>) -> anyhow::Result<u8> {
        let value = serde_json::to_value(self).context("failed to serialize tracked-data value")?;
        let name = value["type"]
            .as_str()
            .context("serialized tracked-data value has no type")?;

        // The extracted entity field tags predate these handler names. We keep those
        // tags for compatibility with entities.json and translate only the exceptions.
        let name = match name {
            "integer" => "int",
            "text_component" => "component",
            "optional_text_component" => "optional_component",
            "rotation" => "rotations",
            "facing" => "direction",
            "lazy_entity_reference" => "optional_living_entity_reference",
            "particle_list" => "particles",
            "optional_int" => "optional_unsigned_int",
            "entity_pose" => "pose",
            "vector3f" => "vector3",
            "quaternionf" => "quaternion",
            name => name,
        };
        handlers
            .get(name)
            .copied()
            .with_context(|| format!("missing tracked-data handler {name} in misc.json"))
    }

    fn field_type(&self) -> TokenStream {
        match self {
            Value::Byte(_) => quote!(i8),
            Value::Integer(_) => quote!(i32),
            Value::Long(_) => quote!(i64),
            Value::Float(_) => quote!(f32),
            Value::String(_) => quote!(String),
            Value::TextComponent(_) => quote!(chunkedge_protocol::Text),
            Value::OptionalTextComponent(_) => quote!(Option<chunkedge_protocol::Text>),
            Value::ItemStack(_) => quote!(chunkedge_protocol::ItemStack),
            Value::Boolean(_) => quote!(bool),
            Value::Rotation { .. } => quote!(crate::EulerAngle),
            Value::BlockPos(_) => quote!(chunkedge_protocol::BlockPos),
            Value::OptionalBlockPos(_) => quote!(Option<chunkedge_protocol::BlockPos>),
            Value::Facing(_) => quote!(chunkedge_protocol::Direction),
            Value::LazyEntityReference(_) => quote!(()), // TODO
            Value::BlockState(_) => quote!(chunkedge_protocol::BlockState),
            Value::OptionalBlockState(_) => quote!(Option<chunkedge_protocol::BlockState>),
            Value::Particle(_) => {
                quote!(chunkedge_protocol::packets::play::level_particles_s2c::Particle)
            }
            Value::ParticleList(_) => {
                quote!(Vec<chunkedge_protocol::packets::play::level_particles_s2c::Particle>)
            }
            Value::VillagerData { .. } => quote!(crate::VillagerData),
            Value::OptionalInt(_) => quote!(Option<i32>),
            Value::EntityPose(_) => quote!(crate::Pose),
            Value::CatVariant(_) => quote!(crate::CatKind),
            Value::CatSoundVariant(_) => quote!(crate::CatSoundKind),
            Value::CowVariant(_) => quote!(crate::CowKind),
            Value::CowSoundVariant(_) => quote!(crate::CowSoundKind),
            Value::WolfVariant(_) => quote!(crate::WolfKind),
            Value::WolfSoundVariant(_) => quote!(crate::WolfSoundKind),
            Value::FrogVariant(_) => quote!(crate::FrogKind),
            Value::PigVariant(_) => quote!(crate::PigKind),
            Value::PigSoundVariant(_) => quote!(crate::PigSoundKind),
            Value::ChickenVariant(_) => quote!(crate::ChickenKind),
            Value::ChickenSoundVariant(_) => quote!(crate::ChickenSoundKind),
            Value::ZombieNautilusVariant(_) => quote!(crate::ZombieNautilusKind),
            Value::OptionalGlobalPos(_) => quote!(()), // TODO
            Value::PaintingVariant(_) => {
                quote!(chunkedge_binary::IdOr<crate::PaintingVariantDefinition>)
            }
            Value::SnifferState(_) => quote!(crate::SnifferState),
            Value::ArmadilloState(_) => quote!(crate::ArmadilloState),
            Value::CopperGolemState(_) => quote!(crate::CopperGolemState),
            Value::WeatheringCopperState(_) => quote!(crate::WeatheringCopperState),
            Value::ResolvableProfile(_) => quote!(()), // TODO
            Value::HumanoidArm(_) => quote!(crate::HumanoidArm),
            Value::Vector3f { .. } => quote!(chunkedge_math::Vec3),
            Value::Quaternionf { .. } => quote!(chunkedge_math::Quat),
        }
    }

    fn default_expr(&self) -> TokenStream {
        match self {
            Value::Byte(b) => quote!(#b),
            Value::Integer(i) => quote!(#i),
            Value::Long(l) => quote!(#l),
            Value::Float(f) => quote!(#f),
            Value::String(s) => quote!(#s.to_owned()),
            Value::TextComponent(txt) => {
                quote!(chunkedge_protocol::Text::text(#txt))
            }
            Value::OptionalTextComponent(t) => match t {
                Some(txt) => quote!(Some(chunkedge_protocol::Text::text(#txt))),
                None => quote!(None),
            },
            Value::ItemStack(_stack) => {
                quote!(chunkedge_protocol::ItemStack::default())
            }
            Value::Boolean(b) => quote!(#b),
            Value::Rotation { pitch, yaw, roll } => quote! {
                crate::EulerAngle {
                    pitch: #pitch,
                    yaw: #yaw,
                    roll: #roll,
                }
            },
            Value::BlockPos(BlockPos { x, y, z }) => {
                quote!(chunkedge_protocol::BlockPos { x: #x, y: #y, z: #z })
            }
            Value::OptionalBlockPos(pos) => {
                assert!(pos.is_none());
                quote!(None)
            }
            Value::Facing(f) => {
                let variant = ident(f.to_pascal_case());
                quote!(chunkedge_protocol::Direction::#variant)
            }
            Value::LazyEntityReference(_) => {
                quote!(())
            }
            Value::BlockState(_) => {
                quote!(chunkedge_protocol::BlockState::default())
            }
            Value::OptionalBlockState(bs) => {
                assert!(bs.is_none());
                quote!(None)
            }
            Value::Particle(p) => match p.to_pascal_case().as_str() {
                // TODO: fix this, now an entyity has this as the default, so we need to extract
                // the data here too somehow
                "EntityEffect" => {
                    quote!(chunkedge_protocol::packets::play::level_particles_s2c::Particle::EntityEffect { color: 0 })
                }
                other => {
                    let variant = ident(other);
                    quote!(chunkedge_protocol::packets::play::level_particles_s2c::Particle::#variant)
                }
            },
            Value::ParticleList(_) => quote!(Vec::new()),
            Value::VillagerData {
                typ,
                profession,
                level,
            } => {
                let typ = ident(typ.to_pascal_case());
                let profession = ident(profession.to_pascal_case());
                quote! {
                    crate::VillagerData {
                        kind: crate::VillagerKind::#typ,
                        profession: crate::VillagerProfession::#profession,
                        level: #level,
                    }
                }
            }
            Value::OptionalInt(i) => {
                assert!(i.is_none());
                quote!(None)
            }
            Value::EntityPose(p) => {
                let variant = ident(entity_enums::variant_name("entity_pose", p));
                quote!(crate::Pose::#variant)
            }
            Value::CatVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::CatKind::#variant)
            }
            Value::CatSoundVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft:");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::CatSoundKind::#variant)
            }
            Value::CowVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::CowKind::#variant)
            }
            Value::CowSoundVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft:");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::CowSoundKind::#variant)
            }
            Value::WolfVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::WolfKind::#variant)
            }
            Value::WolfSoundVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::WolfSoundKind::#variant)
            }
            Value::FrogVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::FrogKind::#variant)
            }
            Value::PigVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::PigKind::#variant)
            }
            Value::PigSoundVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft:");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::PigSoundKind::#variant)
            }
            Value::ChickenVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::ChickenKind::#variant)
            }
            Value::ChickenSoundVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft:");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::ChickenSoundKind::#variant)
            }
            Value::ZombieNautilusVariant(c) => {
                let stripped_variant = c.trim_start_matches("minecraft:");
                let variant = ident(stripped_variant.to_pascal_case());
                quote!(crate::ZombieNautilusKind::#variant)
            }
            Value::OptionalGlobalPos(gp) => {
                assert!(gp.is_none());
                quote!(None)
            }
            Value::PaintingVariant(p) => match p {
                PaintingVariantValue::Identifier(painting) => {
                    let stripped_variant = painting.trim_start_matches("minecraft:");
                    let variant = ident(stripped_variant.to_pascal_case());
                    quote!(chunkedge_binary::IdOr::id(crate::PaintingKind::#variant as i32))
                }
                PaintingVariantValue::Inline(inline) => {
                    let PaintingVariantInline {
                        width,
                        height,
                        asset_id,
                        title,
                        author,
                    } = inline;

                    let title = if let Some(title) = title {
                        quote!(Some(#title.into()))
                    } else {
                        quote!(None)
                    };

                    let author = if let Some(author) = author {
                        quote!(Some(#author.into()))
                    } else {
                        quote!(None)
                    };

                    quote! {
                        chunkedge_binary::IdOr::inline(crate::PaintingVariantDefinition {
                            width: #width,
                            height: #height,
                            asset_id: #asset_id.to_owned(),
                            title: #title,
                            author: #author,
                        })
                    }
                }
            },
            Value::SnifferState(s) => {
                let state = ident(s.to_pascal_case());
                quote!(crate::SnifferState::#state)
            }
            Value::ArmadilloState(s) => {
                let state = ident(s.to_pascal_case());
                quote!(crate::ArmadilloState::#state)
            }
            Value::CopperGolemState(s) => {
                let state = ident(s.to_pascal_case());
                quote!(crate::CopperGolemState::#state)
            }
            Value::WeatheringCopperState(s) => {
                let state = ident(s.to_pascal_case());
                quote!(crate::WeatheringCopperState::#state)
            }
            Value::ResolvableProfile(_) => {
                quote!(()) // TODO
            }
            Value::HumanoidArm(s) => {
                let arm = ident(s.to_pascal_case());
                quote!(crate::HumanoidArm::#arm)
            }
            Value::Vector3f { x, y, z } => quote!(chunkedge_math::Vec3::new(#x, #y, #z)),
            Value::Quaternionf { x, y, z, w } => quote! {
                chunkedge_math::Quat::from_xyzw(#x, #y, #z, #w)
            },
        }
    }

    fn encodable_expr(&self, self_lvalue: TokenStream) -> TokenStream {
        match self {
            Value::Long(_) => quote!(chunkedge_protocol::VarLong(#self_lvalue)),
            Value::Integer(_) => quote!(VarInt(#self_lvalue)),
            Value::OptionalInt(_) => quote!(OptionalInt(#self_lvalue)),
            Value::OptionalBlockState(_) => quote!(OptionalBlockState(#self_lvalue)),
            Value::PaintingVariant(_) => quote!(PaintingVariant(&#self_lvalue)),
            Value::TextComponent(_) => {
                quote!(chunkedge_binary::TextComponent::from(#self_lvalue.clone()))
            }
            Value::OptionalTextComponent(_) => {
                quote!(
                    #self_lvalue
                        .clone()
                        .map(chunkedge_binary::TextComponent::from)
                )
            }
            _ => quote!(&#self_lvalue),
        }
    }
}

type Entities = BTreeMap<String, Entity>;

pub fn main() -> anyhow::Result<()> {
    rerun_if_changed([
        "extracted/misc.json",
        "extracted/entities.json",
        "extracted/paintings.json",
    ]);

    write_generated_file(build_entities()?, "entity.rs")?;
    write_generated_file(entity_enums::build()?, "entity_enums.rs")?;
    write_generated_file(build_painting_kind()?, "painting_kind.rs")?;
    write_generated_file(build_entity_hitbox_defaults()?, "entity_hitbox_defaults.rs")?;

    Ok(())
}

fn build_painting_kind() -> anyhow::Result<TokenStream> {
    let paintings: PaintingVariants = serde_json::from_str(include_str!("extracted/misc.json"))
        .context("failed to deserialize painting variants from misc.json")?;
    let mut paintings: Vec<_> = paintings.painting_variant.into_iter().collect();
    paintings.sort_by_key(|(_, id)| *id);
    // paintings.json is the extractor's standalone copy of the painting_variant registry codec.
    let dimensions: BTreeMap<String, PaintingDimensions> =
        serde_json::from_str(include_str!("extracted/paintings.json"))
            .context("failed to deserialize paintings.json")?;

    anyhow::ensure!(
        paintings.len() == dimensions.len(),
        "painting IDs and painting definitions have different counts"
    );

    for (expected, (name, id)) in paintings.iter().enumerate() {
        anyhow::ensure!(
            *id == expected as i32,
            "painting variant {name} has ID {id}, expected {expected}"
        );
    }

    let count = paintings.len();
    let variants: Vec<_> = paintings
        .iter()
        .map(|(name, id)| {
            let variant = ident(name.to_pascal_case());
            let default = (name == "alban").then(|| quote!(#[default]));
            quote! { #default #variant = #id, }
        })
        .collect();
    let names: Vec<_> = paintings
        .iter()
        .map(|(name, _)| ident(name.to_pascal_case()))
        .collect();
    let sizes: Vec<_> = paintings
        .iter()
        .map(|(name, _)| {
            let PaintingDimensions { width, height } = dimensions
                .get(&format!("minecraft:{name}"))
                .with_context(|| format!("missing dimensions for painting {name}"))?;
            anyhow::ensure!(
                *width > 0 && *height > 0,
                "painting {name} has invalid dimensions"
            );
            Ok(quote! { [#width, #height], })
        })
        .collect::<anyhow::Result<_>>()?;

    anyhow::ensure!(
        paintings.iter().any(|(name, _)| name == "alban"),
        "painting variants must contain the default alban"
    );

    Ok(quote! {
        #[repr(i32)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug, Encode, Decode)]
        pub enum PaintingKind {
            #(#variants)*
        }

        impl PaintingKind {
            pub const ALL: [Self; #count] = [#(Self::#names,)*];

            pub fn registry_id(self) -> chunkedge_protocol::RegistryId {
                chunkedge_protocol::RegistryId::new(self as i32)
            }

            pub fn from_registry_id(id: chunkedge_protocol::RegistryId) -> Option<Self> {
                let idx = usize::try_from(id.id()).ok()?;
                Self::ALL.get(idx).copied()
            }

            pub(crate) fn hitbox_bounds(self) -> chunkedge_math::UVec3 {
                const DIMENSIONS: [[u32; 2]; #count] = [#(#sizes)*];
                let [width, height] = DIMENSIONS[self as usize];
                chunkedge_math::UVec3::new(width, height, width)
            }
        }
    })
}

fn build_entity_hitbox_defaults() -> anyhow::Result<TokenStream> {
    // entities.json records default bounds, but not which hitboxes change at runtime.
    // Keep this list in sync with the update_*_hitbox handlers in src/hitbox.rs,
    // including the kinds matched by update_passive_child_hitbox. Marker is excluded
    // so its hitbox stays user-managed. When updating MC, review new or changed
    // entity kinds against those handlers and their pose, age, data, and facing rules.
    const DYNAMIC: &[&str] = &[
        "area_effect_cloud",
        "armor_stand",
        "bee",
        "camel",
        "chicken",
        "donkey",
        "fox",
        "glow_item_frame",
        "goat",
        "hoglin",
        "horse",
        "item_frame",
        "llama",
        "magma_cube",
        "marker",
        "mooshroom",
        "mule",
        "ocelot",
        "painting",
        "panda",
        "pig",
        "piglin",
        "player",
        "polar_bear",
        "rabbit",
        "sheep",
        "shulker",
        "skeleton_horse",
        "slime",
        "sulfur_cube",
        "turtle",
        "villager",
        "warden",
        "wolf",
        "zoglin",
        "zombie",
        "zombie_horse",
    ];

    let entity_types =
        serde_json::from_str::<EntityTypes>(include_str!("extracted/misc.json"))?.entity_type;
    let entities: Entities = serde_json::from_str(include_str!("extracted/entities.json"))?;
    let mut boxes = BTreeMap::new();
    for entity in entities.into_values() {
        if let Some(typ) = entity.typ {
            let size = entity
                .default_bounding_box
                .with_context(|| format!("missing default bounding box for {typ}"))?;
            anyhow::ensure!(
                [size.size_x, size.size_y, size.size_z]
                    .into_iter()
                    .all(|dimension| dimension.is_finite() && dimension >= 0.0),
                "invalid default bounding box for {typ}"
            );
            boxes.insert(typ, size);
        }
    }

    for name in DYNAMIC {
        anyhow::ensure!(
            entity_types.contains_key(*name),
            "unknown dynamic entity {name}"
        );
    }

    let mut arms = Vec::new();
    for (name, id) in entity_types {
        if DYNAMIC.contains(&name.as_str()) {
            continue;
        }
        let source = if boxes.contains_key(&name) {
            name.as_str()
        } else if name.ends_with("_chest_boat") {
            "acacia_chest_boat"
        } else if name.ends_with("_boat") {
            "acacia_boat"
        } else {
            anyhow::bail!("missing default bounding box for entity type {name}");
        };
        let size = &boxes[source];
        let (x, y, z) = (size.size_x, size.size_y, size.size_z);
        arms.push(quote! { #id => Some(chunkedge_math::DVec3::new(#x, #y, #z)), });
    }

    Ok(quote! {
        fn default_entity_hitbox_size(kind: crate::EntityKind) -> Option<chunkedge_math::DVec3> {
            match kind.get() {
                #(#arms)*
                _ => None,
            }
        }
    })
}

fn build_entities() -> anyhow::Result<TokenStream> {
    let misc = include_str!("extracted/misc.json");
    let entity_types = serde_json::from_str::<EntityTypes>(misc)
        .context("failed to deserialize misc.json")?
        .entity_type;
    let handlers = serde_json::from_str::<TrackedDataHandlers>(misc)
        .context("failed to deserialize tracked-data handlers from misc.json")?
        .tracked_data_handler;

    let entities: Entities =
        serde_json::from_str::<Entities>(include_str!("extracted/entities.json"))
            .context("failed to deserialize entities.json")?
            .into_iter()
            .collect();

    let mut entity_kind_consts = TokenStream::new();
    let mut entity_kind_fmt_args = TokenStream::new();
    let mut translation_key_arms = TokenStream::new();
    let mut modules = TokenStream::new();
    let mut systems = TokenStream::new();
    let mut system_names = vec![];
    let mut derived_system_names = vec![];

    for (entity_name, entity) in entities {
        let entity_name_ident = ident(&entity_name);
        let stripped_shouty_entity_name = strip_entity_suffix(&entity_name).to_shouty_snake_case();
        let stripped_shouty_entity_name_ident = ident(&stripped_shouty_entity_name);
        let stripped_snake_entity_name = strip_entity_suffix(&entity_name).to_snake_case();
        let stripped_snake_entity_name_ident = ident(&stripped_snake_entity_name);

        let mut module_body = TokenStream::new();

        let mut requires: Vec<TokenStream> = vec![];

        if let Some(parent_name) = &entity.parent {
            let stripped_snake_parent_name = strip_entity_suffix(parent_name).to_snake_case();
            let stripped_snake_parent_name_ident = ident(&stripped_snake_parent_name);
            let pascal_parent_name_ident = ident(parent_name);

            let module_doc = format!(
                "Parent class: \
                 [`{stripped_snake_parent_name}`][super::{stripped_snake_parent_name}]."
            );

            module_body.extend([quote! {
                #![doc = #module_doc]
            }]);

            requires.push(quote! {
                super::#stripped_snake_parent_name_ident::#pascal_parent_name_ident
            });
        } else {
            requires.push(quote! {
                super::EntityId,
                super::UniqueId,
                super::EntityLayerId,
                super::OldEntityLayerId,
                super::Position,
                super::OldPosition,
                super::Look,
                super::HeadYaw,
                super::OnGround,
                super::Velocity,
                super::EntityStatuses,
                super::EntityAnimations,
                super::ObjectData,
                super::tracked_data::TrackedData
            });
        }

        // Is this a concrete entity type?
        if let Some(entity_type) = entity.typ {
            let entity_type_id = entity_types[&entity_type];

            entity_kind_consts.extend([quote! {
                pub const #stripped_shouty_entity_name_ident: EntityKind = EntityKind(#entity_type_id);
            }]);

            entity_kind_fmt_args.extend([quote! {
                EntityKind::#stripped_shouty_entity_name_ident => write!(f, "{} ({})", #entity_type_id, #stripped_shouty_entity_name),
            }]);

            let translation_key_expr = if let Some(key) = entity.translation_key {
                quote!(Some(#key))
            } else {
                quote!(None)
            };

            translation_key_arms.extend([quote! {
                EntityKind::#stripped_shouty_entity_name_ident => #translation_key_expr,
            }]);

            requires.push(quote! {
                super::EntityKind = super::EntityKind::#stripped_shouty_entity_name_ident
            });

            if let Some(attributes) = &entity.attributes {
                let mut attribute_default_values = TokenStream::new();

                for attribute in attributes {
                    let name = ident(attribute.name.to_pascal_case());
                    let base_value = attribute.base_value;
                    attribute_default_values.extend([quote! {
                        .with_attribute_and_value(
                            super::EntityAttribute::#name,
                            #base_value,
                        )
                    }]);
                }

                requires.push(quote! {
                    super::attributes::EntityAttributes =
                        super::attributes::EntityAttributes::new() #attribute_default_values
                });
            }
        }

        for field in &entity.fields {
            let pascal_field_name_ident = ident(field.name.to_pascal_case());
            let snake_field_name = field.name.to_snake_case();
            let inner_type = field.default_value.field_type();
            let default_expr = field.default_value.default_expr();

            module_body.extend([quote! {
                #[derive(bevy_ecs::component::Component, PartialEq, Clone, Debug, ::derive_more::Deref, ::derive_more::DerefMut)]
                pub struct #pascal_field_name_ident(pub #inner_type);

                #[allow(clippy::derivable_impls)]
                impl Default for #pascal_field_name_ident {
                    fn default() -> Self {
                        Self(#default_expr)
                    }
                }
            }]);

            requires.push(quote!(#pascal_field_name_ident));

            let system_name_ident = ident(format!(
                "update_{stripped_snake_entity_name}_{snake_field_name}"
            ));
            let component_path =
                quote!(#stripped_snake_entity_name_ident::#pascal_field_name_ident);

            system_names.push(quote!(#system_name_ident));

            let data_index = field.index;
            let data_type = field.default_value.type_id(&handlers)?;
            let encodable_expr = field.default_value.encodable_expr(quote!(value.0));

            systems.extend([quote! {
                #[allow(clippy::needless_borrow)]
                #[allow(clippy::suspicious_else_formatting)]
                #[allow(clippy::needless_borrows_for_generic_args)]
                fn #system_name_ident(
                    mut query: Query<(&#component_path, &mut tracked_data::TrackedData), Changed<#component_path>>
                ) {
                    for (value, mut tracked_data) in &mut query {
                        if *value == Default::default() {
                            tracked_data.remove_init_value(#data_index);
                        } else {
                            tracked_data.insert_init_value(#data_index, #data_type, #encodable_expr);
                        }

                        if !tracked_data.is_added() {
                            tracked_data.append_update_value(#data_index, #data_type, #encodable_expr);
                        }
                    }
                }
            }]);
        }

        match entity_name.as_str() {
            "LivingEntity" => {
                requires.push(quote! {
                    Absorption,
                    super::attributes::EntityAttributes,
                    super::attributes::TrackedEntityAttributes,
                    super::active_status_effects::ActiveStatusEffects
                });
            }
            "PlayerEntity" => {
                requires.push(quote! {
                    Food,
                    Saturation
                });
            }
            _ => {}
        }

        let marker_doc = format!("Marker component for `{stripped_snake_entity_name}` entities.");

        let require_attr = if requires.is_empty() {
            quote!()
        } else {
            quote!(#[require(#(#requires),*)])
        };

        module_body.extend([quote! {
            #[doc = #marker_doc]
            #[derive(bevy_ecs::component::Component, Copy, Clone, Default, Debug)]
            #require_attr
            pub struct #entity_name_ident;
        }]);

        match entity_name.as_str() {
            "LivingEntity" => {
                module_body.extend([quote! {
                    #[doc = "Special untracked component for `LivingEntity` entities."]
                    #[derive(bevy_ecs::component::Component, Copy, Clone, Default, Debug)]
                    pub struct Absorption(pub f32);
                }]);
            }
            "PlayerEntity" => {
                module_body.extend([quote! {
                    #[doc = "Special untracked component for `PlayerEntity` entities."]
                    #[derive(bevy_ecs::component::Component, Copy, Clone, Debug)]
                    pub struct Food(pub i32);

                    impl Default for Food {
                        fn default() -> Self {
                            Self(20)
                        }
                    }

                    #[doc = "Special untracked component for `PlayerEntity` entities."]
                    #[derive(bevy_ecs::component::Component, Copy, Clone, Default, Debug)]
                    pub struct Saturation(pub f32);
                }]);
            }
            _ => {}
        }

        modules.extend([quote! {
            #[allow(clippy::module_inception)]
            pub mod #stripped_snake_entity_name_ident {
                #module_body
            }
        }]);
    }

    systems.extend([quote! {
        #[doc = "Special case for `living::Absorption`."]
        #[doc = "Updates the `AbsorptionAmount` component of the player entity."]
        fn update_living_and_player_absorption(
            mut query: Query<(&living::Absorption, &mut player::AbsorptionAmount), Changed<living::Absorption>>
        ) {
            for (living_absorption, mut player_absorption) in &mut query {
                player_absorption.0 = living_absorption.0;
            }
        }

        #[doc = "Special case for `living::Attributes`."]
        fn update_living_attributes(
            mut query: Query<(
                &mut attributes::TrackedEntityAttributes,
                &mut attributes::EntityAttributes,
            ),
            Changed<attributes::EntityAttributes>>
        ) {
            for (mut tracked, mut attributes) in &mut query {
                for attribute in attributes.take_recently_changed() {
                    tracked.mark_modified(&attributes, attribute);
                }
            }
        }
    }]);

    derived_system_names.push(quote!(update_living_and_player_absorption));
    derived_system_names.push(quote!(update_living_attributes));

    #[derive(Deserialize, Debug)]
    struct MiscEntityData {
        entity_status: BTreeMap<String, u8>,
        entity_animation: BTreeMap<String, u8>,
    }

    let misc_entity_data: MiscEntityData =
        serde_json::from_str(include_str!("extracted/misc.json"))?;

    let entity_status_variants = misc_entity_data
        .entity_status
        .into_iter()
        .map(|(name, code)| {
            let name = ident(name.to_pascal_case());
            let code = code as isize;

            quote! {
                #name = #code,
            }
        });

    let entity_animation_variants =
        misc_entity_data
            .entity_animation
            .into_iter()
            .map(|(name, code)| {
                let name = ident(name.to_pascal_case());
                let code = code as isize;

                quote! {
                    #name = #code,
                }
            });

    Ok(quote! {
        use chunkedge_generated::attributes::EntityAttribute;

        #modules

        #[doc = "Identifies the type of an entity."]
        #[doc = "As a component, the entity kind should not be modified."]
        #[derive(Component, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, ::derive_more::Deref)]
        pub struct EntityKind(i32);

        impl EntityKind {
            #entity_kind_consts

            pub const fn new(inner: i32) -> Self {
                Self(inner)
            }

            pub const fn get(self) -> i32 {
                self.0
            }

            pub const fn translation_key(self) -> Option<&'static str> {
                match self {
                    #translation_key_arms
                    _ => None,
                }
            }
        }

        impl std::fmt::Debug for EntityKind {
            #[allow(clippy::write_literal)]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match *self {
                    #entity_kind_fmt_args
                    EntityKind(other) => write!(f, "{other}"),
                }
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum EntityStatus {
            #(#entity_status_variants)*
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum EntityAnimation {
            #(#entity_animation_variants)*
        }

        fn add_tracked_data_systems(app: &mut App) {
            #systems

            #(
                app.add_systems(PostUpdate, #derived_system_names.in_set(UpdateDerivedEntityDataSet));
            )*

            #(
                app.add_systems(
                    PostUpdate,
                    #system_names
                        .in_set(UpdateTrackedDataSet)
                        .ambiguous_with(UpdateTrackedDataSet)
                );
            )*
        }
    })
}

fn strip_entity_suffix(string: &str) -> String {
    let stripped = string.strip_suffix("Entity").unwrap_or(string);

    if stripped.is_empty() {
        string
    } else {
        stripped
    }
    .to_owned()
}
