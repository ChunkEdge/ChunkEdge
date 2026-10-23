use chunkedge_binary::{Decode, Encode};
use chunkedge_ident::Ident;

use crate::Packet;

/// Synchronizes game rule values to the client.
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct GameRuleValuesS2c {
    pub values: Vec<GameRuleValue>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct GameRuleValue {
    pub name: Ident<String>,
    pub value: String,
}
