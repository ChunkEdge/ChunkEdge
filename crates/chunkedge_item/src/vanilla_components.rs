use valence_generated::item::ItemKind;

use crate::components::Patchable;
use crate::{ItemComponent, NUM_ITEM_COMPONENTS};

pub(crate) trait ItemKindExt {
    /// Returns the default components for the [`ItemKind`].
    fn default_components(&self) -> [Patchable<Box<ItemComponent>>; NUM_ITEM_COMPONENTS];
}

impl ItemKindExt for ItemKind {
    fn default_components(&self) -> [Patchable<Box<ItemComponent>>; NUM_ITEM_COMPONENTS] {
        // TODO: Implement via buildscript

        [const { Patchable::None }; NUM_ITEM_COMPONENTS]
    }
}
