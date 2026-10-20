use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_protocol::packets::play::ClientCommandC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RequestRespawnMessage>()
            .add_message::<RequestStatsMessage>()
            .add_message::<RequestGameruleValuesMessage>()
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

/// Emitted when a vanilla client asks for the current gamerule values.
/// No gamerule storage exists yet; user code can answer with a
/// [`GameRuleValuesS2c`](chunkedge_protocol::packets::play::GameRuleValuesS2c).
#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct RequestGameruleValuesMessage {
    pub client: Entity,
}

fn handle_status(
    mut packets: MessageReader<PacketMessage>,
    mut respawn_messages: MessageWriter<RequestRespawnMessage>,
    mut request_stats_messages: MessageWriter<RequestStatsMessage>,
    mut request_gamerule_messages: MessageWriter<RequestGameruleValuesMessage>,
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
                ClientCommandC2s::RequestGameruleValues => {
                    request_gamerule_messages.write(RequestGameruleValuesMessage {
                        client: packet.client,
                    });
                }
            }
        }
    }
}
