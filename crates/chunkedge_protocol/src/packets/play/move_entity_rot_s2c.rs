use valence_binary::{Decode, Encode, VarInt};

use crate::{ByteAngle, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct MoveEntityRotS2c {
    pub entity_id: VarInt,
    pub yaw: ByteAngle,
    pub pitch: ByteAngle,
    pub on_ground: bool,
}
