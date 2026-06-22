use bevy_ecs::prelude::*;
use chunkedge_server::event_loop::PacketEvent;
use chunkedge_server::protocol::packets::play::SeenAdvancementsC2s;
use chunkedge_server::Ident;

/// This event sends when the client changes or closes advancement's tab.
#[derive(Message, Clone, PartialEq, Eq, Debug)]
pub struct AdvancementTabChangeEvent {
    pub client: Entity,
    /// If None then the client has closed advancement's tabs.
    pub opened_tab: Option<Ident<String>>,
}

pub(crate) fn handle_advancement_tab_change(
    mut packets: MessageReader<PacketEvent>,
    mut advancement_tab_change_events: MessageWriter<AdvancementTabChangeEvent>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<SeenAdvancementsC2s>() {
            advancement_tab_change_events.write(AdvancementTabChangeEvent {
                client: packet.client,
                opened_tab: match pkt {
                    SeenAdvancementsC2s::ClosedScreen => None,
                    SeenAdvancementsC2s::OpenedTab { tab_id } => Some(tab_id.into()),
                },
            });
        }
    }
}
