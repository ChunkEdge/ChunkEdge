use std::borrow::Cow;

use valence_binary::{Decode, Encode, IDSet, VarInt};
use valence_generated::registry_id::RegistryId;
use valence_ident::Ident;
use valence_item::ItemStack;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct UpdateRecipesS2c<'a> {
    pub property_sets: Vec<PropertySet<'a>>,
    pub stonecutter_recipes: Vec<StonecutterRecipe<'a>>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct PropertySet<'a> {
    pub id: Ident<Cow<'a, str>>,
    pub items: Vec<VarInt>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct StonecutterRecipe<'a> {
    pub ingredients: IDSet,
    pub result: SlotDisplay<'a>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SlotDisplay<'a> {
    Empty,
    AnyFuel,
    Item(RegistryId),
    ItemStack(Box<ItemStack>),
    Tag(Ident<Cow<'a, str>>),
    SmithingTrim {
        base: Box<SlotDisplay<'a>>,
        material: Box<SlotDisplay<'a>>,
        pattern: RegistryId, // ID in trim_pattern registry
    },
    WithRemainder {
        ingredient: Box<SlotDisplay<'a>>,
        remainder: Box<SlotDisplay<'a>>,
    },
    Composite(Vec<SlotDisplay<'a>>),
}
