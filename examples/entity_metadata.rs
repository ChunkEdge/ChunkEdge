use std::collections::HashMap;

use valence::entity::breeze::BreezeEntityBundle;
use valence::entity::cat::{self, CatEntityBundle};
use valence::entity::enderman::{self, EndermanEntityBundle};
use valence::entity::frog::FrogEntityBundle;
use valence::entity::painting::{self, PaintingEntityBundle};
use valence::entity::player::PlayerEntityBundle;
use valence::entity::warden::WardenEntityBundle;
use valence::entity::zombie::ZombieEntityBundle;
use valence::entity::{entity, CatKind, EntityLayerId, OnGround, PaintingKind, Pose};
use valence::nbt::{compound, List};
use valence::player_list::{Listed, PlayerListEntryBundle};
use valence::prelude::*;

const FLOOR_Y: i32 = 64;
const GRID_COLUMNS: i32 = 6;
const CELL_WIDTH: i32 = 4;
const CELL_DEPTH: i32 = 7;
const GRID_ORIGIN_X: i32 = -9;
const GRID_ORIGIN_Z: i32 = -16;
const GRID_MARGIN: i32 = 2;
const DEMO_ENTITY_YAW: f32 = 180.0;

/// Should have one for each pose in the [Pose] enum
const POSE_CASES: &[MetadataCase] = &[
    // All poses that do not play an animation have had their pressure plate disabled
    MetadataCase::new(EntityDemo::Mob(MobDemo::Zombie), Pose::Standing, false),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Zombie), Pose::FallFlying, true),
    MetadataCase::new(EntityDemo::PlayerNpc, Pose::Sleeping, false),
    MetadataCase::new(EntityDemo::PlayerNpc, Pose::Swimming, true),
    MetadataCase::new(EntityDemo::PlayerNpc, Pose::Sneaking, false),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Breeze), Pose::LongJumping, true),
    MetadataCase::new(EntityDemo::PlayerNpc, Pose::Dying, false),
    MetadataCase::new(EntityDemo::PlayerNpc, Pose::Sitting, false),
    MetadataCase::new(EntityDemo::PlayerNpc, Pose::SpinAttack, false),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Frog), Pose::Croaking, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Frog), Pose::UsingTongue, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Warden), Pose::Roaring, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Warden), Pose::Sniffing, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Warden), Pose::Emerging, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Warden), Pose::Digging, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Breeze), Pose::Sliding, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Breeze), Pose::Shooting, true),
    MetadataCase::new(EntityDemo::Mob(MobDemo::Breeze), Pose::Inhaling, true),
];

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                reset_station_entities_on_plate,
                despawn_disconnected_clients,
            ),
        )
        .run();
}

#[derive(Clone, Copy, Debug)]
enum MobDemo {
    Breeze,
    Zombie,
    Frog,
    Warden,
}

impl MobDemo {
    fn spawn(
        self,
        commands: &mut Commands,
        position: Position,
        layer: EntityLayerId,
        pose: Pose,
    ) -> Entity {
        macro_rules! spawn_bundle {
            ($bundle:ident) => {
                commands
                    .spawn($bundle {
                        position,
                        layer,
                        look: Look::new(DEMO_ENTITY_YAW, 0.0),
                        head_yaw: HeadYaw(DEMO_ENTITY_YAW),
                        entity_pose: entity::Pose(pose),
                        ..Default::default()
                    })
                    .id()
            };
        }

        match self {
            Self::Breeze => spawn_bundle!(BreezeEntityBundle),
            Self::Frog => spawn_bundle!(FrogEntityBundle),
            Self::Warden => spawn_bundle!(WardenEntityBundle),
            Self::Zombie => spawn_bundle!(ZombieEntityBundle),
        }
    }
}

#[derive(Clone, Copy)]
enum EntityDemo {
    Mob(MobDemo),
    PlayerNpc,
}

impl EntityDemo {
    fn label(self) -> String {
        match self {
            Self::Mob(mob) => format!("{:?}", mob),
            Self::PlayerNpc => "Player".into(),
        }
    }
}

#[derive(Clone, Copy)]
struct MetadataCase {
    entity: EntityDemo,
    pose: Pose,
    has_pressure_plate: bool,
}

