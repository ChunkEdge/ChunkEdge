//! Field codec for nullable network NBT compounds.
//!
//! Absence is a single `TAG_End` byte, rather than an `Option` boolean prefix.
use std::io::Write;

use chunkedge_nbt::Compound;

use crate::Encode;

pub fn encode<W: Write>(nbt: &Option<Compound>, w: W) -> anyhow::Result<()> {
    match nbt {
        Some(nbt) => nbt.encode(w),
        None => 0_u8.encode(w),
    }
}

pub fn decode(r: &mut &[u8]) -> anyhow::Result<Option<Compound>> {
    if r.first() == Some(&0) {
        *r = &r[1..];
        Ok(None)
    } else {
        Ok(Some(chunkedge_nbt::from_unnamed_binary(r)?))
    }
}
