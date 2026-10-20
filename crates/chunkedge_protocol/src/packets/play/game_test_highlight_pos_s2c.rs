use chunkedge_binary::{Decode, Encode};
use chunkedge_protocol::BlockPos;

use crate::Packet;

/// Highlights a game-test position to the client.
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct GameTestHighlightPosS2c {
    pub absolute_pos: BlockPos,
    pub relative_pos: BlockPos,
}
