use valence_binary::{Bounded, Decode, Encode};

use crate::packets::play::client_information_c2s::{ChatMode, DisplayedSkinParts, MainArm};
use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct ClientInformationC2s<'a> {
    pub locale: Bounded<&'a str, 16>,
    pub view_distance: u8,
    pub chat_mode: ChatMode,
    pub chat_colors: bool,
    pub displayed_skin_parts: DisplayedSkinParts,
    pub main_arm: MainArm,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
    pub particle_mode: ParticleMode,
}

#[derive(Clone, Debug, Default, Encode, Decode)]
pub enum ParticleMode {
    #[default]
    All,
    Decreased,
    Minimal,
}
