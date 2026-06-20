use std::borrow::Cow;
use std::collections::BTreeMap;

use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_ident::Ident;

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct UpdateTagsS2c<'a> {
    pub groups: Cow<'a, RegistryMap>,
}

pub type RegistryMap = BTreeMap<Ident<String>, BTreeMap<Ident<String>, Vec<VarInt>>>;
