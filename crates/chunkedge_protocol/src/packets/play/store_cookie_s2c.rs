use std::borrow::Cow;

use valence_binary::{Bounded, Decode, Encode};
use valence_ident::Ident;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
/// Stores a cookie on the client
pub struct StoreCookieS2c<'a> {
    pub key: Ident<Cow<'a, str>>,
    pub payload: Bounded<&'a [u8], 5120>,
}
