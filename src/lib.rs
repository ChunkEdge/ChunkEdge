#![cfg_attr(
    unstable_doc,
    doc = "**❗ NOTE:** This documentation is sourced from the `main` branch. If you're looking for the most recent stable release, go [here](https://docs.rs/chunkedge/latest/chunkedge/).\n\n---\n"
)]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/ChunkEdge/ChunkEdge/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/ChunkEdge/ChunkEdge/main/assets/logo.svg"
)]
#![deny(
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
    rustdoc::missing_crate_level_docs,
    rustdoc::invalid_codeblock_attributes,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::bare_urls,
    rustdoc::invalid_html_tags
)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unused_lifetimes,
    unused_import_braces,
    unreachable_pub,
    clippy::dbg_macro
)]

use bevy_app::{PluginGroup, PluginGroupBuilder};

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(test)]
mod tests;

#[cfg(feature = "log")]
pub use bevy_log as log;
#[cfg(feature = "advancement")]
pub use chunkedge_advancement as advancement;
#[cfg(feature = "anvil")]
pub use chunkedge_anvil as anvil;
#[cfg(feature = "boss_bar")]
pub use chunkedge_boss_bar as boss_bar;
#[cfg(feature = "command")]
pub use chunkedge_command as command;
#[cfg(feature = "command")]
pub use chunkedge_command_macros as command_macros;
#[cfg(feature = "equipment")]
pub use chunkedge_equipment as equipment;
#[cfg(feature = "inventory")]
pub use chunkedge_inventory as inventory;
#[cfg(feature = "network")]
pub use chunkedge_network as network;
#[cfg(feature = "player_list")]
pub use chunkedge_player_list as player_list;
use chunkedge_registry::RegistryPlugin;
#[cfg(feature = "scoreboard")]
pub use chunkedge_scoreboard as scoreboard;
use chunkedge_server::abilities::AbilitiesPlugin;
use chunkedge_server::action::ActionPlugin;
use chunkedge_server::client::ClientPlugin;
use chunkedge_server::client_command::ClientCommandPlugin;
use chunkedge_server::client_settings::ClientSettingsPlugin;
use chunkedge_server::custom_payload::CustomPayloadPlugin;
use chunkedge_server::entity::hitbox::HitboxPlugin;
use chunkedge_server::entity::EntityPlugin;
use chunkedge_server::event_loop::EventLoopPlugin;
use chunkedge_server::hand_swing::HandSwingPlugin;
use chunkedge_server::interact_block::InteractBlockPlugin;
use chunkedge_server::interact_entity::InteractEntityPlugin;
use chunkedge_server::interact_item::InteractItemPlugin;
use chunkedge_server::keepalive::KeepalivePlugin;
use chunkedge_server::layer::LayerPlugin;
use chunkedge_server::message::MessagePlugin;
use chunkedge_server::movement::MovementPlugin;
use chunkedge_server::op_level::OpLevelPlugin;
pub use chunkedge_server::protocol::status_effects;
use chunkedge_server::resource_pack::ResourcePackPlugin;
use chunkedge_server::status::StatusPlugin;
use chunkedge_server::status_effect::StatusEffectPlugin;
use chunkedge_server::teleport::TeleportPlugin;
pub use chunkedge_server::*;
#[cfg(feature = "weather")]
pub use chunkedge_weather as weather;
#[cfg(feature = "world_border")]
pub use chunkedge_world_border as world_border;
use registry::biome::BiomePlugin;
use registry::dimension_type::DimensionTypePlugin;
pub use {chunkedge_item as item, chunkedge_lang as lang};

