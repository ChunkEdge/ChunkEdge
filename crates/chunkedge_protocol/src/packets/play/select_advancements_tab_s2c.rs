use std::borrow::Cow;

use valence_binary::{Decode, Encode};
use valence_ident::Ident;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SelectAdvancementsTabS2c<'a> {
    pub identifier: Option<Ident<Cow<'a, str>>>,
}
