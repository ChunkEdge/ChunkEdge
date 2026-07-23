use crate::protocol::packets::play::GameEventS2c;
use crate::protocol::packets::play::game_event_s2c::GameEventKind;
use crate::testing::*;
use crate::weather::{Rain, Thunder, WeatherBundle};

#[test]
fn test_client_initialization_on_join() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        ..
    } = prepare(true);

    app.update();

    // Check if three game state change packets were sent
    // 1. GameEventKind::StartWaitingForLevelChunks
    // 2. GameEventKind::RainLevelChange
    // 3. GameEventKind::ThunderLevel

    let frames = helper.collect_received();
    let game_event_frames = frames
        .0
        .iter()
        .filter_map(|f| f.decode::<GameEventS2c>().ok())
        .collect::<Vec<_>>();

    assert_eq!(
        game_event_frames[0],
        GameEventS2c {
            kind: GameEventKind::StartWaitingForLevelChunks,
            value: 0.0,
        }
    );

    // Check that we have rain and thunder packets in any order
    let rain_packet = GameEventS2c {
        kind: GameEventKind::RainLevelChange,
        value: 0.5,
    };
    let thunder_packet = GameEventS2c {
        kind: GameEventKind::ThunderLevelChange,
        value: 0.5,
    };

    assert!(
        game_event_frames[1..].contains(&rain_packet),
        "Missing rain packet"
    );
    assert!(
        game_event_frames[1..].contains(&thunder_packet),
        "Missing thunder packet"
    );

    frames.assert_count::<GameEventS2c>(3);
}

#[test]
fn test_chunk_layer_initialization_on_join() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        ..
    } = prepare(false);
    // When client_weather is false, the weather is only visible for this client

    app.update();

    // Check if three game state change packets were sent
    // 1. GameEventKind::StartWaitingForLevelChunks
    // 2. GameEventKind::RainLevelChange
    // 3. GameEventKind::ThunderLevel

    let frames = helper.collect_received();
    let game_event_frames = frames
        .0
        .iter()
        .filter_map(|f| f.decode::<GameEventS2c>().ok())
        .collect::<Vec<_>>();

    assert_eq!(
        game_event_frames[0],
        GameEventS2c {
            kind: GameEventKind::StartWaitingForLevelChunks,
            value: 0.0,
        }
    );

    // The order of the rain and thunder packet is non-deterministic if applied to
    // the chunk layer (for some reason)
    let rain_packet = GameEventS2c {
        kind: GameEventKind::RainLevelChange,
        value: 0.5,
    };
    let thunder_packet = GameEventS2c {
        kind: GameEventKind::ThunderLevelChange,
        value: 0.5,
    };

    assert!(
        game_event_frames[1..].contains(&rain_packet),
        "Missing rain packet"
    );
    assert!(
        game_event_frames[1..].contains(&thunder_packet),
        "Missing thunder packet"
    );

    frames.assert_count::<GameEventS2c>(3);
}

#[test]
fn test_client_rain_change() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = prepare(true);

    app.update();

    helper.clear_received();

    // Change the rain value
    let mut rain = app.world_mut().get_mut::<Rain>(client).unwrap();
    rain.0 = 1.0;

    app.update();

    // Check if a game state change packet was sent
    let frames = helper.collect_received();
    frames.assert_count::<GameEventS2c>(1);
}

#[test]
fn test_client_thunder_change() {
    let ScenarioSingleClient {
        mut app,
        client,
        mut helper,
        ..
    } = prepare(true);

    app.update();

    helper.clear_received();

    // Change the thunder value
    let mut thunder = app.world_mut().get_mut::<Thunder>(client).unwrap();
    thunder.0 = 1.0;

    app.update();

    // Check if a game state change packet was sent
    let frames = helper.collect_received();
    frames.assert_count::<GameEventS2c>(1);
}

#[test]
fn test_chunk_layer_rain_change() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        layer,
        ..
    } = prepare(false);

    app.update();

    helper.clear_received();

    // Change the rain value
    let mut rain = app.world_mut().get_mut::<Rain>(layer).unwrap();
    rain.0 = 1.0;

    app.update();

    // Check if a game state change packet was sent
    let frames = helper.collect_received();
    frames.assert_count::<GameEventS2c>(1);
}

#[test]
fn test_chunk_layer_thunder_change() {
    let ScenarioSingleClient {
        mut app,
        mut helper,
        layer,
        ..
    } = prepare(false);

    app.update();

    helper.clear_received();

    // Change the thunder value
    let mut thunder = app.world_mut().get_mut::<Thunder>(layer).unwrap();
    thunder.0 = 1.0;

    app.update();

    // Check if a game state change packet was sent
    let frames = helper.collect_received();
    frames.assert_count::<GameEventS2c>(1);
}

fn prepare(client_weather: bool) -> ScenarioSingleClient {
    let mut s = ScenarioSingleClient::new();

    // Process a tick to get past the "on join" logic.
    s.app.update();

    // Add weather to either the client or the chunk layer depending on the
    // parameter
    if client_weather {
        add_weather_to_client(&mut s);
    } else {
        add_weather_to_chunk_layer(&mut s);
    }

    s
}

fn add_weather_to_client(s: &mut ScenarioSingleClient) {
    s.app
        .world_mut()
        .entity_mut(s.client)
        .insert(WeatherBundle {
            rain: Rain(0.5),
            thunder: Thunder(0.5),
        });
}

fn add_weather_to_chunk_layer(s: &mut ScenarioSingleClient) {
    s.app.world_mut().entity_mut(s.layer).insert(WeatherBundle {
        rain: Rain(0.5),
        thunder: Thunder(0.5),
    });
}
