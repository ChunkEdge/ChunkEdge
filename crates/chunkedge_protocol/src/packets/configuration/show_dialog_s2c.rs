use chunkedge_binary::{Decode, Encode, IdOr};
use chunkedge_nbt::Compound;

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
/// Shows a dialog to the client during configuration. The dialog is either
/// a registry reference (by ID) or inline NBT data.
pub struct ShowDialogS2c {
    pub dialog: IdOr<Compound>,
}
