use valence_binary::{Decode, Encode};

use crate::{Packet, PacketState};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Status)]
pub struct PingRequestC2s {
    /// May be any number, but vanilla clients will always use the timestamp in
    /// milliseconds.
    pub timestamp: u64,
}
