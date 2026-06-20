use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TickingStepS2c {
    pub tick_steps: VarInt,
}
