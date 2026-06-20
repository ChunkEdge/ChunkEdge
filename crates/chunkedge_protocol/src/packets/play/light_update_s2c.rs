use std::borrow::Cow;

use chunkedge_binary::array::FixedArray;
use chunkedge_binary::{Decode, Encode, VarInt};

use crate::{Packet, VariableBitSet};
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct LightUpdateS2c<'a> {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
    pub sky_light_mask: Cow<'a, VariableBitSet>,
    pub block_light_mask: Cow<'a, VariableBitSet>,
    pub empty_sky_light_mask: Cow<'a, VariableBitSet>,
    pub empty_block_light_mask: Cow<'a, VariableBitSet>,
    pub sky_light_arrays: Cow<'a, [FixedArray<u8, 2048>]>,
    pub block_light_arrays: Cow<'a, [FixedArray<u8, 2048>]>,
}
