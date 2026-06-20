use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SelectAdvancementsTabS2c<'a> {
    pub identifier: Option<Ident<Cow<'a, str>>>,
}
