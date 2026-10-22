use std::borrow::Cow;

use chunkedge_binary::{Decode, Either, Encode, VarInt};
use chunkedge_ident::Ident;
use uuid::Uuid;

use crate::Packet;

/// Tracks, untracks, or updates a waypoint for the client's locator bar.
#[derive(Clone, Debug, PartialEq, Encode, Decode, Packet)]
pub struct WaypointS2c<'a> {
    pub operation: WaypointOperation,
    /// Something that uniquely identifies this specific waypoint.
    pub identifier: WaypointIdentifier<'a>,
    /// Path to the waypoint style JSON: `assets/<namespace>/waypoint_style/<value>.json`.
    pub icon_style: Ident<Cow<'a, str>>,
    /// The color of the waypoint icon.
    pub icon_color: Option<[u8; 3]>,
    pub data: WaypointData,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode)]
pub enum WaypointOperation {
    Track,
    Untrack,
    Update,
}

/// A waypoint identifier is either a UUID or a string.
/// This uniquely identifies this specific waypoint.
pub type WaypointIdentifier<'a> = Either<Uuid, Cow<'a, str>>;

#[derive(Copy, Clone, Debug, PartialEq, Encode, Decode)]
pub enum WaypointData {
    Empty,
    Position {
        x: VarInt,
        y: VarInt,
        z: VarInt,
    },
    Chunk {
        x: VarInt,
        z: VarInt,
    },
    /// Angle in radians, encoded as a big-endian float.
    Azimuth(f32),
}
