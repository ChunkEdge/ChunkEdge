#![allow(clippy::all)] // TODO: block build script creates many warnings.

use std::fmt;
use std::fmt::Display;
use std::iter::FusedIterator;

use chunkedge_ident::{Ident, ident};

use crate::item::ItemKind;

include!(concat!(env!("OUT_DIR"), "/block.rs"));
include!(concat!(env!("OUT_DIR"), "/block_physics.rs"));

impl BlockState {
    /// The offset vanilla adds to this block's shapes at the block position
    /// `(x, _, z)`, or zero if the block isn't offset (vanilla
    /// `BlockStateBase#getOffset`). The height of the position never matters.
    pub fn offset_at(self, x: i32, z: i32) -> chunkedge_math::DVec3 {
        self.to_kind()
            .offset()
            .map_or(chunkedge_math::DVec3::ZERO, |offset| offset.at(x, z))
    }
}

/// How a block's shapes are pseudo-randomly shifted depending on its position,
/// such as flowers, bamboo and pointed dripstone (vanilla
/// `BlockBehaviour.OffsetType`).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockOffset {
    /// Shifted horizontally by at most `max_horizontal` on each axis.
    Xz { max_horizontal: f32 },
    /// Shifted horizontally like [`BlockOffset::Xz`] and down by at most
    /// `max_vertical`.
    Xyz {
        max_horizontal: f32,
        max_vertical: f32,
    },
}

impl BlockOffset {
    /// The offset at the block position `(x, _, z)`, bit-identical to the
    /// vanilla offset function.
    pub fn at(self, x: i32, z: i32) -> chunkedge_math::DVec3 {
        // Vanilla divides a 4-bit slice of the seed as `float`, then widens it
        // to `double` for the rest of the arithmetic.
        let nibble = |bits: i64| f64::from((bits & 15) as f32 / 15.0);

        let seed = position_seed(x, 0, z);
        let (max_horizontal, y) = match self {
            Self::Xz { max_horizontal } => (max_horizontal, 0.0),
            Self::Xyz {
                max_horizontal,
                max_vertical,
            } => (
                max_horizontal,
                (nibble(seed >> 4) - 1.0) * f64::from(max_vertical),
            ),
        };
        let max_horizontal = f64::from(max_horizontal);
        let horizontal =
            |bits: i64| ((nibble(bits) - 0.5) * 0.5).clamp(-max_horizontal, max_horizontal);

        chunkedge_math::DVec3::new(horizontal(seed), y, horizontal(seed >> 8))
    }
}

/// Vanilla `Mth.getSeed`. The `x` term is multiplied as a 32-bit `int` and
/// wraps before it is widened, unlike the `z` term.
fn position_seed(x: i32, y: i32, z: i32) -> i64 {
    let seed = i64::from(x.wrapping_mul(3_129_871))
        ^ i64::from(z).wrapping_mul(116_129_781)
        ^ i64::from(y);
    seed.wrapping_mul(seed)
        .wrapping_mul(42_317_861)
        .wrapping_add(seed.wrapping_mul(11))
        >> 16
}

impl fmt::Debug for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_block_state(*self, f)
    }
}

impl Display for BlockState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_block_state(*self, f)
    }
}

