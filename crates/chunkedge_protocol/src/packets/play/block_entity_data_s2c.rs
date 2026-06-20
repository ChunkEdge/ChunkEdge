use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode};
use chunkedge_generated::block::BlockEntityKind;
use chunkedge_nbt::Compound;

use crate::{BlockPos, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct BlockEntityDataS2c<'a> {
    pub location: BlockPos,
    pub kind: BlockEntityKind,
    pub data: Cow<'a, Compound>,
}
