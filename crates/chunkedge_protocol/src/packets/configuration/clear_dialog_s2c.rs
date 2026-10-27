use chunkedge_binary::{Decode, Encode};

use crate::{Packet, PacketState};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Clears the currently displayed dialog. Empty packet.
pub struct ClearDialogS2c;