impl MetadataCase {
    const fn new(entity: EntityDemo, pose: Pose, has_pressure_plate: bool) -> Self {
        Self {
            entity,
            pose,
            has_pressure_plate,
        }
    }
}

#[derive(Clone, Copy)]
struct PlayerNpcState {
    uuid: UniqueId,
}

struct MetadataStation {
    case: MetadataCase,
    spawn_pos: Position,
    spawned_entity: Option<Entity>,
    player_npc: Option<PlayerNpcState>,
}

#[derive(Resource)]
struct MetadataStations {
    layer: EntityLayerId,
    stations: Vec<MetadataStation>,
    by_plate_xz: HashMap<(i32, i32), usize>,
}

#[derive(Component, Default)]
struct ActivePlate(Option<usize>);

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    let rows = (POSE_CASES.len() as i32 + GRID_COLUMNS - 1) / GRID_COLUMNS;

    let min_x = GRID_ORIGIN_X - GRID_MARGIN;
    let min_z = GRID_ORIGIN_Z - GRID_MARGIN;
    let max_x = GRID_ORIGIN_X + GRID_COLUMNS * CELL_WIDTH + GRID_MARGIN;
    let max_z = GRID_ORIGIN_Z + rows * CELL_DEPTH + GRID_MARGIN;

    for cz in min_z.div_euclid(16) - 1..=max_z.div_euclid(16) + 1 {
        for cx in min_x.div_euclid(16) - 1..=max_x.div_euclid(16) + 1 {
            layer.chunk.insert_chunk([cx, cz], UnloadedChunk::new());
        }
    }

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let on_grid_line = (x - GRID_ORIGIN_X).rem_euclid(CELL_WIDTH) == 0
                || (z - GRID_ORIGIN_Z).rem_euclid(CELL_DEPTH) == 0;
            layer.chunk.set_block(
                [x, FLOOR_Y, z],
                if on_grid_line {
                    BlockState::BLACK_CONCRETE
                } else {
                    BlockState::WHITE_CONCRETE
                },
            );
        }
    }

    let mut by_plate_xz = HashMap::new();
    let mut stations = Vec::with_capacity(POSE_CASES.len());

    for (index, case) in POSE_CASES.iter().copied().enumerate() {
        let index = index as i32;
        let col = index % GRID_COLUMNS;
        let row = index / GRID_COLUMNS;

        let cell_x = GRID_ORIGIN_X + col * CELL_WIDTH;
        let cell_z = GRID_ORIGIN_Z + row * CELL_DEPTH;

        let plate_pos = BlockPos::new(cell_x + (CELL_WIDTH / 2), FLOOR_Y + 1, cell_z + 1);
        let sign_pos = [plate_pos.x, FLOOR_Y + 1, plate_pos.z + 1];
        let spawn_block_pos = BlockPos::new(plate_pos.x, FLOOR_Y, plate_pos.z + 4);
        let spawn_pos = Position::new((
            f64::from(spawn_block_pos.x) + 0.5,
            f64::from(FLOOR_Y) + 1.0,
            f64::from(spawn_block_pos.z) + 0.5,
        ));

        layer.chunk.set_block(
            sign_pos,
            Block {
                state: BlockState::OAK_SIGN.set(PropName::Rotation, PropValue::_8),
                nbt: Some(compound! {
                    "front_text" => compound! {
                        "messages" => List::Compound(vec![
                            case.entity.label().color(Color::DARK_GREEN).into(),
                            format!("{:?}", case.pose).color(Color::BLUE).into(),
                            match case.has_pressure_plate {
                                true => "Step to reset".color(Color::RED).bold().into(),
                                false => "NO START".color(Color::RED).bold().into(),
                            },
                            match case.has_pressure_plate {
                                true => "".into_text().into(),
                                false => "ANIMATION".color(Color::RED).bold().into(),
                            },
                        ])
                    }
                }),
            },
        );

        if case.has_pressure_plate {
            layer
                .chunk
                .set_block(plate_pos, BlockState::STONE_PRESSURE_PLATE);
        }
        layer
            .chunk
            .set_block([plate_pos.x, FLOOR_Y + 1, plate_pos.z + 2], BlockState::AIR);
        layer
            .chunk
            .set_block([plate_pos.x, FLOOR_Y + 1, plate_pos.z + 3], BlockState::AIR);
        layer
            .chunk
            .set_block(spawn_block_pos, BlockState::GOLD_BLOCK);

        by_plate_xz.insert((plate_pos.x, plate_pos.z), stations.len());
        stations.push(MetadataStation {
            case,
            spawn_pos,
            spawned_entity: None,
            player_npc: None,
        });
    }

    let layer_entity = commands.spawn(layer).id();

    let mut metadata_stations = MetadataStations {
        layer: EntityLayerId(layer_entity),
        stations,
        by_plate_xz,
    };

    for station_index in 0..metadata_stations.stations.len() {
        respawn_station_entity(&mut commands, &mut metadata_stations, station_index);
    }

    spawn_metadata_examples(&mut commands, metadata_stations.layer, max_z);

    commands.insert_resource(metadata_stations);
}

