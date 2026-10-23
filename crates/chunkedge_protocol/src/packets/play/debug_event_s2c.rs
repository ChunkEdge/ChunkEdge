use chunkedge_binary::{Bounded, Decode, Encode, RawBytes, VarInt};

use crate::Packet;

const MAX_PAYLOAD_SIZE: usize = 0x200000;

/// A debug subscription event. The `payload` is the subscription-specific
/// event encoding; see the vanilla `DebugSubscription` registry.
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugEventS2c<'a> {
    pub subscription: VarInt,
    pub payload: Bounded<RawBytes<'a>, MAX_PAYLOAD_SIZE>,
}
