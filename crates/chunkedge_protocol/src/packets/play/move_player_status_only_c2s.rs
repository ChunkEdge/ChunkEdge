use chunkedge_binary::{Decode, Encode};

use crate::Packet;
use crate::movement_flags::MovementFlags;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MovePlayerStatusOnlyC2s {
    pub flags: MovementFlags,
}
