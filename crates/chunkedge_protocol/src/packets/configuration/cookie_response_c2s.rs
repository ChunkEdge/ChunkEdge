use std::borrow::Cow;

use chunkedge_binary::{Bounded, Decode, Encode};
use chunkedge_ident::Ident;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Sent by the client to the server to respond to a
/// [`CookieRequestS2c`](crate::packets::configuration::CookieRequestS2c)
/// packet.
pub struct CookieResponseC2s<'a> {
    pub key: Ident<Cow<'a, str>>,
    pub payload: Option<Bounded<&'a [u8], 5120>>,
}
