#![allow(clippy::type_complexity)]

use crate::attributes::{EntityAttribute, EntityAttributes};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryData;
use chunkedge_binary::IdOr;
use chunkedge_item::ItemComponent;
use chunkedge_math::{Aabb, UVec3};
use chunkedge_protocol::Direction;
use derive_more::Deref;

use crate::*;

include!(concat!(env!("OUT_DIR"), "/entity_hitbox_defaults.rs"));

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HitboxShapeUpdateSet;

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HitboxComponentsAddSet;

#[derive(SystemSet, Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HitboxUpdateSet;

pub struct HitboxPlugin;

#[derive(Resource)]
/// Settings for hitbox plugin
pub struct EntityHitboxSettings {
    /// Controls if a plugin should add hitbox component on each created entity.
    /// Otherwise you should add hitbox component by yourself in order to use
    /// it.
    pub add_hitbox_component: bool,
}

impl Default for EntityHitboxSettings {
    fn default() -> Self {
        Self {
            add_hitbox_component: true,
        }
    }
}

impl Plugin for HitboxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityHitboxSettings>()
            .configure_sets(
                PreUpdate,
                (
                    HitboxComponentsAddSet,
                    HitboxShapeUpdateSet,
                    HitboxUpdateSet,
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                add_hitbox_component.in_set(HitboxComponentsAddSet),
            )
            .add_systems(PreUpdate, update_hitbox_shape.in_set(HitboxShapeUpdateSet))
            .add_systems(PreUpdate, update_hitbox.in_set(HitboxUpdateSet))
            .add_systems(
                PostUpdate,
                add_hitbox_component.in_set(HitboxComponentsAddSet),
            );
    }
}

/// Bounding box relative to the entity's position, including local offsets.
/// Its shape follows vanilla age, pose, metadata, and scale rules each tick.
/// Marker entities retain their manually supplied shapes.
#[derive(Component, Debug, PartialEq, Deref)]
pub struct HitboxShape(pub Aabb);

/// Hitbox, aabb of which is calculated each tick using its position and
/// [`HitboxShape`]. In order to change size of this hitbox you need to change
/// [`HitboxShape`].
#[derive(Component, Debug, Deref)]
pub struct Hitbox(Aabb);

impl HitboxShape {
    pub const ZERO: HitboxShape = HitboxShape(Aabb::ZERO);

    pub fn get(&self) -> Aabb {
        self.0
    }

    pub(crate) fn in_world(&self, pos: DVec3) -> Aabb {
        self.0 + pos
    }
}

impl Hitbox {
    pub fn get(&self) -> Aabb {
        self.0
    }
}

fn add_hitbox_component(
    settings: Res<EntityHitboxSettings>,
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Position,
            Option<&entity::Entity>,
            Option<&HitboxShape>,
            Option<&Hitbox>,
        ),
        Or<(Without<HitboxShape>, Without<Hitbox>)>,
    >,
) {
    for (entity, pos, game_entity, shape, hitbox) in &query {
        let auto = settings.add_hitbox_component && game_entity.is_some();
        if shape.is_none() && auto {
            commands.entity(entity).insert(HitboxShape::ZERO);
        }
        if hitbox.is_none() && (auto || shape.is_some()) {
            let local = shape.map_or(Aabb::ZERO, HitboxShape::get);
            commands.entity(entity).insert(Hitbox(local + pos.0));
        }
    }
}

fn update_hitbox(
    mut hitbox_query: Query<
        (&mut Hitbox, &HitboxShape, &Position),
        Or<(Changed<HitboxShape>, Changed<Position>)>,
    >,
) {
    for (mut in_world, hitbox, pos) in &mut hitbox_query {
        in_world.0 = hitbox.in_world(pos.0);
    }
}

#[derive(Clone, Copy)]
struct HitboxTemplate {
    bounds: Aabb,
    scalable: bool,
}

#[derive(QueryData)]
#[query_data(mutable)]
struct HitboxEntityQuery {
    shape: &'static mut HitboxShape,
    kind: &'static EntityKind,
    pose: &'static entity::Pose,
    attributes: Option<&'static EntityAttributes>,
    living: Option<&'static living::LivingEntity>,
    child: Option<&'static ageable_mob::Baby>,
    zombie: Option<&'static zombie::Baby>,
    piglin: Option<&'static piglin::Baby>,
    zoglin: Option<&'static zoglin::Baby>,
    cube: Option<&'static abstract_cube_mob::Size>,
    phantom: Option<&'static phantom::Size>,
    pufferfish: Option<&'static pufferfish::PuffState>,
    salmon: Option<&'static salmon::Type>,
    sniffer: Option<&'static sniffer::State>,
    armor_stand: Option<&'static armor_stand::ClientFlags>,
    cloud: Option<&'static area_effect_cloud::Radius>,
    interaction_width: Option<&'static interaction::Width>,
    interaction_height: Option<&'static interaction::Height>,
    frame_item: Option<&'static item_frame::Item>,
    facing: Option<&'static hanging::Direction>,
    painting: Option<&'static painting::PaintingVariant>,
    shulker_peek: Option<&'static shulker::Peek>,
    shulker_face: Option<&'static shulker::AttachFace>,
}

