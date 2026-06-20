use valence_binary::{Decode, Encode, VarInt};

use crate::{BlockPos, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct JigsawGenerateC2s {
    pub position: BlockPos,
    pub levels: VarInt,
    pub keep_jigsaws: bool,
}
