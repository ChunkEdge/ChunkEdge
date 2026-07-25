use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;
use crate::sound::{SoundCategory, SoundId};
#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SoundEntityS2c {
    pub id: SoundId,
    pub category: SoundCategory,
    pub entity_id: VarInt,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}
