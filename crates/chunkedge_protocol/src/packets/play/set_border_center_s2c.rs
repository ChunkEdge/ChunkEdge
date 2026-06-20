use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetBorderCenterS2c {
    pub x_pos: f64,
    pub z_pos: f64,
}
