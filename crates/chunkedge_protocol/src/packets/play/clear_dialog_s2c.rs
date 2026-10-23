use chunkedge_binary::{Decode, Encode};

use crate::Packet;

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
/// Clears the currently displayed dialog. Empty packet.
pub struct ClearDialogS2c;
