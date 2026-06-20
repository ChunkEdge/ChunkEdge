use valence_binary::{Decode, Encode};

use crate::{BlockPos, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetDefaultSpawnPositionS2c {
    pub position: BlockPos,
    pub angle: f32,
}
