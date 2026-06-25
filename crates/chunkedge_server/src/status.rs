use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_protocol::packets::play::ClientCommandC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RequestRespawnMessage>()
            .add_message::<RequestStatsMessage>()
            .add_systems(EventLoopPreUpdate, handle_status);
    }
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct RequestRespawnMessage {
    pub client: Entity,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct RequestStatsMessage {
    pub client: Entity,
}

fn handle_status(
    mut packets: MessageReader<PacketMessage>,
    mut respawn_messages: MessageWriter<RequestRespawnMessage>,
    mut request_stats_messages: MessageWriter<RequestStatsMessage>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<ClientCommandC2s>() {
            match pkt {
                ClientCommandC2s::PerformRespawn => {
                    respawn_messages.write(RequestRespawnMessage {
                        client: packet.client,
                    });
                }
                ClientCommandC2s::RequestStats => {
                    request_stats_messages.write(RequestStatsMessage {
                        client: packet.client,
                    });
                }
            }
        }
    }
}
