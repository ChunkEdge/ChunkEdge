use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode, IDSet, VarInt};
use chunkedge_generated::registry_id::RegistryId;
use chunkedge_ident::Ident;
use chunkedge_item::ItemStackTemplate;

use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct UpdateRecipesS2c<'a> {
    pub property_sets: Vec<PropertySet<'a>>,
    pub stonecutter_recipes: Vec<StonecutterRecipe>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct PropertySet<'a> {
    pub id: Ident<Cow<'a, str>>,
    pub items: Vec<VarInt>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct StonecutterRecipe {
    pub ingredients: IDSet,
    pub result: SlotDisplay,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum SlotDisplay {
    Empty,
    AnyFuel,
    WithAnyPotion {
        base: Box<SlotDisplay>,
    },
    OnlyWithComponent {
        base: Box<SlotDisplay>,
        /// Raw ID in the data-component-type registry.
        component_type: VarInt,
    },
    Item(RegistryId),
    ItemStack(Box<ItemStackTemplate>),
    Tag(chunkedge_binary::IDSet),
    DyedSlotDemo {
        dye: Box<SlotDisplay>,
        target: Box<SlotDisplay>,
    },
    SmithingTrim {
        base: Box<SlotDisplay>,
        material: Box<SlotDisplay>,
        pattern: RegistryId, // ID in trim_pattern registry
    },
    WithRemainder {
        ingredient: Box<SlotDisplay>,
        remainder: Box<SlotDisplay>,
    },
    Composite(Vec<SlotDisplay>),
}
