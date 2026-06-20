use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct UpdateEnabledFeaturesS2c<'a> {
    pub features: Vec<Ident<Cow<'a, str>>>,
}
