use chunkedge_binary::{Decode, Encode};

use crate::Packet;

/// Warns the client that the server is running low on disk space. Empty packet.
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct LowDiskSpaceWarningS2c;
