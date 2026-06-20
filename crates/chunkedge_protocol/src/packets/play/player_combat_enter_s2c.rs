use chunkedge_binary::{Decode, Encode};

use crate::Packet;

/// Unused by notchian clients.
#[derive(Copy, Clone, PartialEq, Debug, Encode, Decode, Packet)]
pub struct PlayerCombatEnterS2c;
