use std::borrow::Cow;

use chunkedge_binary::{Bounded, Decode, Encode};
use chunkedge_ident::Ident;

use crate::Packet;

/// Carries the payload of a custom click action from a dialog or message.
/// The `payload` is the raw bytes of an untrusted NBT tag of any type.
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct CustomClickActionC2s<'a> {
    pub id: Ident<Cow<'a, str>>,
    pub payload: Bounded<&'a [u8], 65536>,
}
