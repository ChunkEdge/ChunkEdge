use chunkedge_binary::{Decode, Encode, TextComponent, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TestInstanceBlockStatusS2c {
    pub status: TextComponent,
    pub size: Option<(VarInt, VarInt, VarInt)>,
}
