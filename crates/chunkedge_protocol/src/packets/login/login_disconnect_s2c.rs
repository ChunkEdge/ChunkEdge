use std::borrow::Cow;

use valence_binary::{Decode, Encode};
use valence_text::JsonText;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Login)]
/// Sent by the server to the client to disconnect the client from the server.
pub struct LoginDisconnectS2c<'a> {
    pub reason: Cow<'a, JsonText>,
}
