use chunkedge_binary::{Decode, Encode, VarLong};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetBorderLerpSizeS2c {
    pub old_diameter: f64,
    pub new_diameter: f64,
    /// Transition duration in game ticks.
    pub duration_ticks: VarLong,
}
