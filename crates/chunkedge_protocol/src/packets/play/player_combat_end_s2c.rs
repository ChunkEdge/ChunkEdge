use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Unused by notchian clients.
#[derive(Copy, Clone, PartialEq, Debug, Encode, Decode, Packet)]
pub struct PlayerCombatEndS2c {
    pub duration: VarInt,
}
