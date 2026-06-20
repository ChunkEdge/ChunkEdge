use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct RecipeBookRemoveS2c {
    pub recipes: Vec<VarInt>,
}
