use std::borrow::Cow;

use chunkedge_generated::block::{BlockKind, BlockState, PropName, PropValue};
use chunkedge_nbt::Compound;

/// Projects saved (anvil) block-entity data onto the tag understood by the client.
pub(super) fn client_update_tag(state: BlockState, saved: &Compound) -> Cow<'_, Compound> {
    if state.to_kind() != BlockKind::TrialSpawner {
        return Cow::Borrowed(saved);
    }

    // TrialSpawnerStateData.getUpdateTag sends only the display entity and,
    // while active, its next spawn time. This prevents us from sending
    // server-only data to the client, which causes warnings in the client log.
    let mut tag = Compound::new();
    if let Some(spawn_data) = saved.get("spawn_data") {
        tag.insert("spawn_data", spawn_data.clone());
    }
    if state.get(PropName::TrialSpawnerState) == Some(PropValue::Active)
        && let Some(next_spawn) = saved.get("next_mob_spawns_at")
    {
        tag.insert("next_mob_spawns_at", next_spawn.clone());
    }
    Cow::Owned(tag)
}
