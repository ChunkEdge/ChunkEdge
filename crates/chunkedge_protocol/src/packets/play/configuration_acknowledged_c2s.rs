use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ConfigurationAcknowledgedC2s;
