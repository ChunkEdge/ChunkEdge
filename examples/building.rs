#![allow(clippy::type_complexity)]

use chunkedge::hand_swing::HandSwingWriter;
use chunkedge::interact_block::InteractBlockMessage;
use chunkedge::inventory::HeldItem;
use chunkedge::inventory::player_inventory::PlayerInventory;
use chunkedge::prelude::*;

const SPAWN_Y: i32 = 64;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                init_clients,
                despawn_disconnected_clients,
                toggle_gamemode_on_sneak,
                digging,
                place_blocks,
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

    for z in -50..50 {
        for x in -50..50 {
            layer
                .chunk
                .set_block([x, SPAWN_Y, z], BlockState::GRASS_BLOCK);
        }
    }

    commands.spawn(layer);
}

fn init_clients(
    mut clients: Query<
        (
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
        mut client,
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

        client.send_chat_message("Welcome to ChunkEdge! Build something cool.".italic());
    }
}

fn toggle_gamemode_on_sneak(
    mut clients: Query<&mut GameMode>,
    mut messages: MessageReader<SneakMessage>,
) {
    for message in messages.read() {
        let Ok(mut mode) = clients.get_mut(message.client) else {
            continue;
        };
        if message.state == SneakState::Start {
            *mode = match *mode {
                GameMode::Survival => GameMode::Creative,
                GameMode::Creative => GameMode::Survival,
                _ => GameMode::Creative,
            };
        }
    }
}

fn digging(
    clients: Query<&GameMode>,
    mut layers: Query<&mut ChunkLayer>,
    mut messages: MessageReader<DiggingMessage>,
) {
    let mut layer = layers.single_mut().unwrap();

    for message in messages.read() {
        let Ok(game_mode) = clients.get(message.client) else {
            continue;
        };

        if (*game_mode == GameMode::Creative && message.state == DiggingState::Start)
            || (*game_mode == GameMode::Survival && message.state == DiggingState::Stop)
        {
            layer.set_block(message.position, BlockState::AIR);
        }
    }
}

fn place_blocks(
    mut clients: Query<(&mut Inventory, &GameMode, &HeldItem)>,
    mut layers: Query<&mut ChunkLayer>,
    mut messages: MessageReader<InteractBlockMessage>,
    mut swings: HandSwingWriter,
) {
    let mut layer = layers.single_mut().unwrap();

    for message in messages.read() {
        let Ok((mut inventory, game_mode, held)) = clients.get_mut(message.client) else {
            continue;
        };
        // get the held item
        let slot_id = match message.hand {
            Hand::Main => held.slot(),
            Hand::Off => PlayerInventory::SLOT_OFFHAND,
        };
        let stack = inventory.slot(slot_id);
        if stack.is_empty() {
            // no item in the slot
            continue;
        };

        let Some(block_kind) = BlockKind::from_item_kind(stack.item) else {
            // can't place this item as a block
            continue;
        };

        let real_pos = message.position.get_in_direction(message.face);
        let state = block_kind.to_state().set(
            PropName::Axis,
            match message.face {
                Direction::Down | Direction::Up => PropValue::Y,
                Direction::North | Direction::South => PropValue::Z,
                Direction::West | Direction::East => PropValue::X,
            },
        );
        if layer.set_block(real_pos, state).is_none() {
            continue;
        }
        swings.swing(message.client, message.hand);

        if *game_mode == GameMode::Survival {
            let count = inventory.slot(slot_id).count;
            if count > 1 {
                inventory.set_slot_amount(slot_id, count - 1);
            } else {
                inventory.set_slot(slot_id, ItemStack::EMPTY);
            }
        }
    }
}
