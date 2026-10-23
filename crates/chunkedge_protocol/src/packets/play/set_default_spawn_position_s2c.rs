use chunkedge_binary::{Decode, Encode};

use crate::{GlobalPos, Packet};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SetDefaultSpawnPositionS2c<'a> {
    pub position: GlobalPos<'a>,
    pub yaw: f32,
    pub pitch: f32,
}
