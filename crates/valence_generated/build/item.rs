use anyhow::Ok;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use valence_build_utils::{ident, rerun_if_changed};
// TODO: Update to support components (also default values for item components)

#[derive(Deserialize, Clone, Debug)]
struct Item {
    id: u16,
    name: String,
    translation_key: String,
    max_stack: i8,
    max_durability: u16,
    enchantability: u8,
    fireproof: bool,
    // TODO: Implement food component
    // food: Option<FoodComponent>,
}

pub(crate) fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/items.json"]);

    let items = serde_json::from_str::<Vec<Item>>(include_str!("../extracted/items.json"))?;

    let item_kind_count = items.len();

    let item_kind_from_raw_id_arms = items
        .iter()
        .map(|item| {
            let id = &item.id;
            let name = ident(item.name.to_pascal_case());

            quote! {
                #id => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_raw_id_arms = items
        .iter()
        .map(|item| {
            let id = &item.id;
            let name = ident(item.name.to_pascal_case());

            quote! {
                Self::#name => #id,
            }
        })
        .collect::<TokenStream>();

    let item_kind_from_str_arms = items
        .iter()
        .map(|item| {
            let str_name = &item.name;
            let name = ident(str_name.to_pascal_case());
            quote! {
                #str_name => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_str_arms = items
        .iter()
        .map(|item| {
            let str_name = &item.name;
            let name = ident(str_name.to_pascal_case());
            quote! {
                Self::#name => #str_name,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_translation_key_arms = items
        .iter()
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let translation_key = &item.translation_key;
            quote! {
                Self::#name => #translation_key,
            }
        })
        .collect::<TokenStream>();

    let item_kind_variants = items
        .iter()
        .map(|item| ident(item.name.to_pascal_case()))
        .collect::<Vec<_>>();

    let item_kind_to_max_stack_arms = items
        .iter()
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let max_stack = item.max_stack;

            quote! {
                Self::#name => #max_stack,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_max_durability_arms = items
        .iter()
        .filter(|item| item.max_durability != 0)
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let max_durability = item.max_durability;

            quote! {
                Self::#name => #max_durability,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_enchantability_arms = items
        .iter()
        .filter(|item| item.enchantability != 0)
        .map(|item| {
            let name = ident(item.name.to_pascal_case());
            let ench = item.enchantability;

            quote! {
                Self::#name => #ench,
            }
        })
        .collect::<TokenStream>();

    let item_kind_to_fireproof_arms = items
        .iter()
        .filter(|item| item.fireproof)
        .map(|item| {
            let name = ident(item.name.to_pascal_case());

            quote! {
                Self::#name => true,
            }
        })
        .collect::<TokenStream>();

    Ok(quote! {
        use crate::registry_id::RegistryId;

        /// Represents an item from the game
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
        #[repr(u16)]
        pub enum ItemKind {
            #[default]
            #(#item_kind_variants,)*
        }

        #[doc = "Contains food information about an item."]
        #[doc = ""]
        #[doc = "Only food items have a food component."]
        #[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
        pub struct FoodComponent {
            pub hunger: u16,
            pub saturation: f32,
            pub always_edible: bool,
            pub meat: bool,
            pub snack: bool,
        }

        impl ItemKind {
            #[doc = "Constructs a item kind from a raw item ID."]
            #[doc = ""]
            #[doc = "If the given ID is invalid, `None` is returned."]
            pub const fn from_raw(id: u16) -> Option<Self> {
                match id {
                    #item_kind_from_raw_id_arms
                    _ => None
                }
            }

            #[doc = "Gets the raw item ID from the item kind"]
            pub const fn to_raw(self) -> u16 {
                match self {
                    #item_kind_to_raw_id_arms
                }
            }

            /// Construct an item kind for its `snake_case` name.
            ///
            /// Returns `None` if the name is invalid.
            #[allow(clippy::should_implement_trait)]
            pub fn from_str(name: &str) -> Option<ItemKind> {
                match name {
                    #item_kind_from_str_arms
                    _ => None
                }
            }

            /// Gets the `snake_case` name of this item kind.
            pub const fn to_str(self) -> &'static str {
                match self {
                    #item_kind_to_str_arms
                }
            }

            #[doc = "Gets the translation key of this item kind."]
            pub const fn translation_key(self) -> &'static str {
                match self {
                    #item_kind_to_translation_key_arms
                }
            }

            #[doc = "Returns the maximum stack count."]
            pub const fn max_stack(self) -> i8 {
                match self {
                    #item_kind_to_max_stack_arms
                }
            }

            #[doc = "Returns the maximum durability before the item will break."]
            #[doc = ""]
            #[doc = "If the item doesn't have durability, `0` is returned."]
            pub const fn max_durability(self) -> u16 {
                match self {
                    #item_kind_to_max_durability_arms
                    _ => 0,
                }
            }

            #[doc = "Returns the enchantability of the item kind."]
            #[doc = ""]
            #[doc = "If the item doesn't have durability, `0` is returned."]
            pub const fn enchantability(self) -> u8 {
                match self {
                    #item_kind_to_enchantability_arms
                    _ => 0,
                }
            }

            #[doc = "Returns if the item can survive in fire/lava."]
            pub const fn fireproof(self) -> bool {
                #[allow(clippy::match_like_matches_macro)]
                match self {
                    #item_kind_to_fireproof_arms
                    _ => false
                }
            }

            #[doc = "An array of all item kinds."]
            pub const ALL: [Self; #item_kind_count] = [#(Self::#item_kind_variants,)*];
        }

        impl From<ItemKind> for RegistryId {
            fn from(item: ItemKind) -> Self {
                RegistryId::new(i32::from(item.to_raw()))
            }
        }
    })
}
