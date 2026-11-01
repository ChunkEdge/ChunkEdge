use chunkedge_binary::{Decode, Encode};

/// The side of a sign, encoded as vanilla's `SignTextSlot` [`chunkedge_binary::VarInt`] ID.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum SignTextSlot {
    Back,
    Front,
}
