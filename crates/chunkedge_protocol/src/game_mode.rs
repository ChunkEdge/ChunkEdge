use std::io::Write;

use anyhow::bail;
use bevy_ecs::prelude::*;
use chunkedge_binary::{Decode, Encode, VarInt};
use derive_more::{From, Into};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, Encode, Decode, Component)]
pub enum GameMode {
    #[default]
    Survival,
    Creative,
    Adventure,
    Spectator,
}

/// An optional [`GameMode`] with `None` encoded as [`VarInt`] `0`. Isomorphic to
/// `Option<GameMode>`.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, From, Into)]
pub struct OptGameMode(pub Option<GameMode>);

impl Encode for OptGameMode {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        match self.0 {
            Some(gm) => VarInt(gm as i32 + 1).encode(w),
            None => VarInt(0).encode(w),
        }
    }
}

impl Decode<'_> for OptGameMode {
    fn decode(r: &mut &'_ [u8]) -> anyhow::Result<Self> {
        Ok(Self(match VarInt::decode(r)?.0 {
            0 => None,
            1 => Some(GameMode::Survival),
            2 => Some(GameMode::Creative),
            3 => Some(GameMode::Adventure),
            4 => Some(GameMode::Spectator),
            other => bail!("invalid optional game mode of {other}"),
        }))
    }
}
