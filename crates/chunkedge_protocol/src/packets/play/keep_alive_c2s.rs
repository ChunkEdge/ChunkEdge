use chunkedge_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct KeepAliveC2s {
    pub id: i64,
}
