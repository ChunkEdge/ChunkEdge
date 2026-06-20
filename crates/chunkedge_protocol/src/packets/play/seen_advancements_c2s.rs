use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub enum SeenAdvancementsC2s<'a> {
    OpenedTab { tab_id: Ident<Cow<'a, str>> },
    ClosedScreen,
}
