#![doc = include_str!("../README.md")]

pub mod abilities;
pub mod action;
pub mod brand;
mod chunk_view;
pub mod client;
pub mod client_command;
pub mod client_settings;
pub mod custom_payload;
pub mod event_loop;
pub mod hand_swing;
pub mod interact_block;
pub mod interact_entity;
pub mod interact_item;
pub mod keepalive;
pub mod layer;
pub mod message;
pub mod movement;
pub mod op_level;
pub mod resource_pack;
pub mod spawn;
pub mod status;
pub mod status_effect;
pub mod teleport;
pub mod title;

pub use bevy_app as app;
pub use bevy_ecs as ecs;
pub use chunk_view::ChunkView;
pub use event_loop::{EventLoopPostUpdate, EventLoopPreUpdate, EventLoopUpdate};
pub use layer::{ChunkLayer, EntityLayer, Layer, LayerBundle};
pub use rand;
pub use valence_entity as entity;
pub use valence_nbt as nbt;
pub use valence_protocol as protocol;
pub use valence_protocol::{
    block, ident, math, text, uuid, BiomePos, BlockPos, BlockState, ChunkPos, CompressionThreshold,
    Difficulty, Direction, GameMode, Hand, Ident, ItemKind, ItemStack, Text, MINECRAFT_VERSION,
    PROTOCOL_VERSION,
};
pub use valence_registry as registry;
pub use valence_server_common::*;
