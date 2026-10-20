use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Sent by the server when a mount's inventory is opened. Renamed from
/// `HorseScreenOpenS2c`; the wire format is unchanged.
///
/// wiki: [Mount Screen Open](https://minecraft.wiki/w/Java_Edition_protocol#Mount_Screen_Open)
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MountScreenOpenS2c {
    pub window_id: VarInt,
    pub slot_count: VarInt,
    pub entity_id: i32,
}
