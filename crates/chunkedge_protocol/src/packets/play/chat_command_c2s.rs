use std::borrow::Cow;

use valence_binary::{Decode, Encode};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ChatCommandC2s<'a> {
    pub command: Cow<'a, str>,
}
