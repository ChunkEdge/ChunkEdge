use chunkedge_binary::{Decode, Encode};

use crate::{BlockPos, Packet, SignTextSlot};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct OpenSignEditorS2c {
    pub location: BlockPos,
    pub slot: SignTextSlot,
}
