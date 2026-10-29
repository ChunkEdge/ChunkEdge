use crate::{BlockPos, BlockState, Packet};
use chunkedge_binary::{Decode, Encode};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct AddTransientBlockS2c {
    pub position: BlockPos,
    pub block_state: BlockState,
}
