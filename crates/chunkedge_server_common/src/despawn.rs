use bevy_ecs::prelude::*;

/// A marker [`Component`] for entities that should be despawned at the end of
/// the tick.
///
/// In ChunkEdge, some entities such as Minecraft entities must not be removed
/// from the [`World`] directly. ChunkEdge needs an opportunity to perform
/// deinitialization work while the entity's components still exist.
///
/// To resolve this problem, you must give the entities you wish to despawn the
/// `Despawned` component. At the end of the tick, ChunkEdge will despawn all
/// entities with this component for you.
///
/// The `Despawned` component can be used on entities that ChunkEdge does not
/// know about. The entity will be despawned regardless.
#[derive(Component, Copy, Clone, Default, PartialEq, Eq, Debug)]
pub struct Despawned;

pub(super) fn despawn_marked_entities(
    entities: Query<Entity, With<Despawned>>,
    mut commands: Commands,
) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
