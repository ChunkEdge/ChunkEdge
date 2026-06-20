use valence_binary::{Decode, Encode, TextComponent, VarInt};

use crate::packets::play::set_structure_block_c2s::Rotation;
use crate::{BlockPos, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct TestInstanceBlockActionC2s {
    pub position: BlockPos,
    pub action: TestInstanceBlockAction,
    /// ID in the `minecraft:test_instance_kind` registry.
    pub test: Option<VarInt>,
    pub size_x: VarInt,
    pub size_y: VarInt,
    pub size_z: VarInt,
    pub rotation: Rotation,
    pub ignore_entities: bool,
    pub status: TestInstanceBlockStatus,
    pub error_message: Option<TextComponent>,
}

#[derive(Copy, Clone, Debug, Encode, Decode)]
pub enum TestInstanceBlockAction {
    Init,
    Query,
    Set,
    Reset,
    Save,
    Export,
    Run,
}

#[derive(Copy, Clone, Debug, Encode, Decode)]
pub enum TestInstanceBlockStatus {
    Cleared,
    Running,
    Finished,
}
