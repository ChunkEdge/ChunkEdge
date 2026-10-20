use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Subscribes to debug subscriptions. Replaces the old debug sample
/// subscription packet.
///
/// Each entry of `subscriptions` is a raw ID into the `debug_subscription`
/// registry (e.g. dedicated server tick time, bees, brains, ...).
///
/// wiki: [Debug Subscription Request](https://minecraft.wiki/w/Java_Edition_protocol#Debug_Subscription_Request)
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugSubscriptionRequestC2s {
    pub subscriptions: Vec<VarInt>,
}
