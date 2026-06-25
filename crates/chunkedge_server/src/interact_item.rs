use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_protocol::packets::play::UseItemC2s;
use chunkedge_protocol::Hand;

use crate::action::ActionSequence;
use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct InteractItemPlugin;

impl Plugin for InteractItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<InteractItemMessage>()
            .add_systems(EventLoopPreUpdate, handle_player_interact_item);
    }
}

#[derive(Message, Copy, Clone, Debug)]
pub struct InteractItemMessage {
    pub client: Entity,
    pub hand: Hand,
    pub sequence: i32,
}

fn handle_player_interact_item(
    mut packets: MessageReader<PacketMessage>,
    mut clients: Query<&mut ActionSequence>,
    mut messages: MessageWriter<InteractItemMessage>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<UseItemC2s>() {
            if let Ok(mut action_seq) = clients.get_mut(packet.client) {
                action_seq.update(pkt.sequence.0);
            }

            messages.write(InteractItemMessage {
                client: packet.client,
                hand: pkt.hand,
                sequence: pkt.sequence.0,
            });
        }
    }
}
