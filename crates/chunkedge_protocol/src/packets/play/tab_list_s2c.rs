use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TabListS2c<'a> {
    pub header: Cow<'a, TextComponent>,
    pub footer: Cow<'a, TextComponent>,
}
