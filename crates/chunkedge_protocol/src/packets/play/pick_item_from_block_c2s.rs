use valence_binary::{Decode, Encode};

use crate::{BlockPos, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct PickItemFromBlockC2s {
    pub block_position: BlockPos,
    pub include_data: bool,
}
