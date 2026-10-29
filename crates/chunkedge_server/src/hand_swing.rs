use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::EntityAnimations;
use chunkedge_protocol::Hand;
use chunkedge_protocol::packets::play::PunchC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct HandSwingPlugin;

impl Plugin for HandSwingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HandSwingMessage>()
            .add_systems(EventLoopPreUpdate, handle_hand_swing);
    }
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct HandSwingMessage {
    pub client: Entity,
    pub hand: Hand,
}

fn handle_hand_swing(
    mut packets: MessageReader<PacketMessage>,
    mut clients: Query<&mut EntityAnimations>,
    mut messages: MessageWriter<HandSwingMessage>,
) {
    for packet in packets.read() {
        if packet.decode::<PunchC2s>().is_some() {
            if let Ok(mut anim) = clients.get_mut(packet.client) {
                anim.trigger_swing(
                    Hand::Main,
                    // TODO: the kind and duration of this should not be hardcoded.
                    // We probably need to get this from the item in the player's hand
                    // but this should be done after item component defaults are implemented
                    // for now, hardcoding it as below is fine.
                    chunkedge_item::SwingAnimation {
                        kind: chunkedge_item::SwingAnimationType::Whack,
                        duration: chunkedge_protocol::VarInt(6),
                    },
                );
            }

            messages.write(HandSwingMessage {
                client: packet.client,
                hand: Hand::Main,
            });
        }
    }
}
