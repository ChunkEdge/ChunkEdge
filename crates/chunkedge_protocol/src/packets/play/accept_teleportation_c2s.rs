use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct AcceptTeleportationC2s {
    pub teleport_id: VarInt,
}
