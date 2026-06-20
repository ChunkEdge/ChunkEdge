use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent, VarInt};

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct PlayerCombatKillS2c<'a> {
    pub player_id: VarInt,
    pub message: Cow<'a, TextComponent>,
}
