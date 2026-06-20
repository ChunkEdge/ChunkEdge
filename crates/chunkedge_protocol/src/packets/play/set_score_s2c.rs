use std::borrow::Cow;

use valence_binary::{Decode, Encode, TextComponent, VarInt};

use super::set_objective_s2c::NumberFormat;
use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetScoreS2c<'a> {
    //The entity whose score this is. For players, this is their username; for other entities, it
    // is their UUID.
    pub entity_name: &'a str,
    pub objective_name: &'a str,
    pub value: VarInt,
    pub display_name: Option<Cow<'a, TextComponent>>,
    pub number_format: Option<NumberFormat<'a>>,
}
