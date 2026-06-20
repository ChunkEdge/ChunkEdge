use std::borrow::Cow;

use valence_binary::{Bounded, Decode, Encode, RawBytes};
use valence_ident::Ident;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// A custom payload sent from the server to the client.
/// You can read more about custom payloads [here](https://wiki.vg/Plugin_channels).
pub struct CustomPayloadS2c<'a> {
    pub channel: Ident<Cow<'a, str>>,
    pub data: Bounded<RawBytes<'a>, 1048576>,
}
