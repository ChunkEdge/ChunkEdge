use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_protocol::packets::play::ClientCommandC2s;

use crate::event_loop::{EventLoopPreUpdate, PacketEvent};

pub struct StatusPlugin;

impl Plugin for StatusPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RequestRespawnEvent>()
            .add_message::<RequestStatsEvent>()
            .add_systems(EventLoopPreUpdate, handle_status);
    }
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct RequestRespawnEvent {
    pub client: Entity,
}

#[derive(Message, Copy, Clone, PartialEq, Eq, Debug)]
pub struct RequestStatsEvent {
    pub client: Entity,
}

fn handle_status(
    mut packets: MessageReader<PacketEvent>,
    mut respawn_events: MessageWriter<RequestRespawnEvent>,
    mut request_stats_events: MessageWriter<RequestStatsEvent>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<ClientCommandC2s>() {
            match pkt {
                ClientCommandC2s::PerformRespawn => {
                    respawn_events.write(RequestRespawnEvent {
                        client: packet.client,
                    });
                }
                ClientCommandC2s::RequestStats => {
                    request_stats_events.write(RequestStatsEvent {
                        client: packet.client,
                    });
                }
            }
        }
    }
}
