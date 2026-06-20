use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DisconnectS2c<'a> {
    pub reason: Cow<'a, TextComponent>,
}
