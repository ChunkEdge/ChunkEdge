use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
/// Request the client to send the cookie with the specified key.
pub struct CookieRequestS2c<'a> {
    pub key: Ident<Cow<'a, str>>,
}
