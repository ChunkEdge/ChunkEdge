use valence_binary::{Decode, Encode, VarInt};

use crate::{BlockPos, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct BlockEntityTagQueryC2s {
    pub transaction_id: VarInt,
    pub position: BlockPos,
}
