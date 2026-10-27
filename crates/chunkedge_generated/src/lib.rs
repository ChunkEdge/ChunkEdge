#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::needless_raw_strings)]

pub mod block;
pub mod registry_id;

/// Raw Minecraft extractor output shared with downstream build scripts.
pub mod extracted {
    /// Miscellaneous registries and entity metadata as JSON.
    pub const MISC_JSON: &str = include_str!("../extracted/misc.json");

    /// Packet names, directions, states, and IDs as JSON.
    pub const PACKETS_JSON: &str = include_str!("../extracted/packets.json");
}

pub mod attributes {
    include!(concat!(env!("OUT_DIR"), "/attributes.rs"));
}

pub mod item {
    include!(concat!(env!("OUT_DIR"), "/item.rs"));
}

/// Vanilla item component type IDs extracted from the Minecraft registry.
pub mod item_component_type {
    include!(concat!(env!("OUT_DIR"), "/item_component_type.rs"));
}

pub mod sound {
    include!(concat!(env!("OUT_DIR"), "/sound.rs"));
}

/// Contains constants for every vanilla packet ID.
pub mod packet_id {
    include!(concat!(env!("OUT_DIR"), "/packet_id.rs"));
}

/// Vanilla particle type IDs extracted from the Minecraft registry.
pub mod particle_type {
    include!(concat!(env!("OUT_DIR"), "/particle_type.rs"));
}

pub mod chunk_view {
    include!(concat!(env!("OUT_DIR"), "/chunk_view.rs"));
}

pub mod status_effects {
    include!(concat!(env!("OUT_DIR"), "/status_effects.rs"));
}

/// Vanilla command argument type IDs extracted from the Minecraft registry.
pub mod command_argument_type {
    include!(concat!(env!("OUT_DIR"), "/command_argument_type.rs"));
}
