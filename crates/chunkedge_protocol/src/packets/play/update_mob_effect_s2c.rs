use bitfield_struct::bitfield;
use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct UpdateMobEffectS2c {
    pub entity_id: VarInt,
    pub effect_id: VarInt, // TODO: effect ID registry.
    pub amplifier: VarInt,
    pub duration: VarInt,
    pub flags: Flags,
}

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct Flags {
    pub is_ambient: bool,
    pub show_particles: bool,
    pub show_icon: bool,
    pub blend: bool,
    #[bits(4)]
    _pad: u8,
}
