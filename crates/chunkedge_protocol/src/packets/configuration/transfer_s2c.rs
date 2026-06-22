use chunkedge_binary::{Bounded, Decode, Encode, VarInt};

use crate::{Packet, PacketState};

#[derive(Clone, Debug, Encode, Decode, Packet)]
#[packet(state = PacketState::Configuration)]
pub struct TransferS2c<'a> {
    pub host: Bounded<&'a str, 32767>,
    pub port: VarInt,
}