fn spawn_metadata_examples(commands: &mut Commands, layer: EntityLayerId, max_z: i32) {
    let showcase_z = max_z - 1;
    let ground_y = f64::from(FLOOR_Y) + 1.0;
    let painting_y = f64::from(FLOOR_Y) + 2.0;
    let x_at = |index: i32| GRID_ORIGIN_X + 2 + index * CELL_WIDTH;

    commands.spawn(CatEntityBundle {
        layer,
        position: Position::new((
            f64::from(x_at(0)) + 0.5,
            ground_y,
            f64::from(showcase_z) + 0.5,
        )),
        look: Look::new(DEMO_ENTITY_YAW, 0.0),
        head_yaw: HeadYaw(DEMO_ENTITY_YAW),
        cat_cat_variant: cat::CatVariant(CatKind::AllBlack),
        entity_custom_name: entity::CustomName(Some("Cat variant: AllBlack".into())),
        entity_name_visible: entity::NameVisible(true),
        ..Default::default()
    });

    commands.spawn(CatEntityBundle {
        layer,
        position: Position::new((
            f64::from(x_at(1)) + 0.5,
            ground_y,
            f64::from(showcase_z) + 0.5,
        )),
        look: Look::new(DEMO_ENTITY_YAW, 0.0),
        head_yaw: HeadYaw(DEMO_ENTITY_YAW),
        cat_cat_variant: cat::CatVariant(CatKind::Tabby),
        entity_custom_name: entity::CustomName(Some("Cat variant: Tabby".into())),
        entity_name_visible: entity::NameVisible(true),
        ..Default::default()
    });

    // commands.spawn(PaintingEntityBundle {
    //     layer,
    //     position: Position::new((
    //         f64::from(x_at(2)) + 0.5,
    //         painting_y,
    //         f64::from(showcase_z) + 0.5,
    //     )),
    //     look: Look::new(DEMO_ENTITY_YAW, 0.0),
    //     head_yaw: HeadYaw(DEMO_ENTITY_YAW),
    //     painting_variant: painting::Variant(PaintingKind::Alban),
    //     entity_custom_name: entity::CustomName(Some("Painting: Alban (1x1)".into())),
    //     entity_name_visible: entity::NameVisible(true),
    //     ..Default::default()
    // });

    // commands.spawn(PaintingEntityBundle {
    //     layer,
    //     position: Position::new((
    //         f64::from(x_at(3)) + 0.5,
    //         painting_y,
    //         f64::from(showcase_z) + 0.5,
    //     )),
    //     look: Look::new(DEMO_ENTITY_YAW, 0.0),
    //     head_yaw: HeadYaw(DEMO_ENTITY_YAW),
    //     painting_variant: painting::Variant(PaintingKind::BurningSkull),
    //     entity_custom_name: entity::CustomName(Some("Painting: BurningSkull (4x4)".into())),
    //     entity_name_visible: entity::NameVisible(true),
    //     ..Default::default()
    // });

    commands.spawn(EndermanEntityBundle {
        layer,
        position: Position::new((
            f64::from(x_at(4)) + 0.5,
            ground_y,
            f64::from(showcase_z) + 0.5,
        )),
        look: Look::new(DEMO_ENTITY_YAW, 0.0),
        head_yaw: HeadYaw(DEMO_ENTITY_YAW),
        enderman_carried_block: enderman::CarriedBlock(Some(BlockState::STONE)),
        entity_custom_name: entity::CustomName(Some("Enderman: carried_block = STONE".into())),
        entity_name_visible: entity::NameVisible(true),
        ..Default::default()
    });

    commands.spawn(EndermanEntityBundle {
        layer,
        position: Position::new((
            f64::from(x_at(5)) + 0.5,
            ground_y,
            f64::from(showcase_z) + 0.5,
        )),
        look: Look::new(DEMO_ENTITY_YAW, 0.0),
        head_yaw: HeadYaw(DEMO_ENTITY_YAW),
        enderman_carried_block: enderman::CarriedBlock(None),
        entity_custom_name: entity::CustomName(Some("Enderman: carried_block = None".into())),
        entity_name_visible: entity::NameVisible(true),
        ..Default::default()
    });
}

