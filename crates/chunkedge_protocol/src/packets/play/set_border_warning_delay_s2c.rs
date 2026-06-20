use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetBorderWarningDelayS2c {
    pub warning_time: VarInt,
}
