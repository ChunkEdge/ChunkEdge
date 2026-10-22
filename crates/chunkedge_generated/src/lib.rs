#![allow(clippy::unseparated_literal_suffix)]
#![allow(clippy::needless_raw_strings)]

pub mod block;
pub mod registry_id;

pub mod attributes {
    include!(concat!(env!("OUT_DIR"), "/attributes.rs"));
}

pub mod item {
    include!(concat!(env!("OUT_DIR"), "/item.rs"));
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
