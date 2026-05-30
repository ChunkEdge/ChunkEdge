use valence_binary::{Decode, Encode, VarInt};

use super::recipe_book_add_s2c::RecipeDisplay;
use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct PlaceGhostRecipeS2c<'a> {
    pub window_id: VarInt,
    pub recipe_display: RecipeDisplay<'a>,
}
