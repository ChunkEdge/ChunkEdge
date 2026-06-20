use std::borrow::Cow;

use chunkedge_binary::{Bounded, Decode, Encode, RawBytes};
use chunkedge_ident::Ident;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// A custom payload sent from the client to the server.
/// You can read more about custom payloads [here](https://wiki.vg/Plugin_channels).
pub struct CustomPayloadC2s<'a> {
    pub channel: Ident<Cow<'a, str>>,
    pub data: Bounded<RawBytes<'a>, 32767>,
}
