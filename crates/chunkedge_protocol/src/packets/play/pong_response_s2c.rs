use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct PongResponseS2c {
    pub payload: u64,
}
