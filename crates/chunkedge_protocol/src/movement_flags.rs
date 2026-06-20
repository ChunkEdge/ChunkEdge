use bitfield_struct::bitfield;
use valence_binary::{Decode, Encode};

#[bitfield(u8)]
#[derive(PartialEq, Eq, Encode, Decode)]
pub struct MovementFlags {
    pub on_ground: bool,
    pub pushing_against_wall: bool,
    #[bits(6)]
    _padding: u8,
}
