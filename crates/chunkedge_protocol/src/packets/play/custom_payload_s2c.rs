use std::borrow::Cow;

use chunkedge_binary::{Bounded, Decode, Encode, RawBytes};
use chunkedge_ident::Ident;

use crate::Packet;

const MAX_PAYLOAD_SIZE: usize = 0x100000;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct CustomPayloadS2c<'a> {
    pub channel: Ident<Cow<'a, str>>,
    pub data: Bounded<RawBytes<'a>, MAX_PAYLOAD_SIZE>,
}
