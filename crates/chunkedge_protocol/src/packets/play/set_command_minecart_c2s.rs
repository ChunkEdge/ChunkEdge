use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetCommandMinecartC2s<'a> {
    pub entity_id: VarInt,
    pub command: &'a str,
    pub track_output: bool,
}
