use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::{EntityManager, entity::SharedFlags};
use chunkedge_math::DVec3;
use chunkedge_protocol::Hand;
use chunkedge_protocol::packets::play::{AttackC2s, InteractC2s};

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct InteractEntityPlugin;

impl Plugin for InteractEntityPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<InteractEntityMessage>().add_systems(
            EventLoopPreUpdate,
            handle_interact_entity.after(crate::client_command::handle_player_input),
        );
    }
}

/// Whether an entity was attacked (left-click) or interacted with (right-click).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum EntityInteractionKind {
    Attack,
    Interact,
}

#[derive(Message, Copy, Clone, Debug)]
pub struct InteractEntityMessage {
    pub client: Entity,
    /// The entity being interacted with.
    pub entity: Entity,
    /// The kind of interaction that occurred.
    pub kind: EntityInteractionKind,
    /// If the client was sneaking during the interaction.
    /// For attacks, this comes from the client's tracked input state.
    pub sneaking: bool,
    /// The hand used for the interaction. Always the main hand for attacks.
    pub hand: Hand,
    /// The interaction target, relative to the entity. Zero for attacks, which send no hit position.
    pub target: DVec3,
}

fn handle_interact_entity(
    mut packets: MessageReader<PacketMessage>,
    entities: Res<EntityManager>,
    clients: Query<&SharedFlags>,
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
                    kind: EntityInteractionKind::Interact,
                    sneaking: pkt.sneaking,
                    hand: pkt.hand,
                    target: pkt.target_vec(),
                });
            }
        } else if let Some(pkt) = packet.decode::<AttackC2s>()
            && let Some(entity) = entities.get_by_id(pkt.entity_id.0)
        {
            messages.write(InteractEntityMessage {
                client: packet.client,
                entity,
                kind: EntityInteractionKind::Attack,
                sneaking: clients.get(packet.client).is_ok_and(SharedFlags::sneaking),
                hand: Hand::Main,
                target: DVec3::ZERO,
            });
        }
    }
}
