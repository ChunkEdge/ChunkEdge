use std::borrow::Cow;

use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_ident::Ident;
use uuid::Uuid;

use crate::Packet;

/// Tracks, untracks, or updates a waypoint for the client's locator bar.
///
/// The `identifier` is either a UUID (when `has_uuid`) or a string ID.
/// `icon_color` is an optional packed RGB integer (three bytes).
/// `waypoint_type` selects the `data` shape: 0 = empty, 1 = block position
/// (`data_pos`), 2 = chunk position (`data_chunk_x`, `data_chunk_z`),
/// 3 = azimuth angle (`data_azimuth`).
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct WaypointS2c<'a> {
    pub operation: VarInt,
    pub has_uuid: bool,
    pub uuid: Option<Uuid>,
    pub id: Option<Ident<Cow<'a, str>>>,
    pub icon_style: Ident<Cow<'a, str>>,
    pub icon_color: Option<i32>,
    pub waypoint_type: VarInt,
    pub data_pos: Option<(VarInt, VarInt, VarInt)>,
    pub data_chunk_x: Option<VarInt>,
    pub data_chunk_z: Option<VarInt>,
    pub data_azimuth: Option<f32>,
}
