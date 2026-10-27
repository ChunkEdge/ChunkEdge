use chunkedge_binary::{Decode, Encode};

use crate::{ChunkPos, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct ForgetLevelChunkS2c {
    #[packet(with = crate::chunk_pos::packed)]
    pub pos: ChunkPos,
}
