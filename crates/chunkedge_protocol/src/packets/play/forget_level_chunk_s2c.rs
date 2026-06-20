use chunkedge_binary::{Decode, Encode};

use crate::{ChunkPos, Packet};

#[derive(Copy, Clone, Debug, Packet)]
pub struct ForgetLevelChunkS2c {
    pub pos: ChunkPos,
}

// Note: The order of X and Z is inverted, because the client reads them
// as one big-endian Long, with Z being the upper 32 bits.
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Chunk_Biomes
impl Decode<'_> for ForgetLevelChunkS2c {
    fn decode(r: &mut &'_ [u8]) -> anyhow::Result<Self> {
        Ok(ForgetLevelChunkS2c {
            pos: ChunkPos {
                z: Decode::decode(r)?,
                x: Decode::decode(r)?,
            },
        })
    }
}

// Note: The order is inverted, because the client reads this packet as
// one big-endian Long, with Z being the upper 32 bits.
// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Unload_Chunk
impl Encode for ForgetLevelChunkS2c {
    fn encode(&self, mut w: impl std::io::Write) -> anyhow::Result<()> {
        self.pos.z.encode(&mut w)?;
        self.pos.x.encode(&mut w)
    }
}
