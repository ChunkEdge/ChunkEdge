use chunkedge_binary::{Decode, Encode, IDSet, OptionalVarInt, VarInt};
use chunkedge_generated::registry_id::RegistryId;

use crate::Packet;
use crate::packets::play::update_recipes_s2c::SlotDisplay;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct RecipeBookAddS2c {
    pub recipes: Vec<RecipeEntry>,
    pub replace: bool,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct RecipeEntry {
    pub id: VarInt,
    pub display: RecipeDisplay,
    pub group: OptionalVarInt,
    pub category: RegistryId,
    pub ingredients: Option<Vec<IDSet>>,
    // 0x01: show notification; 0x02: highlight as new
    pub flags: u8,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct RecipeDisplay {
    pub kind: RecipeDisplayKind,
}

#[derive(Clone, Debug, Encode, Decode)]
pub enum RecipeDisplayKind {
    CraftingShapeless {
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    CraftingShaped {
        width: VarInt,
        height: VarInt,
        ingredients: Vec<SlotDisplay>,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    Furnace {
        ingredient: SlotDisplay,
        fuel: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
        duration: VarInt,
        experience: f32,
    },
    Stonecutter {
        ingredient: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
    Smithing {
        template: SlotDisplay,
        base: SlotDisplay,
        addition: SlotDisplay,
        result: SlotDisplay,
        crafting_station: SlotDisplay,
    },
}