fn update_hitbox_shape(mut query: Query<HitboxEntityQuery>) {
    for mut entity in &mut query {
        let kind = *entity.kind;
        if kind == EntityKind::MARKER {
            continue;
        }
        let baby = entity.child.is_some_and(|v| v.0)
            || entity.zombie.is_some_and(|v| v.0)
            || entity.piglin.is_some_and(|v| v.0)
            || entity.zoglin.is_some_and(|v| v.0);
        let variant = if let Some(puff) = entity.pufferfish {
            match puff.0 {
                0 => 0,
                1 => 1,
                _ => 2,
            }
        } else if let Some(salmon) = entity.salmon {
            salmon.0.clamp(0, 2) as usize
        } else if let Some(phantom) = entity.phantom {
            phantom.0.clamp(0, 64) as usize
        } else if let Some(sniffer) = entity.sniffer {
            usize::from(sniffer.0 == SnifferState::Digging)
        } else if let Some(flags) = entity.armor_stand {
            usize::from(flags.0 & 1 != 0) + 2 * usize::from(flags.0 & 16 != 0)
        } else {
            0
        };
        let Some(template) = entity_hitbox_template(kind, baby, entity.pose.0, variant) else {
            continue;
        };
        let scale = entity
            .attributes
            .filter(|_| entity.living.is_some())
            .and_then(|attributes| attributes.get_compute_value(EntityAttribute::Scale))
            .unwrap_or(1.0) as f32;
        let mut bounds = template.bounds;
        if template.scalable {
            if let Some(cube) = entity.cube {
                bounds = scale_bounds(bounds, cube.0.clamp(1, 127) as f32);
            }
            bounds = scale_bounds(bounds, scale);
        }
        let scale = f64::from(scale);
        if let Some(radius) = entity.cloud {
            let diameter = f64::from(radius.0.max(0.0)) * 2.0;
            bounds = Aabb::from_bottom_size(DVec3::ZERO, DVec3::new(diameter, 0.5, diameter));
        }
        if let (Some(width), Some(height)) = (entity.interaction_width, entity.interaction_height) {
            bounds = Aabb::from_bottom_size(
                DVec3::ZERO,
                DVec3::new(
                    f64::from(width.0.max(0.0)),
                    f64::from(height.0.max(0.0)),
                    f64::from(width.0.max(0.0)),
                ),
            );
        }
        if let (Some(item), Some(facing)) = (entity.frame_item, entity.facing) {
            let map_id = ItemComponent::MapId(VarInt(0)).id() as usize;
            let width = if !item.0.is_empty() && item.0.get_component(map_id).is_some() {
                1.0
            } else {
                0.75
            };
            let normal = direction_vector(facing.0);
            let center = DVec3::splat(0.5) - normal * 0.46875;
            let size = DVec3::new(
                if normal.x != 0.0 { 0.0625 } else { width },
                if normal.y != 0.0 { 0.0625 } else { width },
                if normal.z != 0.0 { 0.0625 } else { width },
            );
            bounds = Aabb::new(center - size / 2.0, center + size / 2.0);
        }
        if let (Some(variant), Some(facing)) = (entity.painting, entity.facing) {
            bounds = painting_hitbox(&variant.0, facing.0);
        }
        if let (Some(peek), Some(face)) = (entity.shulker_peek, entity.shulker_face) {
            let mut min = DVec3::new(-0.5, 0.0, -0.5) * scale;
            let mut max = DVec3::new(0.5, 1.0, 0.5) * scale;
            let peek = (0.5
                - (f64::from(peek.0.clamp(0, 100)) * 0.01 * std::f64::consts::PI).cos() * 0.5)
                * scale;
            let movement = -direction_vector(face.0) * peek;
            min += movement.min(DVec3::ZERO);
            max += movement.max(DVec3::ZERO);
            bounds = Aabb::new(min, max);
        }
        entity.shape.set_if_neq(HitboxShape(bounds));
    }
}

fn scale_bounds(bounds: Aabb, scale: f32) -> Aabb {
    // Vanilla applies EntityDimensions.scale in float precision, including
    // separate cube-size and attribute multiplications.
    Aabb::new(
        (bounds.min().as_vec3() * scale).as_dvec3(),
        (bounds.max().as_vec3() * scale).as_dvec3(),
    )
}

fn direction_vector(direction: Direction) -> DVec3 {
    match direction {
        Direction::Down => -DVec3::Y,
        Direction::Up => DVec3::Y,
        Direction::North => -DVec3::Z,
        Direction::South => DVec3::Z,
        Direction::West => -DVec3::X,
        Direction::East => DVec3::X,
    }
}

fn painting_hitbox(variant: &IdOr<PaintingVariantDefinition>, facing: Direction) -> Aabb {
    let size = match variant {
        IdOr::Id(id) => PaintingVariant::from_registry_id(*id)
            .map_or_else(|| UVec3::splat(1), PaintingVariant::hitbox_bounds),
        IdOr::Inline(inline) => inline_painting_hitbox_bounds(inline),
    };
    let normal = match facing {
        Direction::Down | Direction::Up => DVec3::Z,
        _ => direction_vector(facing),
    };
    let left = DVec3::new(normal.z, 0.0, -normal.x);
    let mut center = DVec3::splat(0.5) - normal * 0.46875;
    center += left * if size.x.is_multiple_of(2) { 0.5 } else { 0.0 };
    center.y += if size.y.is_multiple_of(2) { 0.5 } else { 0.0 };
    let size = if normal.x != 0.0 {
        DVec3::new(0.0625, f64::from(size.y), f64::from(size.z))
    } else {
        DVec3::new(f64::from(size.x), f64::from(size.y), 0.0625)
    };
    Aabb::new(center - size / 2.0, center + size / 2.0)
}

fn inline_painting_hitbox_bounds(inline: &PaintingVariantDefinition) -> UVec3 {
    let width = u32::try_from(inline.width)
        .ok()
        .filter(|&n| n > 0)
        .unwrap_or(1);
    let height = u32::try_from(inline.height)
        .ok()
        .filter(|&n| n > 0)
        .unwrap_or(1);
    UVec3::new(width, height, width)
}
