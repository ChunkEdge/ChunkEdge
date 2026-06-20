use std::borrow::Cow;

use chunkedge_binary::array::FixedArray;
use chunkedge_binary::{Decode, Encode};
use chunkedge_generated::block::BlockEntityKind;
use chunkedge_nbt::Compound;

use crate::{ChunkPos, Packet, VariableBitSet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct LevelChunkWithLightS2c<'a> {
    pub pos: ChunkPos,
    pub heightmaps: Cow<'a, [HeightMap]>,
    pub blocks_and_biomes: &'a [u8],
    pub block_entities: Cow<'a, [ChunkDataBlockEntity<'a>]>,
    pub sky_light_mask: Cow<'a, VariableBitSet>,
    pub block_light_mask: Cow<'a, VariableBitSet>,
    pub empty_sky_light_mask: Cow<'a, VariableBitSet>,
    pub empty_block_light_mask: Cow<'a, VariableBitSet>,
    pub sky_light_arrays: Cow<'a, [FixedArray<u8, 2048>]>,
    pub block_light_arrays: Cow<'a, [FixedArray<u8, 2048>]>,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct HeightMap {
    pub kind: HeightMapKind,
    pub data: Vec<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Encode, Decode)]
pub enum HeightMapKind {
    /// All blocks other than air, cave air and void air.
    #[packet(tag = 1)]
    WorldSurface,
    /// "Solid" blocks, except bamboo saplings and cactuses; fluids.
    #[packet(tag = 4)]
    MotionBlocking,
    /// Same as `MOTION_BLOCKING`, excluding leaf blocks.
    #[packet(tag = 5)]
    MotionBlockingNoLeaves,
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct ChunkDataBlockEntity<'a> {
    pub packed_xz: i8,
    pub y: i16,
    pub kind: BlockEntityKind,
    pub data: Cow<'a, Compound>,
}
