use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Login)]
/// Sent by the server to the client to request a cookie.
pub struct CookieRequestS2c<'a> {
    pub key: Ident<Cow<'a, str>>,
}