/// Contains the most frequently used items in ChunkEdge projects.
///
/// This is usually glob imported like so:
///
/// ```no_run
/// use chunkedge::prelude::*; // Glob import.
///
/// let mut app = App::empty();
/// app.add_systems(Update, || println!("yippee!"));
/// app.update()
/// // ...
/// ```
pub mod prelude {
    pub use bevy_app::prelude::*;
    pub use bevy_ecs; // Needed for bevy_ecs macros to function correctly.
    pub use bevy_ecs::prelude::*;
    #[cfg(feature = "advancement")]
    pub use chunkedge_advancement::{
        event::AdvancementTabChangeEvent, Advancement, AdvancementBundle, AdvancementClientUpdate,
        AdvancementCriteria, AdvancementDisplay, AdvancementFrameType, AdvancementRequirements,
    };
    #[cfg(feature = "equipment")]
    pub use chunkedge_equipment::Equipment;
    #[cfg(feature = "inventory")]
    pub use chunkedge_inventory::{
        CursorItem, Inventory, InventoryKind, InventoryWindow, InventoryWindowMut, OpenInventory,
    };
    #[cfg(feature = "network")]
    pub use chunkedge_network::{
        ConnectionMode, ErasedNetworkCallbacks, NetworkCallbacks, NetworkSettings, NewClientInfo,
        SharedNetworkState,
    };
    #[cfg(feature = "player_list")]
    pub use chunkedge_player_list::{PlayerList, PlayerListEntry};
    pub use chunkedge_registry::biome::{Biome, BiomeId, BiomeRegistry};
    pub use chunkedge_registry::dimension_type::{DimensionType, DimensionTypeRegistry};
    pub use chunkedge_server::action::{DiggingEvent, DiggingState};
    pub use chunkedge_server::block::{BlockKind, BlockState, PropName, PropValue};
    pub use chunkedge_server::client::{
        despawn_disconnected_clients, Client, Ip, OldView, OldViewDistance, Properties, Username,
        View, ViewDistance, VisibleChunkLayer, VisibleEntityLayers,
    };
    pub use chunkedge_server::client_command::{
        JumpWithHorseEvent, JumpWithHorseState, LeaveBedEvent, PlayerCommand, SneakEvent,
        SneakState, SprintEvent, SprintState,
    };
    pub use chunkedge_server::entity::hitbox::{Hitbox, HitboxShape};
    pub use chunkedge_server::entity::{
        EntityAnimation, EntityKind, EntityLayerId, EntityManager, EntityStatus, HeadYaw, Look,
        OldEntityLayerId, OldPosition, Position,
    };
    pub use chunkedge_server::event_loop::{
        EventLoopPostUpdate, EventLoopPreUpdate, EventLoopUpdate,
    };
    pub use chunkedge_server::ident::Ident;
    pub use chunkedge_server::interact_entity::{EntityInteraction, InteractEntityEvent};
    pub use chunkedge_server::layer::chunk::{
        Block, BlockRef, Chunk, ChunkLayer, LoadedChunk, UnloadedChunk,
    };
    pub use chunkedge_server::layer::{EntityLayer, LayerBundle};
    pub use chunkedge_server::math::{DVec2, DVec3, Vec2, Vec3};
    pub use chunkedge_server::message::SendMessage as _;
    pub use chunkedge_server::nbt::Compound;
    pub use chunkedge_server::protocol::packets::play::level_particles_s2c::Particle;
    pub use chunkedge_server::protocol::text::{Color, IntoText, Text};
    pub use chunkedge_server::protocol::RegistryId;
    pub use chunkedge_server::spawn::{
        ClientSpawnQuery, ClientSpawnQueryReadOnly, RespawnPosition,
    };
    pub use chunkedge_server::title::SetTitle as _;
    pub use chunkedge_server::{
        ident, BlockPos, ChunkPos, ChunkView, Despawned, Direction, GameMode, Hand, ItemKind,
        ItemStack, Server, UniqueId,
    };
    pub use uuid::Uuid;

    pub use super::DefaultPlugins;
}

/// This plugin group will add all the default plugins for a ChunkEdge
/// application.
///
/// [`DefaultPlugins`] obeys Cargo feature flags. Users may exert control over
/// this plugin group by disabling `default-features` in their `Cargo.toml` and
/// enabling only those features that they wish to use.
pub struct DefaultPlugins;

impl PluginGroup for DefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        #[allow(unused_mut)]
        let mut group = PluginGroupBuilder::start::<Self>()
            .add(ServerPlugin)
            .add(RegistryPlugin)
            .add(BiomePlugin)
            .add(DimensionTypePlugin)
            .add(EntityPlugin)
            .add(HitboxPlugin)
            .add(LayerPlugin)
            .add(ClientPlugin)
            .add(EventLoopPlugin)
            .add(MovementPlugin)
            .add(ClientCommandPlugin)
            .add(KeepalivePlugin)
            .add(InteractEntityPlugin)
            .add(ClientSettingsPlugin)
            .add(ActionPlugin)
            .add(TeleportPlugin)
            .add(MessagePlugin)
            .add(CustomPayloadPlugin)
            .add(HandSwingPlugin)
            .add(InteractBlockPlugin)
            .add(InteractItemPlugin)
            .add(OpLevelPlugin)
            .add(ResourcePackPlugin)
            .add(StatusPlugin)
            .add(StatusEffectPlugin)
            .add(AbilitiesPlugin);

        #[cfg(feature = "log")]
        {
            group = group.add(bevy_log::LogPlugin::default())
        }

        #[cfg(feature = "network")]
        {
            group = group.add(chunkedge_network::NetworkPlugin)
        }

        #[cfg(feature = "player_list")]
        {
            group = group.add(chunkedge_player_list::PlayerListPlugin)
        }

        #[cfg(feature = "equipment")]
        {
            group = group.add(chunkedge_equipment::EquipmentPlugin)
        }

        #[cfg(feature = "inventory")]
        {
            group = group.add(chunkedge_inventory::InventoryPlugin)
        }

        #[cfg(feature = "anvil")]
        {
            group = group.add(chunkedge_anvil::AnvilPlugin)
        }

        #[cfg(feature = "advancement")]
        {
            group = group.add(chunkedge_advancement::AdvancementPlugin)
        }

        #[cfg(feature = "weather")]
        {
            group = group.add(chunkedge_weather::WeatherPlugin)
        }

        #[cfg(feature = "world_border")]
        {
            group = group.add(chunkedge_world_border::WorldBorderPlugin)
        }

        #[cfg(feature = "boss_bar")]
        {
            group = group.add(chunkedge_boss_bar::BossBarPlugin)
        }

        #[cfg(feature = "command")]
        {
            group = group.add(chunkedge_command::manager::CommandPlugin)
        }

        #[cfg(feature = "scoreboard")]
        {
            group = group.add(chunkedge_scoreboard::ScoreboardPlugin)
        }

        group
    }
}
