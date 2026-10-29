use crate::{Packet, PacketState};
use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;
use std::borrow::Cow;

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct PostEffectsS2c<'a> {
    pub post_effects: Vec<Ident<Cow<'a, str>>>,
}
