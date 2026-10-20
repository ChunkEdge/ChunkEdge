use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::Packet;

/// Sets game rule values from the client (edit-game-rules screens).
///
/// wiki: [Set Game Rule](https://minecraft.wiki/w/Java_Edition_protocol#Set_Game_Rule)
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetGameRuleC2s {
    pub entries: Vec<SetGameRuleEntry>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct SetGameRuleEntry {
    pub name: Ident<String>,
    pub value: String,
}
