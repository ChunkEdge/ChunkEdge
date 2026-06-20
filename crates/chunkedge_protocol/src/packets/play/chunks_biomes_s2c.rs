use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};

use crate::{ChunkPos, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ChunksBiomesS2c<'a> {
    pub chunks: Cow<'a, [ChunkBiome<'a>]>,
}

#[derive(Clone, Debug)]
pub struct ChunkBiome<'a> {
    pub pos: ChunkPos,
    /// Chunk data structure, with sections containing only the `Biomes` field.
    pub data: &'a [u8],
}

// Note: The order of X and Z is inverted, because the client reads them
// as one big-endian Long, with Z being the upper 32 bits.
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Biomes
impl<'a> Decode<'a> for ChunkBiome<'a> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        Ok(ChunkBiome {
            pos: ChunkPos {
                z: Decode::decode(r)?,
                x: Decode::decode(r)?,
            },
            data: Decode::decode(r)?,
        })
    }
}

// Note: The order of X and Z is inverted, because the client reads them
// as one big-endian Long, with Z being the upper 32 bits.
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Biomes
impl Encode for ChunkBiome<'_> {
    fn encode(&self, mut w: impl std::io::Write) -> anyhow::Result<()> {
        self.pos.z.encode(&mut w)?;
        self.pos.x.encode(&mut w)?;
        self.data.encode(&mut w)
    }
}
