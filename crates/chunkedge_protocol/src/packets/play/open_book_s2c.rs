use valence_binary::{Decode, Encode};

use crate::{Hand, Packet};

#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct OpenBookS2c {
    pub hand: Hand,
}
