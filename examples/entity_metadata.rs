use std::collections::HashMap;

use valence::entity::breeze::BreezeEntityBundle;
use valence::entity::cat::{self, CatEntityBundle};
use valence::entity::enderman::{self, EndermanEntityBundle};
use valence::entity::frog::FrogEntityBundle;
use valence::entity::painting::{self, PaintingEntityBundle};
use valence::entity::player::PlayerEntityBundle;
use valence::entity::warden::WardenEntityBundle;
use valence::entity::zombie::ZombieEntityBundle;
use valence::entity::{
    entity, CatKind, EntityLayerId, ObjectData, OnGround, PaintingKind, PaintingVariantDefinition,
    Pose,
};
use valence::nbt::{compound, List};
use valence::player_list::{Listed, PlayerListEntryBundle};
use valence::prelude::*;
use valence::protocol::IdOr;

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
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Zombie), Pose::Standing, false),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Zombie), Pose::FallFlying, true),
    MetadataCase::pose(PoseEntity::PlayerNpc, Pose::Sleeping, false),
    MetadataCase::pose(PoseEntity::PlayerNpc, Pose::Swimming, true),
    MetadataCase::pose(PoseEntity::PlayerNpc, Pose::Sneaking, false),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Breeze), Pose::LongJumping, true),
    MetadataCase::pose(PoseEntity::PlayerNpc, Pose::Dying, false),
    MetadataCase::pose(PoseEntity::PlayerNpc, Pose::Sitting, false),
    MetadataCase::pose(PoseEntity::PlayerNpc, Pose::SpinAttack, false),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Frog), Pose::Croaking, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Frog), Pose::UsingTongue, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Warden), Pose::Roaring, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Warden), Pose::Sniffing, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Warden), Pose::Emerging, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Warden), Pose::Digging, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Breeze), Pose::Sliding, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Breeze), Pose::Shooting, true),
    MetadataCase::pose(PoseEntity::Mob(MobDemo::Breeze), Pose::Inhaling, true),
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

#[derive(Clone, Copy, Debug)]
enum PoseEntity {
    Mob(MobDemo),
    PlayerNpc,
}

impl PoseEntity {
    fn label(self) -> String {
        match self {
            Self::Mob(mob) => format!("{mob:?}"),
            Self::PlayerNpc => "Player".into(),
        }
    }
}

#[derive(Clone, Debug)]
enum MetadataCase {
    Pose {
        entity: PoseEntity,
        pose: Pose,
        has_pressure_plate: bool,
    },
    Cat(CatKind),
    Painting(IdOr<PaintingVariantDefinition>),
    Enderman(Option<BlockState>),
}

impl MetadataCase {
    const fn pose(entity: PoseEntity, pose: Pose, has_pressure_plate: bool) -> Self {
        Self::Pose {
            entity,
            pose,
            has_pressure_plate,
        }
    }

    fn has_pressure_plate(&self) -> bool {
        match self {
            Self::Pose {
                has_pressure_plate, ..
            } => *has_pressure_plate,
            _ => false,
        }
    }

    fn sign_description(&self) -> (String, String, String, String) {
        match self {
            Self::Pose {
                entity,
                pose,
                has_pressure_plate,
            } => (
                "Pose".into(),
                entity.label(),
                format!("{pose:?}"),
                if *has_pressure_plate {
                    "Step to reset".into()
                } else {
                    "Static".into()
                },
            ),
            other => (
                "Metadata".into(),
                format!("{other:?}")
                    .split_terminator("(")
                    .next()
                    .unwrap()
                    .into(),
                match other {
                    MetadataCase::Cat(variant) => format!("{variant:?}"),
                    MetadataCase::Painting(variant) => match variant {
                        IdOr::Id(id) => {
                            format!(
                                "{:?}",
                                PaintingKind::from_registry_id(*id).expect("Wrong paining ID")
                            )
                        }
                        IdOr::Inline(def) => def
                            .asset_id
                            .split(':')
                            .next_back()
                            .unwrap_or(&def.asset_id)
                            .into(),
                    },
                    MetadataCase::Enderman(block) => {
                        if let Some(block) = block {
                            format!("{block:?}")
                        } else {
                            "None".into()
                        }
                    }
                    MetadataCase::Pose { .. } => {
                        unreachable!()
                    }
                },
                {
                    if let MetadataCase::Painting(IdOr::Inline(inline)) = other {
                        format!("inline {}x{}", inline.width, inline.height)
                    } else {
                        "Static sample".into()
                    }
                },
            ),
        }
    }

    fn sign_lines(&self) -> [Compound<String>; 4] {
        let (line_1, line_2, line_3, line_4) = self.sign_description();

        [
            line_1.color(Color::DARK_GREEN).bold().into(),
            line_2.color(Color::BLUE).bold().into(),
            line_3.into_text().into(),
            line_4.into_text().into(),
        ]
    }

