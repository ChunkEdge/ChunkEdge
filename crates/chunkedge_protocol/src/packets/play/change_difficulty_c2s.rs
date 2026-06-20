use valence_binary::{Decode, Encode};

use crate::{Difficulty, Packet};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode, Packet)]
pub struct ChangeDifficultyC2s {
    pub difficulty: Difficulty,
}
