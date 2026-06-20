use valence_binary::{Decode, Encode, VarInt};

use crate::{ByteAngle, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct RotateHeadS2c {
    pub entity_id: VarInt,
    pub head_yaw: ByteAngle,
}
