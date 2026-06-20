use valence_binary::{Bounded, Decode, Encode, VarInt};

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Handshake)]
/// This packet causes the server to switch into the target state.
/// It should be sent right after opening the TCP connection to prevent the
/// server from disconnecting.
pub struct IntentionC2s<'a> {
    pub protocol_version: VarInt,
    pub server_address: Bounded<&'a str, 255>,
    pub server_port: u16,
    pub intent: HandShakeIntent,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum HandShakeIntent {
    #[packet(tag = 1)]
    Status,
    #[packet(tag = 2)]
    Login,
    #[packet(tag = 3)]
    Transfer,
}
