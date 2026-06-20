use valence_binary::{Decode, Encode};

use crate::{Packet, PacketState};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Status)]
pub struct PongResponseS2c {
    pub timestamp: u64,
}
