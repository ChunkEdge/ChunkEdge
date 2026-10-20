use chunkedge_binary::{Decode, Encode};

use crate::{GameMode, Packet};

/// Requests a game mode change (e.g. via F3+F4 or commands).
///
/// wiki: [Change Game Mode](https://minecraft.wiki/w/Java_Edition_protocol#Change_Game_Mode)
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct ChangeGameModeC2s {
    pub mode: GameMode,
}
