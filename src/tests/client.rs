use chunkedge_server::protocol::movement_flags::MovementFlags;

use crate::abilities::PlayerAbilitiesFlags;
use crate::client::Client;
use crate::entity::EntityId;
use crate::entity::avatar::PlayerModeCustomisation;
use crate::layer::ChunkLayer;
use crate::layer::chunk::UnloadedChunk;
use crate::math::DVec3;
use crate::protocol::packets::play::{
    AcceptTeleportationC2s, LoginS2c, MoveEntityPosS2c, MovePlayerPosRotC2s, PlayerCombatKillS2c,
    PlayerPositionS2c, SetEntityDataS2c,
};
use crate::protocol::{Packet, VarInt};
use crate::testing::{ScenarioSingleClient, create_mock_client};
use crate::{ChunkPos, GameMode};

#[test]
fn client_teleport_and_move() {
    let ScenarioSingleClient {
        mut app,
        helper: mut helper_1,
        layer: layer_ent,
        ..
    } = ScenarioSingleClient::new();

    let mut layer = app.world_mut().get_mut::<ChunkLayer>(layer_ent).unwrap();

    for z in -10..10 {
        for x in -10..10 {
            layer.insert_chunk(ChunkPos::new(x, z), UnloadedChunk::new());
        }
    }

    let (mut bundle, mut helper_2) = create_mock_client(
        "other",
        &mut app
            .world_mut()
            .resource_mut::<chunkedge_server::entity::EntityManager>(),
    );

    bundle.layer.0 = layer_ent;
    bundle.visible_chunk_layer.0 = layer_ent;
    bundle.visible_entity_layers.0.insert(layer_ent);

    app.world_mut().spawn(bundle);

    app.update();

    // Client received an initial teleport.
    helper_1
        .collect_received()
        .assert_count::<PlayerPositionS2c>(1);

    // Confirm the initial teleport from the server.
    helper_1.send(&AcceptTeleportationC2s {
        teleport_id: 0.into(),
    });

    // Move a little.
    helper_1.send(&MovePlayerPosRotC2s {
        position: DVec3::new(1.0, 0.0, 0.0),
        yaw: 0.0,
        pitch: 0.0,
        flags: MovementFlags::new().with_on_ground(true),
    });

    app.update();

    // Check that the other client saw the client moving.
    helper_2
        .collect_received()
        .assert_count::<MoveEntityPosS2c>(1);
}

#[test]
fn client_gamemode_changed_ability() {
    let mut scenario = ScenarioSingleClient::new();

    *scenario
        .app
        .world_mut()
        .get_mut::<GameMode>(scenario.client)
        .unwrap() = GameMode::Creative;

    scenario.app.update();

    let abilities = scenario
        .app
        .world_mut()
        .get::<PlayerAbilitiesFlags>(scenario.client)
        .unwrap();

    assert!(abilities.allow_flying());
    assert!(abilities.instant_break());
    assert!(abilities.invulnerable());

    *scenario
        .app
        .world_mut()
        .get_mut::<GameMode>(scenario.client)
        .unwrap() = GameMode::Adventure;

    scenario.app.update();

    let abilities = scenario
        .app
        .world_mut()
        .get::<PlayerAbilitiesFlags>(scenario.client)
        .unwrap();

    assert!(!abilities.allow_flying());
    assert!(!abilities.instant_break());
    assert!(!abilities.invulnerable());
}

#[test]
// Regression test for a scheduling race where derived player tracked data
// could be updated after tracked-data serialization during the initial join.
// That left a SetEntityDataS2c packet to be emitted on the next tick, which
// made tests that expected no unrelated packets after join flaky.
fn client_does_not_emit_delayed_tracked_data_after_initial_join() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    app.update();
    helper.clear_received();

    app.update();

    helper
        .collect_received()
        .assert_count::<SetEntityDataS2c>(0);
}

#[test]
fn client_knows_itself_by_its_entity_id() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = ScenarioSingleClient::new();

    app.update();
    let entity_id = app.world().get::<EntityId>(client).unwrap().get();
    assert_ne!(entity_id, 0, "ID 0 means unassigned to the client");

    let recvd = helper.collect_received();
    recvd.assert_count::<LoginS2c>(1);
    assert_eq!(recvd.first::<LoginS2c>().entity_id, entity_id);

    app.world_mut()
        .get_mut::<Client>(client)
        .unwrap()
        .kill("test");
    app.update();

    let recvd = helper.collect_received();
    recvd.assert_count::<PlayerCombatKillS2c>(1);
    assert_eq!(
        recvd.first::<PlayerCombatKillS2c>().player_id,
        VarInt(entity_id)
    );
}

#[test]
// Regression test: the extracted default for the model-part mask was 127
// (taken from a mannequin), so a player showing every skin layer had that
// value dropped from its initial tracked data and others saw no layers.
fn client_skin_layers_are_sent_to_other_clients() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        layer: layer_ent,
        ..
    } = ScenarioSingleClient::new();

    app.update();
    helper.clear_received();

    let (mut bundle, _helper_2) = create_mock_client(
        "other",
        &mut app
            .world_mut()
            .resource_mut::<chunkedge_server::entity::EntityManager>(),
    );
    bundle.layer.0 = layer_ent;
    bundle.visible_chunk_layer.0 = layer_ent;
    bundle.visible_entity_layers.0.insert(layer_ent);
    bundle.player_model_parts = PlayerModeCustomisation(0x7f);
    let other = app.world_mut().spawn(bundle).id();

    app.update();

    let other_id = app.world().get::<EntityId>(other).unwrap().get();
    let recvd = helper.collect_received();
    let sent_all_layers = recvd
        .0
        .iter()
        .filter(|frame| frame.id == SetEntityDataS2c::ID)
        .map(|frame| frame.decode::<SetEntityDataS2c>().unwrap())
        .filter(|pkt| pkt.entity_id.0 == other_id)
        // Index 16 (Avatar model parts), byte serializer (0), every layer.
        .any(|pkt| pkt.tracked_values.0.windows(3).any(|w| w == [16, 0, 0x7f]));

    assert!(sent_all_layers, "model-part mask missing from tracked data");
}
