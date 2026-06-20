use std::borrow::Cow;

use valence_binary::{Bounded, Decode, Encode, RawBytes};
use valence_ident::Ident;

use crate::Packet;

pub const MAX_PAYLOAD_SIZE: usize = 32767;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct CustomPayloadC2s<'a> {
    pub channel: Ident<Cow<'a, str>>,
    pub data: Bounded<RawBytes<'a>, MAX_PAYLOAD_SIZE>,
}
