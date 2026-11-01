use crate::Packet;
use chunkedge_binary::{Decode, Encode};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct PunchC2s;
