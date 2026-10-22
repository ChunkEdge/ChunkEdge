use chunkedge_binary::{Decode, Encode};
use chunkedge_nbt::Compound;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Shows a dialog to the client during configuration using inline NBT data.
pub struct ShowDialogS2c {
    pub dialog: Compound,
}
