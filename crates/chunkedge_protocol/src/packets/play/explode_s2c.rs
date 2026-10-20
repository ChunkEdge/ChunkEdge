use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_math::DVec3;

use crate::sound::SoundId;
use crate::{Packet, Particle};

#[derive(Clone, Debug, Encode, Decode, Packet)]
pub struct ExplodeS2c {
    pub pos: DVec3,
    pub radius: f32,
    pub block_count: i32,
    pub player_motion: Option<DVec3>,
    pub particle: Particle,
    pub sound: SoundId,
    pub block_particles: Vec<WeightedExplosionParticle>,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct WeightedExplosionParticle {
    pub info: ExplosionParticleInfo,
    pub weight: VarInt,
}

#[derive(Clone, Debug, Encode, Decode)]
pub struct ExplosionParticleInfo {
    pub particle: Particle,
    pub scaling: f32,
    pub speed: f32,
}
