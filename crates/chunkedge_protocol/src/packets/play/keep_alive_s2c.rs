use chunkedge_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct KeepAliveS2c {
    pub id: i64,
}
