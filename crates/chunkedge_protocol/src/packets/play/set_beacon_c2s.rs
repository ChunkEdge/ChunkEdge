use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SetBeaconC2s {
    pub primary_effect: Option<VarInt>,
    pub secondary_effect: Option<VarInt>,
}
