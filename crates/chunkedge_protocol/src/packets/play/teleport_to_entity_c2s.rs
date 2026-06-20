use uuid::Uuid;
use valence_binary::{Decode, Encode};

use crate::Packet;
//Teleports the player to the given entity. The player must be in spectator
// mode.
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct TeleportToEntityC2s {
    pub target: Uuid,
}
