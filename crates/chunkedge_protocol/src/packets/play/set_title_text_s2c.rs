use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetTitleTextS2c<'a> {
    pub title_text: Cow<'a, TextComponent>,
}
