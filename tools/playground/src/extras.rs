//! Put stuff in here if you find that you have to write the same code for
//! multiple playgrounds.

use chunkedge::prelude::*;

/// Toggles client's game mode between survival and creative when they start
/// sneaking.
pub(crate) fn toggle_gamemode_on_sneak(
    mut clients: Query<&mut GameMode>,
    mut messages: MessageReader<SneakMessage>,
) {
    for message in messages.read() {
        if message.state == SneakState::Start {
            if let Ok(mut mode) = clients.get_mut(message.client) {
                *mode = match *mode {
                    GameMode::Survival => GameMode::Creative,
                    GameMode::Creative => GameMode::Survival,
                    _ => GameMode::Creative,
                };
            }
        }
    }
}
