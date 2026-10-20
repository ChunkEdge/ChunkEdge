use chunkedge_binary::{Decode, Encode, TextComponent, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TestInstanceBlockStatusS2c {
    pub status: TextComponent,
    /// Optional size as three `VarInt`s, matching vanilla `Vec3i`.
    pub size: Option<(VarInt, VarInt, VarInt)>,
}
