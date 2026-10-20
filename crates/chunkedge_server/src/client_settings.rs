use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use chunkedge_entity::avatar::{DataPlayerMainHand, DataPlayerModeCustomisation};
use chunkedge_protocol::packets::configuration::client_information_c2s::ParticleMode;
use chunkedge_protocol::packets::play::ClientInformationC2s;
use chunkedge_protocol::packets::play::client_information_c2s::ChatMode;

use crate::client::{ViewDistance, map_main_arm};
use crate::event_loop::{EventLoopPreUpdate, PacketMessage};

pub struct ClientSettingsPlugin;

impl Plugin for ClientSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EventLoopPreUpdate, handle_client_settings);
    }
}

/// Component containing client-controlled settings about a client.
#[derive(Component, Default, Debug)]
pub struct ClientSettings {
    pub locale: Box<str>,
    pub chat_mode: ChatMode,
    pub chat_colors: bool,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
    pub particle_mode: ParticleMode,
}

fn handle_client_settings(
    mut packets: MessageReader<PacketMessage>,
    mut clients: Query<(
        &mut ViewDistance,
        &mut ClientSettings,
        &mut DataPlayerModeCustomisation,
        &mut DataPlayerMainHand,
    )>,
) {
    for packet in packets.read() {
        if let Some(pkt) = packet.decode::<ClientInformationC2s>()
            && let Ok((mut view_dist, mut settings, mut model_parts, mut main_arm)) =
                clients.get_mut(packet.client)
        {
            // TODO: set a server max view distance
            view_dist.set_if_neq(ViewDistance::new(pkt.view_distance));

            settings.locale = pkt.locale.0.into();
            settings.chat_mode = pkt.chat_mode;
            settings.chat_colors = pkt.chat_colors;
            settings.enable_text_filtering = pkt.enable_text_filtering;
            settings.allow_server_listings = pkt.allow_server_listings;
            settings.particle_mode = pkt.particle_mode;

            model_parts.set_if_neq(DataPlayerModeCustomisation(
                u8::from(pkt.displayed_skin_parts) as i8,
            ));
            main_arm.set_if_neq(DataPlayerMainHand(map_main_arm(pkt.main_arm)));
        }
    }
}
