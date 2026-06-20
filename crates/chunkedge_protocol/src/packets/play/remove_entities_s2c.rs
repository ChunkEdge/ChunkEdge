use std::borrow::Cow;

use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, PartialEq, Debug, Encode, Decode, Packet)]
pub struct RemoveEntitiesS2c<'a> {
    pub entity_ids: Cow<'a, [VarInt]>,
}
