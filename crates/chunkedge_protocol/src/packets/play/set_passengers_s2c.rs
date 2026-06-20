use std::borrow::Cow;

use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetPassengersS2c<'a> {
    /// Vehicle's entity id
    pub entity_id: VarInt,
    pub passengers: Cow<'a, [VarInt]>,
}
