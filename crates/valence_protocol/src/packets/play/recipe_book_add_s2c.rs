use valence_binary::{Decode, Encode, IDSet, VarInt};
use valence_generated::registry_id::RegistryId;

use crate::packets::play::update_recipes_s2c::SlotDisplay;
use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct RecipeBookAddS2c<'a> {
    pub recipes: Vec<RecipeEntry<'a>>,
    pub replace: bool,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct RecipeEntry<'a> {
    pub id: VarInt,
    pub display: RecipeDisplay<'a>,
    pub group: RegistryId,
    pub category: RegistryId,
    pub ingredients: Option<Vec<IDSet>>,
    // 0x01: show notification; 0x02: highlight as new
    pub flags: u8,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct RecipeDisplay<'a> {
    pub kind: RecipeDisplayKind<'a>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum RecipeDisplayKind<'a> {
    CraftingShapeless {
        ingredients: Vec<SlotDisplay<'a>>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
    CraftingShaped {
        width: VarInt,
        height: VarInt,
        ingredients: Vec<SlotDisplay<'a>>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
    Furnace {
        ingredient: SlotDisplay<'a>,
        fuel: SlotDisplay<'a>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
        duration: VarInt,
        experience: f32,
    },
    Stonecutter {
        ingredient: SlotDisplay<'a>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
    Smithing {
        template: SlotDisplay<'a>,
        base: SlotDisplay<'a>,
        addition: SlotDisplay<'a>,
        result: SlotDisplay<'a>,
        crafting_station: SlotDisplay<'a>,
    },
}
