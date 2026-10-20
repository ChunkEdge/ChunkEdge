use chunkedge_binary::{Decode, Encode};

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Sends the server's code of conduct to the client.
pub struct CodeOfConductS2c {
    pub contents: String,
}
