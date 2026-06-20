use valence_binary::{Decode, Encode};

use crate::movement_flags::MovementFlags;
use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MovePlayerStatusOnlyC2s {
    pub flags: MovementFlags,
}
