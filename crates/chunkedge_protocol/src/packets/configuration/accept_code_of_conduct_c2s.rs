use chunkedge_binary::{Decode, Encode};

use crate::{Packet, PacketState};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Accepts the server's code of conduct. Empty packet.
pub struct AcceptCodeOfConductC2s;
