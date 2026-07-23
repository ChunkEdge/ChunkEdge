use chunkedge_binary::{Decode, Encode};
use chunkedge_math::IVec3;

use crate::Packet;
use crate::sound::{SoundCategory, SoundId};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SoundS2c {
    pub id: SoundId,
    pub category: SoundCategory,
    pub position: IVec3,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}
