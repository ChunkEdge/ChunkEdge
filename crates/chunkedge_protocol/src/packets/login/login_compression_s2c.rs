use valence_binary::{Decode, Encode, VarInt};

use crate::{Packet, PacketState};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Login)]
// Optionally sent by the server to the client to enable compression for the
// connection.
pub struct LoginCompressionS2c {
    pub threshold: VarInt,
}
