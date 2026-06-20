use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct EntityEventS2c {
    pub entity_id: i32,
    pub entity_status: u8,
}
