use chunkedge_binary::{Decode, Encode, IdOr};
pub use chunkedge_generated::sound::Sound;
use chunkedge_ident::Ident;

pub type SoundId = IdOr<SoundDirect>;

#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct SoundDirect {
    pub id: Ident<String>,
    pub range: Option<f32>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Encode, Decode)]
pub enum SoundCategory {
    Master,
    Music,
    Record,
    Weather,
    Block,
    Hostile,
    Neutral,
    Player,
    Ambient,
    Voice,
}
