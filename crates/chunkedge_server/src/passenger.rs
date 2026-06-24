//! Passenger/riding API built on Bevy's entity relationships.
//!
//! Insert [`Riding`] on a passenger entity to mount it onto a vehicle; the
//! vehicle's [`Passengers`] component and the clients viewing it are kept in
//! sync automatically.

use std::borrow::Cow;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::relationship::RelationshipTarget;
use chunkedge_entity::{EntityId, EntityLayerId, Position};
use derive_more::Deref;

use crate::client::{
    Client, FlushPacketsSet, LoadEntityForClientEvent, ViewDistance, VisibleEntityLayers,
};
use crate::layer::UpdateLayersPreClientSet;
use crate::protocol::packets::play::SetPassengersS2c;
use crate::protocol::{VarInt, WritePacket};
use crate::{ChunkView, EntityLayer, Layer};

pub struct PassengerPlugin;

impl Plugin for PassengerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                (send_passengers, send_empty_passengers_on_removal)
                    .before(UpdateLayersPreClientSet),
                send_passengers_on_entity_load.before(FlushPacketsSet),
            ),
        );
    }
}

/// Placed on a passenger entity, pointing at the vehicle it rides.
///
/// Inserting this component mounts the entity; removing it (or despawning the
/// entity) dismounts it. The vehicle's [`Passengers`] component is kept in sync
/// automatically.
#[derive(Component, Copy, Clone, PartialEq, Eq, Debug, Deref)]
#[relationship(relationship_target = Passengers)]
pub struct Riding(pub Entity);

/// Automatically maintained on a vehicle entity; lists the entities currently
/// riding it.
///
/// This is populated by inserting
/// [`Riding`] on passenger entities.
/// You can access the passenger entities using `.iter()`
#[derive(Component, Debug)]
#[relationship_target(relationship = Riding)]
pub struct Passengers(Vec<Entity>);

fn send_passengers(
    vehicles: Query<(&EntityId, &Passengers, &EntityLayerId, &Position), Changed<Passengers>>,
    entity_ids: Query<&EntityId>,
    mut clients: Query<(
        &mut Client,
        &EntityId,
        &Position,
        &ViewDistance,
        &VisibleEntityLayers,
    )>,
) {
    for (vehicle_id, passengers, layer_id, vehicle_pos) in &vehicles {
        for (mut client, client_id, client_pos, view_distance, visible_layers) in &mut clients {
            if !visible_layers.0.contains(&layer_id.0) {
                continue;
            }

            let view = ChunkView::new(client_pos.0.into(), view_distance.get());
            if !view.contains(vehicle_pos.0.into()) {
                continue;
            }

            client.write_packet(&packet_for_viewer(
                client_id.get(),
                vehicle_id.get(),
                passengers,
                &entity_ids,
            ));
        }
    }
}

fn send_empty_passengers_on_removal(
    mut removed: RemovedComponents<Passengers>,
    vehicles: Query<(&EntityId, &EntityLayerId, &Position), Without<Passengers>>,
    mut entity_layers: Query<&mut EntityLayer>,
) {
    for entity in removed.read() {
        let Ok((vehicle_id, layer_id, pos)) = vehicles.get(entity) else {
            continue;
        };
        let Ok(mut entity_layer) = entity_layers.get_mut(layer_id.0) else {
            continue;
        };

        let packet = SetPassengersS2c {
            entity_id: VarInt(vehicle_id.get()),
            passengers: Cow::Borrowed(&[]),
        };

        entity_layer.view_writer(pos.0).write_packet(&packet);
    }
}

fn send_passengers_on_entity_load(
    mut events: MessageReader<LoadEntityForClientEvent>,
    mut clients: Query<(&mut Client, &EntityId)>,
    vehicles: Query<(&EntityId, &Passengers)>,
    entity_ids: Query<&EntityId>,
) {
    for event in events.read() {
        let Ok((vehicle_id, passengers)) = vehicles.get(event.entity_loaded) else {
            continue;
        };
        let Ok((mut client, client_id)) = clients.get_mut(event.client) else {
            continue;
        };

        client.write_packet(&packet_for_viewer(
            client_id.get(),
            vehicle_id.get(),
            passengers,
            &entity_ids,
        ));
    }
}

fn packet_for_viewer<'a>(
    viewer_id: i32,
    vehicle_id: i32,
    passengers: &Passengers,
    entity_ids: &Query<&EntityId>,
) -> SetPassengersS2c<'a> {
    let ids: Vec<VarInt> = passengers
        .iter()
        .filter_map(|passenger| entity_ids.get(passenger).ok())
        .map(|id| VarInt(rewrite_self(id.get(), viewer_id)))
        .collect();

    SetPassengersS2c {
        entity_id: VarInt(rewrite_self(vehicle_id, viewer_id)),
        passengers: Cow::Owned(ids),
    }
}

fn rewrite_self(id: i32, viewer_id: i32) -> i32 {
    if id == viewer_id {
        0
    } else {
        id
    }
}
