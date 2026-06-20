use std::fmt::Debug;
use std::io::Write;

use valence_binary::{Encode, VarInt};
use valence_generated::item::ItemKind;

use crate::components::{ItemComponent, Patchable};
use crate::vanilla_components::ItemKindExt;
use crate::NUM_ITEM_COMPONENTS;

/// A stack of items in an inventory.
#[derive(Clone, PartialEq)]
pub struct ItemStack {
    pub item: ItemKind,
    pub count: i8,
    pub(crate) components: [Patchable<Box<ItemComponent>>; NUM_ITEM_COMPONENTS],
}

impl Default for ItemStack {
    fn default() -> Self {
        ItemStack::EMPTY
    }
}

impl Debug for ItemStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemStack")
            .field("item", &self.item)
            .field("count", &self.count)
            .field(
                "components",
                &self
                    .components
                    .iter()
                    .enumerate()
                    .filter_map(|(i, c)| c.as_option().map(|comp| (i, comp)))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct HashedItemStack {
    pub item: ItemKind,
    pub count: i8,
    pub(crate) components: [Patchable<()>; NUM_ITEM_COMPONENTS],
}
impl HashedItemStack {
    pub const EMPTY: Self = Self {
        item: ItemKind::Air,
        count: 0,
        components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
    };

    #[must_use]
    pub const fn new(item: ItemKind, count: i8) -> Self {
        Self {
            item,
            count,
            components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
        }
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.item, ItemKind::Air) || self.count <= 0
    }
}

impl From<ItemStack> for HashedItemStack {
    fn from(stack: ItemStack) -> Self {
        Self {
            item: stack.item,
            count: stack.count,
            components: stack.components.map(|c| match c {
                Patchable::Default(_) => Patchable::Default(()),
                Patchable::Added((_, h)) => Patchable::Added(((), h)),
                Patchable::Removed => Patchable::Removed,
                Patchable::None => Patchable::None,
            }),
        }
    }
}

impl ItemStack {
    pub const EMPTY: ItemStack = ItemStack {
        item: ItemKind::Air,
        count: 0,
        components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
    };

    /// Creates a new item stack without any components.
    #[must_use]
    pub const fn new(item: ItemKind, count: i8) -> Self {
        Self {
            item,
            count,
            components: [const { Patchable::None }; NUM_ITEM_COMPONENTS],
        }
    }

    /// Creates a new item stack with the vanilla default components for the
    /// given [`ItemKind`].
    pub fn new_vanilla(item: ItemKind, count: i8) -> Self {
        let components = item.default_components();
        Self {
            item,
            count,
            components,
        }
    }

    /// Read the components of the item stack.
    pub fn components(&self) -> Vec<&ItemComponent> {
        self.components
            .iter()
            .filter_map(|component| component.as_option())
            .map(|boxed| &**boxed)
            .collect()
    }

    /// Returns the default components for the [`ItemKind`].
    pub fn default_components(&self) -> Vec<ItemComponent> {
        self.item
            .default_components()
            .iter()
            .filter_map(|component| component.as_option().map(|b| &**b))
            .cloned()
            .collect()
    }

    /// Attach a component to the item stack.
    pub fn insert_component(&mut self, component: ItemComponent) {
        let id = component.id() as usize;
        if let Patchable::Default(default) = &self.components[id] {
            // We don't need to add a components if its default for the item kind.
            if **default == component {
                return;
            }
        }

        let hash = component.hash();
        self.components[id] = Patchable::Added((Box::new(component), hash));
    }

    /// Remove a component from the item stack by its ID, see
    /// [`ItemComponent::id`].String
    ///
    /// Returns the removed component if it was present, otherwise `None`.
    pub fn remove_component<I: Into<usize>>(&mut self, id: I) -> Option<ItemComponent> {
        let id = id.into();
        if id < NUM_ITEM_COMPONENTS {
            std::mem::replace(&mut self.components[id], Patchable::Removed)
                .to_option()
                .map(|boxed| *boxed)
        } else {
            None
        }
    }

    /// Get a specific component by its ID, see [`ItemComponent::id`].
    pub fn get_component<I: Into<usize>>(&self, id: I) -> Option<&ItemComponent> {
        let id = id.into();
        if id < NUM_ITEM_COMPONENTS {
            match &self.components[id] {
                Patchable::Added((component, _)) | Patchable::Default(component) => {
                    Some(&**component)
                }
                _ => None,
            }
        } else {
            None
        }
    }

    #[must_use]
    pub const fn with_count(mut self, count: i8) -> Self {
        self.count = count;
        self
    }

    #[must_use]
    pub const fn with_item(mut self, item: ItemKind) -> Self {
        self.item = item;
        self
    }

    #[must_use]
    pub fn with_components(mut self, components: Vec<ItemComponent>) -> Self {
        for component in components {
            self.insert_component(component);
        }
        self
    }

    pub const fn is_empty(&self) -> bool {
        matches!(self.item, ItemKind::Air) || self.count <= 0
    }

    pub fn encode_recursive<W: Write>(
        &self,
        mut w: W,
        prefixed: bool,
    ) -> Result<(), anyhow::Error> {
        if self.is_empty() {
            VarInt(0).encode(w)
        } else {
            // Break recursion loop by erasing the type
            let w: &mut dyn Write = &mut w;

            VarInt(i32::from(self.count)).encode(&mut *w)?;
            self.item.encode(&mut *w)?;

            let mut added = Vec::new();
            let mut removed = Vec::new();

            for (i, patch) in self.components.iter().enumerate() {
                match patch {
                    Patchable::Added((comp, _)) => added.push((i, comp)),
                    Patchable::Removed => removed.push(i),
                    _ => {}
                }
            }

            // Encode Added & removed
            VarInt(added.len() as i32).encode(&mut *w)?;
            VarInt(removed.len() as i32).encode(&mut *w)?;

            for (id, comp) in added {
                VarInt(id as i32).encode(&mut *w)?;
                if prefixed {
                    // We need to record the length of the component data.
                    // Then we encode len then the data.
                    //
                    // We use a dummy writer to avoid allocator pressure at the cost of cpu.

                    struct ByteCounter {
                        count: usize,
                    }

                    impl Write for ByteCounter {
                        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                            self.count += buf.len();
                            Ok(buf.len())
                        }

                        fn flush(&mut self) -> std::io::Result<()> {
                            Ok(())
                        }
                    }

                    // Encode to the counter to determine the length
                    let mut counter = ByteCounter { count: 0 };
                    comp.encode(&mut counter)?;

                    // Write the length prefix
                    VarInt(counter.count as i32).encode(&mut *w)?;

                    // Real run: Encode the data to the actual writer
                    comp.encode(&mut *w)?;
                } else {
                    comp.encode(&mut *w)?;
                }
            }

            for id in removed {
                VarInt(id as i32).encode(&mut *w)?;
            }

            Ok(())
        }
    }
}
