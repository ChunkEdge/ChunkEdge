use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetBorderSizeS2c {
    pub diameter: f64,
}
