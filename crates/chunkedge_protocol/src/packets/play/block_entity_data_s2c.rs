use std::borrow::Cow;

use valence_binary::{Decode, Encode};
use valence_generated::block::BlockEntityKind;
use valence_nbt::Compound;

use crate::{BlockPos, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct BlockEntityDataS2c<'a> {
    pub location: BlockPos,
    pub kind: BlockEntityKind,
    pub data: Cow<'a, Compound>,
}
