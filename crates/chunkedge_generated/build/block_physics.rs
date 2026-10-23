use std::collections::HashMap;

use chunkedge_build_utils::{ident, rerun_if_changed};
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::quote;
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

pub(crate) fn build() -> anyhow::Result<TokenStream> {
    rerun_if_changed(["extracted/block_physics.json"]);

    let TopLevel { blocks } =
        serde_json::from_str(include_str!("../extracted/block_physics.json"))?;

    let friction = f32_accessor(&blocks, |b| b.friction);
    let speed_factor = f32_accessor(&blocks, |b| b.speed_factor);
    let jump_factor = f32_accessor(&blocks, |b| b.jump_factor);
    let bounce_restitution = f32_accessor(&blocks, |b| b.bounce_restitution);

    let offset_arms = blocks
        .iter()
        .filter_map(|block| {
            let offset = block.offset.as_ref()?;
            let kind = ident(block.name.to_pascal_case());
            let max_horizontal = offset.max_horizontal.to_bits();
            let max_vertical = offset.max_vertical.to_bits();
            Some(match offset.kind {
                OffsetKind::Xz => quote! {
                    Self::#kind => Some(BlockOffset::Xz {
                        max_horizontal: f32::from_bits(#max_horizontal),
                    }),
                },
                OffsetKind::Xyz => quote! {
                    Self::#kind => Some(BlockOffset::Xyz {
                        max_horizontal: f32::from_bits(#max_horizontal),
                        max_vertical: f32::from_bits(#max_vertical),
                    }),
                },
            })
        })
        .collect::<TokenStream>();

    Ok(quote! {
        impl BlockKind {
            /// The block's slipperiness (vanilla `Block#getFriction`), `0.6`
            /// for most blocks. Entities standing on it keep this fraction of
            /// their horizontal velocity each tick, before air drag.
            pub const fn friction(self) -> f32 {
                #friction
            }

            /// Horizontal velocity multiplier applied after moving on or in
            /// this block (vanilla `Block#getSpeedFactor`), such as `0.4` for
            /// soul sand and honey.
            pub const fn speed_factor(self) -> f32 {
                #speed_factor
            }

            /// Jump power multiplier (vanilla `Block#getJumpFactor`), such as
            /// `0.5` for honey.
            pub const fn jump_factor(self) -> f32 {
                #jump_factor
            }

            /// Fraction of the vertical velocity an entity keeps when it lands
            /// on this block (vanilla `Block#getBounceRestitution`), such as
            /// `1.0` for slime blocks and `0.75` for beds.
            pub const fn bounce_restitution(self) -> f32 {
                #bounce_restitution
            }

            /// How this block's shapes are randomly shifted depending on its
            /// position, or `None` if they are not.
            pub const fn offset(self) -> Option<BlockOffset> {
                match self {
                    #offset_arms
                    _ => None,
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct TopLevel {
    blocks: Vec<Block>,
}

#[derive(Deserialize)]
struct Block {
    name: String,
    #[serde(deserialize_with = "exact_f32")]
    friction: f32,
    #[serde(deserialize_with = "exact_f32")]
    speed_factor: f32,
    #[serde(deserialize_with = "exact_f32")]
    jump_factor: f32,
    #[serde(deserialize_with = "exact_f32")]
    bounce_restitution: f32,
    offset: Option<Offset>,
}

#[derive(Deserialize)]
struct Offset {
    #[serde(rename = "type")]
    kind: OffsetKind,
    #[serde(deserialize_with = "exact_f32")]
    max_horizontal: f32,
    #[serde(deserialize_with = "exact_f32")]
    max_vertical: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum OffsetKind {
    Xz,
    Xyz,
}

/// Emits a `match` over block kinds returning each block's value, with the
/// most common value as the fallback arm.
fn f32_accessor(blocks: &[Block], value: impl Fn(&Block) -> f32) -> TokenStream {
    let mut counts = HashMap::<u32, usize>::new();
    for block in blocks {
        *counts.entry(value(block).to_bits()).or_default() += 1;
    }
    let (fallback, _) = counts
        .into_iter()
        .max_by_key(|&(bits, count)| (count, bits))
        .expect("there is at least one block");

    let arms = blocks
        .iter()
        .filter(|block| value(block).to_bits() != fallback)
        .map(|block| {
            let kind = ident(block.name.to_pascal_case());
            let bits = value(block).to_bits();
            quote!(Self::#kind => f32::from_bits(#bits),)
        })
        .collect::<TokenStream>();

    quote! {
        match self {
            #arms
            _ => f32::from_bits(#fallback),
        }
    }
}

/// Parses a Java `float` written with `Float.toString`.
///
/// Going through `f64` first, as `serde_json` does for `f32`, rounds twice and
/// can land on a neighbouring float. Parsing the decimal text as `f32` rounds
/// once, which recovers the exact float Java wrote.
fn exact_f32<'de, D: Deserializer<'de>>(deserializer: D) -> Result<f32, D::Error> {
    let raw = <&RawValue>::deserialize(deserializer)?;
    raw.get().parse().map_err(serde::de::Error::custom)
}
