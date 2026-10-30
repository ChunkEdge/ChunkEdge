use crate::{Hand, Packet};
use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_item::SwingAnimation;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SwingAnimationS2c {
    pub entity_id: VarInt,
    pub hand: Hand,
    pub animation: SwingAnimation,
}
