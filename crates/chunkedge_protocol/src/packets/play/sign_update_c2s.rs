use chunkedge_binary::{Bounded, Decode, Encode};

use crate::{BlockPos, Packet, SignTextSlot};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SignUpdateC2s<'a> {
    pub position: BlockPos,
    pub lines: [Bounded<&'a str, 384>; 4],
    pub slot: SignTextSlot,
}
