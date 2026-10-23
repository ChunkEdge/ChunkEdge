use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};

use crate::{ChunkPos, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ChunksBiomesS2c<'a> {
    pub chunks: Cow<'a, [ChunkBiome<'a>]>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ChunkBiome<'a> {
    #[packet(with = crate::chunk_pos::packed)]
    pub pos: ChunkPos,
    /// Chunk data structure, with sections containing only the `Biomes` field.
    pub data: &'a [u8],
}