fn init_clients(
    mut commands: Commands,
    mut clients: Query<
        (
            Entity,
            &mut Client,
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
        client_entity,
        mut client,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut pos,
        mut game_mode,
    ) in &mut clients
    {
        let layer = layers.single();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);
        pos.set([
            f64::from(GRID_ORIGIN_X + 1),
            f64::from(FLOOR_Y) + 1.0,
            f64::from(GRID_ORIGIN_Z),
        ]);
        *game_mode = GameMode::Creative;

        client.send_chat_message(
            "Entity metadata demo: entities are pre-spawned. Step on any pressure plate to reset that station's entity.",
        );

        client.send_chat_message(
            "Dying, Sitting and SpinAttack are known to not display correctly in this demo due to additional required metadata that is not set. So it is expected behavior that you don't see these displayed correctly.".color(Color::RED).bold(),
        );

        commands
            .entity(client_entity)
            .insert(ActivePlate::default());
    }
}

fn reset_station_entities_on_plate(
    mut commands: Commands,
    mut stations: ResMut<MetadataStations>,
    mut clients: Query<(&Position, &OnGround, &mut ActivePlate), With<Client>>,
) {
    for (position, on_ground, mut active_plate) in &mut clients {
        let x = position.0.x.floor() as i32;
        let z = position.0.z.floor() as i32;

        let current_station = if on_ground.0 {
            stations.by_plate_xz.get(&(x, z)).copied()
        } else {
            None
        };

        if current_station != active_plate.0 {
            if let Some(station_index) = current_station {
                respawn_station_entity(&mut commands, &mut stations, station_index);
            }
            active_plate.0 = current_station;
        }
    }
}

fn respawn_station_entity(
    commands: &mut Commands,
    stations: &mut MetadataStations,
    station_index: usize,
) {
    let layer = stations.layer;
    let station = &mut stations.stations[station_index];

    if let Some(entity) = station.spawned_entity.take() {
        commands.entity(entity).insert(Despawned);
    }

    let spawned_entity = match station.case.entity {
        EntityDemo::Mob(mob) => mob.spawn(commands, station.spawn_pos, layer, station.case.pose),
        EntityDemo::PlayerNpc => spawn_player_npc(commands, station, layer),
    };

    station.spawned_entity = Some(spawned_entity);
}

fn spawn_player_npc(
    commands: &mut Commands,
    station: &mut MetadataStation,
    layer: EntityLayerId,
) -> Entity {
    if station.player_npc.is_none() {
        let uuid = UniqueId::default();

        commands.spawn(PlayerListEntryBundle {
            uuid,
            username: Username(format!("!_{:?}_!", station.case.pose).into()),
            listed: Listed(false),
            ..Default::default()
        });

        station.player_npc = Some(PlayerNpcState { uuid });
    }

    let npc = station.player_npc.unwrap();

    commands
        .spawn(PlayerEntityBundle {
            uuid: npc.uuid,
            layer,
            position: station.spawn_pos,
            look: Look::new(DEMO_ENTITY_YAW, 0.0),
            head_yaw: HeadYaw(DEMO_ENTITY_YAW),
            entity_pose: entity::Pose(station.case.pose),
            ..Default::default()
        })
        .id()
}