    fn spawn(
        self,
        commands: &mut Commands,
        station: &mut MetadataStation,
        layer: EntityLayerId,
    ) -> Entity {
        match self {
            Self::Pose { entity, pose, .. } => match entity {
                PoseEntity::Mob(mob) => mob.spawn(commands, station.spawn_pos, layer, pose),
                PoseEntity::PlayerNpc => spawn_player_npc(commands, station, layer, pose),
            },
            Self::Cat(variant) => spawn_cat_variant(commands, station.spawn_pos, layer, variant),
            Self::Painting(variant) => {
                spawn_painting_variant(commands, station.spawn_pos, layer, variant)
            }
            Self::Enderman(has_carried_block) => {
                spawn_enderman(commands, station.spawn_pos, layer, has_carried_block)
            }
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

struct StationLayout {
    plate: BlockPos,
    sign: [i32; 3],
    spawn_block: BlockPos,
    spawn: Position,
}

fn station_rows(case_count: usize) -> i32 {
    (case_count as i32 + GRID_COLUMNS - 1) / GRID_COLUMNS
}

fn station_layout(index: i32) -> StationLayout {
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

    StationLayout {
        plate: plate_pos,
        sign: sign_pos,
        spawn_block: spawn_block_pos,
        spawn: spawn_pos,
    }
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
) {
    let other_cases: &[MetadataCase] = &[
        MetadataCase::Cat(CatKind::AllBlack),
        MetadataCase::Cat(CatKind::Tabby),
        MetadataCase::Painting(IdOr::id(PaintingKind::Aztec.registry_id())),
        MetadataCase::Painting(IdOr::id(PaintingKind::Bouquet.registry_id())),
        MetadataCase::Painting(IdOr::inline(PaintingVariantDefinition {
            width: 2,
            height: 1,
            asset_id: "minecraft:fighters".to_owned(),
            title: Some("Inline Pool".into()),
            author: Some("Valence Example".into()),
        })),
        MetadataCase::Enderman(Some(BlockState::DIAMOND_BLOCK)),
        MetadataCase::Enderman(None),
    ];

    let station_cases: Vec<_> = POSE_CASES
        .iter()
        .chain(other_cases.iter())
        .cloned()
        .collect();

    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    let rows = station_rows(station_cases.len());

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
    let mut stations = Vec::with_capacity(station_cases.len());

    for (index, case) in station_cases.iter().cloned().enumerate() {
        let layout = station_layout(index as i32);
        let lines = case.sign_lines();
        layer.chunk.set_block(
            layout.sign,
            Block {
                state: BlockState::OAK_SIGN.set(PropName::Rotation, PropValue::_8),
                nbt: Some(compound! {
                    "front_text" => compound! {
                        "messages" => List::Compound(lines.to_vec())
                    }
                }),
            },
        );

        if case.has_pressure_plate() {
            layer
                .chunk
                .set_block(layout.plate, BlockState::STONE_PRESSURE_PLATE);
            by_plate_xz.insert((layout.plate.x, layout.plate.z), stations.len());
        }

        layer
            .chunk
            .set_block(layout.spawn_block, BlockState::GOLD_BLOCK);

        stations.push(MetadataStation {
            case,
            spawn_pos: layout.spawn,
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

    commands.insert_resource(metadata_stations);
}

fn spawn_cat_variant(
    commands: &mut Commands,
    position: Position,
    layer: EntityLayerId,
    variant: CatKind,
) -> Entity {
    commands
        .spawn(CatEntityBundle {
            layer,
            position,
            look: Look::new(DEMO_ENTITY_YAW, 0.0),
            head_yaw: HeadYaw(DEMO_ENTITY_YAW),
            cat_cat_variant: cat::CatVariant(variant),
            ..Default::default()
        })
        .id()
}

fn spawn_painting_variant(
    commands: &mut Commands,
    position: Position,
    layer: EntityLayerId,
    variant: IdOr<PaintingVariantDefinition>,
) -> Entity {
    commands
        .spawn(PaintingEntityBundle {
            layer,
            position: Position::new((position.0.x, position.0.y + 1.0, position.0.z)),
            object_data: ObjectData(2),
            painting_variant: painting::Variant(variant),
            ..Default::default()
        })
        .id()
}

fn spawn_enderman(
    commands: &mut Commands,
    position: Position,
    layer: EntityLayerId,
    block: Option<BlockState>,
) -> Entity {
    let carried_block;
    if let Some(block) = block {
        carried_block = Some(block);
    } else {
        carried_block = None;
    }

    commands
        .spawn(EndermanEntityBundle {
            layer,
            position,
            look: Look::new(DEMO_ENTITY_YAW, 0.0),
            head_yaw: HeadYaw(DEMO_ENTITY_YAW),
            enderman_carried_block: enderman::CarriedBlock(carried_block),
            ..Default::default()
        })
        .id()
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

        client.send_chat_message("Entity metadata demo:");

        client.send_chat_message("Stations with pressure plates respawn the entity.");

        client.send_chat_message(
            "Stations with no pressure plates would display nothing extra when respawned.",
        );

        client.send_chat_message(
            "Dying, Sitting and SpinAttack are known to not display correctly in this demo due to \
             additional required metadata that is not set. This is expected."
                .color(Color::RED)
                .bold(),
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

    let spawned_entity = station.case.clone().spawn(commands, station, layer);

    station.spawned_entity = Some(spawned_entity);
}

fn spawn_player_npc(
    commands: &mut Commands,
    station: &mut MetadataStation,
    layer: EntityLayerId,
    pose: Pose,
) -> Entity {
    if station.player_npc.is_none() {
        let uuid = UniqueId::default();

        commands.spawn(PlayerListEntryBundle {
            uuid,
            username: Username(format!("!_{pose:?}_!")),
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
            entity_pose: entity::Pose(pose),
            ..Default::default()
        })
        .id()
}
