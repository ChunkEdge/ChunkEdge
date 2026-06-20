use valence_binary::{Decode, Encode, VarInt};

use crate::{Hand, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct UseItemC2s {
    pub hand: Hand,
    pub sequence: VarInt,
    pub yaw: f32,
    pub pitch: f32,
}
