use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Sent by the server when a mount's inventory is opened.
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MountScreenOpenS2c {
    pub window_id: VarInt,
    pub slot_count: VarInt,
    pub entity_id: i32,
}
