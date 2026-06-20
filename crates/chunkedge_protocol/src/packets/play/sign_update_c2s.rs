use valence_binary::{Bounded, Decode, Encode};

use crate::{BlockPos, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SignUpdateC2s<'a> {
    pub position: BlockPos,
    pub is_front_text: bool,
    pub lines: [Bounded<&'a str, 384>; 4],
}
