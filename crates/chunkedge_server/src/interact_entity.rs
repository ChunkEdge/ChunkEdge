use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::EntityManager;
use chunkedge_math::DVec3;
use chunkedge_protocol::Hand;
use chunkedge_protocol::packets::play::InteractC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct InteractEntityPlugin;

impl Plugin for InteractEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<InteractEntityMessage>()
            .add_systems(EventLoopPreUpdate, handle_interact_entity);
    }
}

#[derive(Message, Copy, Clone, Debug)]
pub struct InteractEntityMessage {
    pub client: Entity,
    /// The entity being interacted with.
    pub entity: Entity,
    /// If the client was sneaking during the interaction.
    pub sneaking: bool,
    /// The hand used for the interaction.
    pub hand: Hand,
    /// The interaction target, relative to the entity.
    pub target: DVec3,
}

fn handle_interact_entity(
    mut packets: MessageReader<PacketMessage>,
    entities: Res<EntityManager>,
    mut messages: MessageWriter<InteractEntityMessage>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<InteractC2s>() {
            // TODO: check that the entity is in the same instance as the player.
            // TODO: check that the distance between the player and the interacted entity is
            // within some configurable tolerance level.

            if let Some(entity) = entities.get_by_id(pkt.entity_id.0) {
                messages.write(InteractEntityMessage {
                    client: packet.client,
                    entity,
                    sneaking: pkt.sneaking,
                    hand: pkt.hand,
                    target: pkt.target_vec(),
                });
            }
        }
    }
}
