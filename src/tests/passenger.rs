use bevy_ecs::prelude::*;
use chunkedge_server::entity::pig::PigEntity;
use chunkedge_server::entity::{EntityId, EntityLayerId, Position};
use chunkedge_server::interact_entity::{EntityInteraction, InteractEntityMessage};
use chunkedge_server::passenger::{Passengers, Riding};
use chunkedge_server::protocol::packets::play::{InteractC2s, SetPassengersS2c};
use chunkedge_server::protocol::VarInt;
use chunkedge_server::Hand;

use crate::testing::ScenarioSingleClient;

#[test]
fn passengers_broadcast_on_mount_and_dismount() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        layer,
    } = ScenarioSingleClient::new();

    app.update();
    helper.clear_received();

    let vehicle = app
        .world_mut()
        .spawn((
            PigEntity,
            EntityLayerId(layer),
            Position::new([0.0, 64.0, 0.0]),
        ))
        .id();

    app.update();
    helper.clear_received();

    app.world_mut().entity_mut(client).insert(Riding(vehicle));

    app.update();

    assert!(
        app.world().get::<Passengers>(vehicle).is_some(),
        "vehicle should have a Passengers component after mounting"
    );

    let recvd = helper.collect_received();
    recvd.assert_count::<SetPassengersS2c>(1);
    let pkt = recvd.first::<SetPassengersS2c>();
    assert_eq!(pkt.passengers.len(), 1, "vehicle should have one passenger");
    assert_eq!(
        pkt.passengers[0].0, 0,
        "the riding client should see its own entity as ID 0 so it mounts itself"
    );

    helper.clear_received();

    app.world_mut().entity_mut(client).remove::<Riding>();

    app.update();

    assert!(
        app.world().get::<Passengers>(vehicle).is_none(),
        "vehicle should lose its Passengers component after the last rider leaves"
    );

    let recvd = helper.collect_received();
    recvd.assert_count::<SetPassengersS2c>(1);
    let pkt = recvd.first::<SetPassengersS2c>();
    assert_eq!(
        pkt.passengers.len(),
        0,
        "an empty passenger list should be sent on dismount"
    );
}

#[test]
fn passengers_sent_when_vehicle_enters_view() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        layer,
    } = ScenarioSingleClient::new();

    app.update();
    helper.clear_received();

    let far = [1000.0, 64.0, 1000.0];

    let vehicle = app
        .world_mut()
        .spawn((PigEntity, EntityLayerId(layer), Position::new(far)))
        .id();
    let rider = app
        .world_mut()
        .spawn((PigEntity, EntityLayerId(layer), Position::new(far)))
        .id();
    app.world_mut().entity_mut(rider).insert(Riding(vehicle));

    app.update();
    helper.clear_received();

    helper
        .collect_received()
        .assert_count::<SetPassengersS2c>(0);

    *app.world_mut().get_mut::<Position>(client).unwrap() = Position::new(far);

    app.update();
    app.update();

    let recvd = helper.collect_received();
    recvd.assert_count::<SetPassengersS2c>(1);
    let pkt = recvd.first::<SetPassengersS2c>();
    assert_eq!(
        pkt.passengers.len(),
        1,
        "the vehicle should report its rider when it enters a client's view"
    );
}

#[derive(Component)]
struct Vehicle;

fn toggle_ride(
    mut commands: Commands,
    mut messages: MessageReader<InteractEntityMessage>,
    vehicles: Query<(), With<Vehicle>>,
    riders: Query<Has<Riding>>,
) {
    for message in messages.read() {
        if !matches!(message.interact, EntityInteraction::Interact(_)) {
            continue;
        }
        if vehicles.get(message.entity).is_err() {
            continue;
        }
        match riders.get(message.client) {
            Ok(true) => {
                commands.entity(message.client).remove::<Riding>();
            }
            Ok(false) => {
                commands
                    .entity(message.client)
                    .insert(Riding(message.entity));
            }
            Err(_) => {}
        }
    }
}

#[test]
fn right_click_interaction_mounts_the_client() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        layer,
    } = ScenarioSingleClient::new();

    app.add_systems(bevy_app::Update, toggle_ride);

    app.update();
    helper.confirm_initial_pending_teleports();
    helper.clear_received();

    let vehicle = app
        .world_mut()
        .spawn((
            PigEntity,
            EntityLayerId(layer),
            Position::new([0.0, 64.0, 0.0]),
            Vehicle,
        ))
        .id();

    app.update();

    let vehicle_id = app.world().get::<EntityId>(vehicle).unwrap().get();

    helper.send(&InteractC2s {
        entity_id: VarInt(vehicle_id),
        interact: EntityInteraction::Interact(Hand::Main),
        sneaking: false,
    });

    app.update();

    assert!(
        app.world().get::<Riding>(client).is_some(),
        "client should be riding the vehicle after a right-click interaction"
    );
}
