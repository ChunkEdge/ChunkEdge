use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetActionBarTextS2c<'a> {
    pub action_bar_text: Cow<'a, TextComponent>,
}
