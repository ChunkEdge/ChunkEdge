#![allow(clippy::all)] // TODO: block build script creates many warnings.

use std::fmt;
use std::fmt::Display;
use std::iter::FusedIterator;

use chunkedge_ident::{Ident, ident};

use crate::item::ItemKind;

include!(concat!(env!("OUT_DIR"), "/block.rs"));

/// A block's collision shape in vanilla's voxel representation: a grid whose
/// lines on each axis are given by a sorted coordinate list, plus the set of
/// full cells in that grid.
///
/// Vanilla's collision code scans this grid cell by cell, so lines inside a
/// solid region (for example the 1/8 steps of a snow layer) can stop movement.
/// Reproducing vanilla collisions exactly needs this representation rather
/// than the boxes from [`BlockState::collision_shapes`].
#[derive(Debug, PartialEq)]
pub struct VoxelShape {
    x: &'static [f64],
    y: &'static [f64],
    z: &'static [f64],
    full: &'static [u8],
}

/// An axis of a [`VoxelShape`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl VoxelShape {
    /// The grid coordinates along `axis`, ascending, in block-local space.
    /// There is one more coordinate than there are cells on that axis.
    pub const fn coords(&self, axis: Axis) -> &'static [f64] {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
            Axis::Z => self.z,
        }
    }

    /// The number of cells along `axis`.
    pub const fn size(&self, axis: Axis) -> usize {
        self.coords(axis).len() - 1
    }

    /// Whether the cell at the given grid indices is full. Indices outside the
    /// grid are never full, like vanilla's `DiscreteVoxelShape#isFullWide`.
    pub fn is_full(&self, x: i32, y: i32, z: i32) -> bool {
        let (Ok(x), Ok(y), Ok(z)) = (usize::try_from(x), usize::try_from(y), usize::try_from(z))
        else {
            return false;
        };
        let (size_x, size_y, size_z) = (self.size(Axis::X), self.size(Axis::Y), self.size(Axis::Z));
        if x >= size_x || y >= size_y || z >= size_z {
            return false;
        }

        let index = (x * size_y + y) * size_z + z;
        self.full[index / 8] & (1 << (index % 8)) != 0
    }

    /// Whether no cell is full.
    pub fn is_empty(&self) -> bool {
        self.full.iter().all(|&byte| byte == 0)
    }
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

    /// Lists every full cell as `(x, y, z)` grid indices.
    fn full_cells(shape: &VoxelShape) -> Vec<(i32, i32, i32)> {
        let size = |axis| i32::try_from(shape.size(axis)).unwrap();
        let mut cells = vec![];
        for x in 0..size(Axis::X) {
            for y in 0..size(Axis::Y) {
                for z in 0..size(Axis::Z) {
                    if shape.is_full(x, y, z) {
                        cells.push((x, y, z));
                    }
                }
            }
        }
        cells
    }

    #[test]
    fn voxel_shape_of_air_is_empty() {
        let shape = BlockState::AIR.collision_voxel_shape();
        assert!(shape.is_empty());
        assert_eq!(shape.coords(Axis::Y), &[0.0]);
        assert!(!shape.is_full(0, 0, 0));
    }

    #[test]
    fn voxel_shape_of_full_block() {
        let shape = BlockState::STONE.collision_voxel_shape();
        for axis in [Axis::X, Axis::Y, Axis::Z] {
            assert_eq!(shape.coords(axis), &[0.0, 1.0]);
        }
        assert_eq!(full_cells(shape), [(0, 0, 0)]);
        assert!(!shape.is_full(-1, 0, 0));
        assert!(!shape.is_full(1, 0, 0));
    }

    #[test]
    fn voxel_shape_of_bottom_slab() {
        let slab = BlockState::SMOOTH_STONE_SLAB.set(PropName::Type, PropValue::Bottom);
        let shape = slab.collision_voxel_shape();
        assert_eq!(shape.coords(Axis::X), &[0.0, 1.0]);
        assert_eq!(shape.coords(Axis::Y), &[0.0, 0.5, 1.0]);
        assert_eq!(shape.coords(Axis::Z), &[0.0, 1.0]);
        assert_eq!(full_cells(shape), [(0, 0, 0)]);
    }

    #[test]
    fn voxel_shape_of_stairs() {
        let stairs = BlockState::OAK_STAIRS
            .set(PropName::Facing, PropValue::North)
            .set(PropName::Half, PropValue::Bottom)
            .set(PropName::Shape, PropValue::Straight);
        let shape = stairs.collision_voxel_shape();
        assert_eq!(shape.coords(Axis::X), &[0.0, 1.0]);
        assert_eq!(shape.coords(Axis::Y), &[0.0, 0.5, 1.0]);
        assert_eq!(shape.coords(Axis::Z), &[0.0, 0.5, 1.0]);
        // The full bottom half, then the upper step on the north (-z) side.
        assert_eq!(full_cells(shape), [(0, 0, 0), (0, 0, 1), (0, 1, 0)]);
    }

    #[test]
    fn voxel_shape_of_snow_keeps_eighth_grid_lines() {
        // Two snow layers collide as one eighth of a block, on an 8-cell grid.
        let snow = BlockState::SNOW.set(PropName::Layers, PropValue::_2);
        let shape = snow.collision_voxel_shape();
        assert_eq!(
            shape.coords(Axis::Y),
            &[0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875, 1.0]
        );
        assert_eq!(full_cells(shape), [(0, 0, 0)]);

        let snow = BlockState::SNOW.set(PropName::Layers, PropValue::_4);
        assert_eq!(
            full_cells(snow.collision_voxel_shape()),
            [(0, 0, 0), (0, 1, 0), (0, 2, 0)]
        );
    }
}
