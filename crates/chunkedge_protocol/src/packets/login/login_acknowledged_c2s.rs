use valence_binary::{Decode, Encode};

use crate::{Packet, PacketState};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Login)]
/// Sent by the client to the server to acknowledge the login process.
pub struct LoginAcknowledgedC2s;
