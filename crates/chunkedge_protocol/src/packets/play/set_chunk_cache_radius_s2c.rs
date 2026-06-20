use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetChunkCacheRadiusS2c {
    pub view_distance: VarInt,
}
