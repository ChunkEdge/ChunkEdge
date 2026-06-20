use valence_binary::{Decode, Encode};

use crate::{BlockPos, BlockState, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct BlockUpdateS2c {
    pub position: BlockPos,
    pub block_id: BlockState,
}
