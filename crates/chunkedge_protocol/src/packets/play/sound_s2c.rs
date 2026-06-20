use valence_binary::{Decode, Encode};
use valence_math::IVec3;

use crate::sound::{SoundCategory, SoundId};
use crate::Packet;

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct SoundS2c {
    pub id: SoundId,
    pub category: SoundCategory,
    pub position: IVec3,
    pub volume: f32,
    pub pitch: f32,
    pub seed: i64,
}
