use valence_binary::{Decode, Encode, TextComponent};
use valence_math::DVec3;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TestInstanceBlockStatusS2c {
    pub status: TextComponent,
    pub size: Option<DVec3>,
}
