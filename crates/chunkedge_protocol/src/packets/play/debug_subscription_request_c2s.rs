use chunkedge_binary::{Bounded, Decode, Encode, VarInt};

use crate::Packet;

/// Requests debug data for the specified subscriptions.
///
/// Each entry of `subscriptions` is a raw ID into the `debug_subscription`
/// registry (e.g. dedicated server tick time, bees, brains, ...).
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugSubscriptionRequestC2s {
    pub subscriptions: Bounded<Vec<VarInt>, 32>,
}
