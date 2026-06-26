#![allow(clippy::type_complexity)]

use chunkedge::prelude::*;

type VehicleBundle = chunkedge::entity::pig::PigEntityBundle;
type CowBundle = chunkedge::entity::cow::CowEntityBundle;

const SPAWN_Y: i32 = 64;
const VEHICLE_POS: [f64; 3] = [0.0, SPAWN_Y as f64 + 1.0, 3.0];
const COW_POS: [f64; 3] = [2.0, SPAWN_Y as f64 + 1.0, 3.0];

#[derive(Component)]
struct Vehicle;

#[derive(Component)]
struct Orbit(DVec3);

const ORBIT_RADIUS: f64 = 2.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                mount_on_interact,
                dismount_on_sneak,
                orbit_entities,
                despawn_disconnected_clients,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);

    for z in -5..5 {
        for x in -5..5 {
            layer.chunk.insert_chunk([x, z], UnloadedChunk::new());
        }
    }

    for z in -25..25 {
        for x in -25..25 {
            layer
                .chunk
                .set_block(BlockPos::new(x, SPAWN_Y, z), BlockState::BEDROCK);
        }
    }

    let layer_id = commands.spawn(layer).id();

    commands.spawn((
        VehicleBundle {
            layer: EntityLayerId(layer_id),
            position: Position::new(VEHICLE_POS),
            ..Default::default()
        },
        Vehicle,
        Orbit(VEHICLE_POS.into()),
    ));

    let cow = commands
        .spawn((
            CowBundle {
                layer: EntityLayerId(layer_id),
                position: Position::new(COW_POS),
                ..Default::default()
            },
            Orbit(COW_POS.into()),
        ))
        .id();

    commands.spawn((
        CowBundle {
            layer: EntityLayerId(layer_id),
            position: Position::new(COW_POS),
            ..Default::default()
        },
        Riding(cow),
    ));
}

fn orbit_entities(
    server: Res<Server>,
    mut entities: Query<(&Orbit, &mut Position, &mut Look, &mut HeadYaw)>,
) {
    let angle = server.current_tick() as f64 / f64::from(server.tick_rate().get());

    for (orbit, mut pos, mut look, mut head_yaw) in &mut entities {
        pos.0.x = orbit.0.x + ORBIT_RADIUS * angle.cos();
        pos.0.z = orbit.0.z + ORBIT_RADIUS * angle.sin();
        pos.0.y = orbit.0.y;

        let heading = Vec3::new(-(angle.sin() as f32), 0.0, angle.cos() as f32);
        look.set_vec(heading);
        head_yaw.0 = look.yaw;
    }
}

fn init_clients(
    mut clients: Query<
        (
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut Position,
            &mut GameMode,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, (With<ChunkLayer>, With<EntityLayer>)>,
) {
    for (
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut pos,
        mut game_mode,
    ) in &mut clients
    {
        let layer = layers.single().unwrap();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);
        pos.set([0.0, f64::from(SPAWN_Y) + 1.0, 0.0]);
        *game_mode = GameMode::Creative;
    }
}

fn mount_on_interact(
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

        let Ok(is_riding) = riders.get(message.client) else {
            continue;
        };

        if !is_riding {
            commands
                .entity(message.client)
                .insert(Riding(message.entity));
        }
    }
}

fn dismount_on_sneak(
    mut commands: Commands,
    mut messages: MessageReader<SneakMessage>,
    riders: Query<Has<Riding>>,
) {
    for message in messages.read() {
        if message.state != SneakState::Start {
            continue;
        }

        let Ok(is_riding) = riders.get(message.client) else {
            continue;
        };

        if is_riding {
            commands.entity(message.client).remove::<Riding>();
        }
    }
}
