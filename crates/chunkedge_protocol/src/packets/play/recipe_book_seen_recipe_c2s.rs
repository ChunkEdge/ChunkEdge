use valence_binary::{Decode, Encode, VarInt};

use crate::Packet;
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct RecipeBookSeenRecipeC2s {
    pub recipe_id: VarInt,
}
