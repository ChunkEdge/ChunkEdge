use std::borrow::Cow;

use valence_binary::{Decode, Encode};
use valence_ident::Ident;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Request the client to send the cookie with the specified key.
pub struct CookieRequestS2c<'a> {
    pub key: Ident<Cow<'a, str>>,
}
