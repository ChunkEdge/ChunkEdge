use chunkedge_binary::{Decode, Encode};

use crate::{GameMode, Packet};

/// Requests a game mode change (e.g. via F3+F4 or commands).
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct ChangeGameModeC2s {
    pub mode: GameMode,
}
