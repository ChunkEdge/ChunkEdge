use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetEntityLinkS2c {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
}
