use chunkedge_binary::{Bounded, Decode, Encode, VarInt};

use crate::{FixedBitSet, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ChatCommandSignedC2s<'a> {
    pub command: Bounded<&'a str, 32767>,
    pub timestamp: i64,
    pub salt: i64,
    pub argument_signatures: Bounded<Vec<CommandArgumentSignature<'a>>, 8>,
    pub message_count: VarInt,
    //// This is a bitset of 20; each bit represents one
    //// of the last 20 messages received and whether or not
    //// the message was acknowledged by the client
    pub acknowledgement: FixedBitSet<20, 3>,
    pub checksum: i8,
}

#[derive(Copy, Clone, Debug, Encode, Decode)]
pub struct CommandArgumentSignature<'a> {
    pub argument_name: Bounded<&'a str, 16>,
    pub signature: &'a [u8; 256],
}
