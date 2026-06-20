use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent};

use super::player_chat_s2c::ChatType;
use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct DisguisedChatS2c<'a> {
    pub message: Cow<'a, TextComponent>,
    pub chat_type: ChatType<'a>,
    pub sender_name: Cow<'a, TextComponent>,
    pub target_name: Option<Cow<'a, TextComponent>>,
}
