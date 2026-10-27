use chunkedge_binary::{Decode, Encode, IdOr};
use chunkedge_nbt::Compound;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
/// Shows a dialog to the client. The dialog is either a registry reference
/// (by ID) or inline NBT data.
pub struct ShowDialogS2c {
    pub dialog: IdOr<Compound>,
}