fn fmt_block_state(bs: BlockState, f: &mut fmt::Formatter) -> fmt::Result {
    let kind = bs.to_kind();

    write!(f, "{}", kind.to_str())?;

    let props = kind.props();

    if !props.is_empty() {
        let mut list = f.debug_list();
        for &p in kind.props() {
            struct KeyVal<'a>(&'a str, &'a str);

            impl<'a> fmt::Debug for KeyVal<'a> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{}={}", self.0, self.1)
                }
            }

            list.entry(&KeyVal(p.to_str(), bs.get(p).unwrap().to_str()));
        }
        list.finish()
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_consistency() {
        for kind in BlockKind::ALL {
            let block = kind.to_state();

            for &prop in kind.props() {
                let new_block = block.set(prop, block.get(prop).unwrap());
                assert_eq!(new_block, block);
            }
        }
    }

    #[test]
    fn blockstate_to_wall() {
        assert_eq!(BlockState::STONE.wall_block_id(), None);
        assert_eq!(
            BlockState::OAK_SIGN.wall_block_id(),
            Some(BlockState::OAK_WALL_SIGN)
        );
        assert_eq!(
            BlockState::GREEN_BANNER.wall_block_id(),
            Some(BlockState::GREEN_WALL_BANNER)
        );
        assert_ne!(
            BlockState::GREEN_BANNER.wall_block_id(),
            Some(BlockState::GREEN_BANNER)
        );
    }

    /// Expected values are the bits of the Java `float` literals, so a
    /// rounding mistake between the extractor and Rust can't hide behind a
    /// comparison against a Rust literal parsed the same wrong way.
    #[test]
    fn physics_properties_match_java_float_bits() {
        let cases = [
            (BlockKind::Stone.friction(), 0x3f19999a),
            (BlockKind::Stone.speed_factor(), 0x3f800000),
            (BlockKind::Stone.jump_factor(), 0x3f800000),
            (BlockKind::Stone.bounce_restitution(), 0x00000000),
            (BlockKind::Ice.friction(), 0x3f7ae148),
            (BlockKind::PackedIce.friction(), 0x3f7ae148),
            (BlockKind::BlueIce.friction(), 0x3f7d2f1b),
            (BlockKind::SlimeBlock.friction(), 0x3f4ccccd),
            (BlockKind::SlimeBlock.bounce_restitution(), 0x3f800000),
            (BlockKind::RedBed.bounce_restitution(), 0x3f400000),
            (BlockKind::SoulSand.speed_factor(), 0x3ecccccd),
            (BlockKind::HoneyBlock.speed_factor(), 0x3ecccccd),
            (BlockKind::HoneyBlock.jump_factor(), 0x3f000000),
        ];

        for (index, (value, bits)) in cases.into_iter().enumerate() {
            assert_eq!(value.to_bits(), bits, "case {index}");
        }
    }

    #[test]
    fn offset_definitions() {
        assert_eq!(BlockKind::Stone.offset(), None);
        assert_eq!(
            BlockKind::Bamboo.offset(),
            Some(BlockOffset::Xz {
                max_horizontal: 0.25
            })
        );
        assert_eq!(
            BlockKind::PointedDripstone.offset(),
            Some(BlockOffset::Xz {
                max_horizontal: 0.125
            })
        );
        assert_eq!(
            BlockKind::SmallDripleaf.offset(),
            Some(BlockOffset::Xyz {
                max_horizontal: 0.25,
                max_vertical: 0.1
            })
        );
    }

    #[test]
    fn bamboo_offsets() {
        // (x, z, offset x, offset z), computed by vanilla 26.2.
        let cases = [
            (0, 0, -0.25, -0.25),
            (1, 2, -0.11666665971279144, -0.18333332985639572),
            (-7, 13, -0.04999999701976776, -0.11666665971279144),
            (100000, -3, -0.11666665971279144, 0.0833333432674408),
        ];

        for (x, z, offset_x, offset_z) in cases {
            let offset = BlockState::BAMBOO.offset_at(x, z);
            assert_eq!(offset.x.to_bits(), f64::to_bits(offset_x), "({x}, {z})");
            assert_eq!(offset.y.to_bits(), 0.0_f64.to_bits(), "({x}, {z})");
            assert_eq!(offset.z.to_bits(), f64::to_bits(offset_z), "({x}, {z})");
        }
        assert_eq!(
            BlockState::STONE.offset_at(1, 2),
            chunkedge_math::DVec3::ZERO
        );
    }

    /// Checks every offset block against the offsets vanilla computed for it
    /// while extracting.
    #[test]
    fn offsets_match_vanilla_samples() {
        #[derive(serde::Deserialize)]
        struct TopLevel<'a> {
            #[serde(borrow)]
            blocks: Vec<Block<'a>>,
        }

        #[derive(serde::Deserialize)]
        struct Block<'a> {
            name: &'a str,
            #[serde(borrow)]
            offset: Option<Offset<'a>>,
        }

        #[derive(serde::Deserialize)]
        struct Offset<'a> {
            #[serde(borrow)]
            samples: Vec<Sample<'a>>,
        }

        #[derive(serde::Deserialize)]
        struct Sample<'a> {
            pos: [i32; 3],
            #[serde(borrow)]
            offset: [&'a serde_json::value::RawValue; 3],
        }

        let TopLevel { blocks } =
            serde_json::from_str(include_str!("../extracted/block_physics.json")).unwrap();

        let mut checked = 0;
        for block in blocks {
            let kind = BlockKind::from_str(block.name).unwrap();
            let Some(offset) = block.offset else {
                assert_eq!(kind.offset(), None, "{}", block.name);
                continue;
            };

            for sample in offset.samples {
                let [x, _, z] = sample.pos;
                let actual = kind.offset().unwrap().at(x, z);
                // Parse the decimal text directly: `Double.toString` round-trips.
                let expected = sample.offset.map(|raw| raw.get().parse::<f64>().unwrap());
                assert_eq!(
                    actual.to_array().map(f64::to_bits),
                    expected.map(f64::to_bits),
                    "{} at {:?}",
                    block.name,
                    sample.pos
                );
                checked += 1;
            }
        }
        assert!(checked > 0);
    }
}
