#![allow(clippy::type_complexity)]

use bevy_app::App;
use chunkedge::client::despawn_disconnected_clients;
use chunkedge::inventory::HeldItem;
use chunkedge::message::{ChatReceivedMessage, SendMessage};
use chunkedge::prelude::*;
use chunkedge::world_border::*;

const SPAWN_Y: i32 = 64;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                despawn_disconnected_clients,
                init_clients,
                border_controls,
                display_diameter,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    biomes: Res<BiomeRegistry>,
    dimensions: Res<DimensionTypeRegistry>,
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
                .set_block([x, SPAWN_Y, z], BlockState::MOSSY_COBBLESTONE);
        }
    }

    commands.spawn((
        layer,
        WorldBorderBundle {
            lerp: WorldBorderLerp {
                target_diameter: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn init_clients(
    mut clients: Query<
        (
            &mut Client,
            &mut EntityLayerId,
            &mut VisibleChunkLayer,
            &mut VisibleEntityLayers,
            &mut Position,
            &mut Inventory,
            &HeldItem,
        ),
        Added<Client>,
    >,
    layers: Query<Entity, With<ChunkLayer>>,
) {
    for (
        mut client,
        mut layer_id,
        mut visible_chunk_layer,
        mut visible_entity_layers,
        mut pos,
        mut inv,
        main_slot,
    ) in &mut clients
    {
        let layer = layers.single().unwrap();

        layer_id.0 = layer;
        visible_chunk_layer.0 = layer;
        visible_entity_layers.0.insert(layer);
        pos.set([0.5, f64::from(SPAWN_Y) + 1.0, 0.5]);
        let pickaxe = ItemStack::new(ItemKind::WoodenPickaxe, 1);
        inv.set_slot(main_slot.slot(), pickaxe);
        client
            .send_chat_message("Use `add <diameter change> <ticks>` or `center <x> <z>` in chat to change the world border.");
    }
}

fn display_diameter(mut layers: Query<(&mut ChunkLayer, &WorldBorderLerp)>) {
    for (mut layer, lerp) in &mut layers {
        if lerp.remaining_ticks > 0 {
            layer.send_chat_message(format!("diameter = {}", lerp.current_diameter));
        }
    }
}

fn border_controls(
    mut messages: MessageReader<ChatReceivedMessage>,
    mut layers: Query<(&mut WorldBorderCenter, &mut WorldBorderLerp), With<ChunkLayer>>,
    mut clients: Query<&mut Client>,
) {
    for message in messages.read() {
        let parts: Vec<&str> = message.message.split_whitespace().collect();
        match parts.as_slice() {
            ["add", value, ticks]
                if let (Ok(value), Ok(ticks)) = (value.parse::<f64>(), ticks.parse::<u64>())
                    && value.is_finite() =>
            {
                let (_, mut lerp) = layers.single_mut().unwrap();
                lerp.target_diameter = lerp.current_diameter + value;
                lerp.remaining_ticks = ticks;
            }
            ["center", x, z]
                if let (Ok(x), Ok(z)) = (x.parse::<f64>(), z.parse::<f64>())
                    && x.is_finite()
                    && z.is_finite() =>
            {
                let (mut center, _) = layers.single_mut().unwrap();
                center.x = x;
                center.z = z;
            }
            ["add" | "center", ..] => {
                if let Ok(mut client) = clients.get_mut(message.client) {
                    client.send_chat_message(
                        "Usage: add <diameter change> <ticks> or center <x> <z>. Coordinates and diameter changes must be finite numbers; ticks must be a non-negative integer.",
                    );
                }
            }
            _ => (),
        }
    }
}
