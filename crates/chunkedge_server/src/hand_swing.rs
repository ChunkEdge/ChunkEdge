use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::{EntityAnimation, EntityAnimations};
use chunkedge_protocol::packets::play::SwingC2s;
use chunkedge_protocol::Hand;

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
        if let Some(pkt) = packet.decode::<SwingC2s>() {
            if let Ok(mut anim) = clients.get_mut(packet.client) {
                anim.trigger(match pkt.hand {
                    Hand::Main => EntityAnimation::SwingMainHand,
                    Hand::Off => EntityAnimation::SwingOffHand,
                });
            }

            messages.write(HandSwingMessage {
                client: packet.client,
                hand: pkt.hand,
            });
        }
    }
}
