use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DebugSampleS2c {
    pub sample: Vec<i64>,
    pub sample_type: DebugSampleType,
}
#[derive(Clone, Debug, Encode, Decode)]
pub enum DebugSampleType {
    TickTime,
}
