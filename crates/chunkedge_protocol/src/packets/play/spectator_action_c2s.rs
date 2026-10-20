use chunkedge_binary::{Decode, Encode, VarInt};

use crate::Packet;

/// Requests a spectator action on an entity.
///
/// wiki: [Spectator Action](https://minecraft.wiki/w/Java_Edition_protocol#Spectator_Action)
#[derive(Copy, Clone, Debug, Encode, Decode, Packet)]
pub struct SpectatorActionC2s {
    pub entity_id: SpectatorTarget,
}

/// An optionally-present spectated entity, encoded like vanilla
/// `OPTIONAL_VAR_INT`: `0` means no target, otherwise the entity ID plus one.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct SpectatorTarget(pub Option<VarInt>);

impl Encode for SpectatorTarget {
    fn encode(&self, w: impl std::io::Write) -> anyhow::Result<()> {
        match self.0 {
            None => VarInt(0).encode(w),
            Some(id) => VarInt(id.0 + 1).encode(w),
        }
    }
}

impl Decode<'_> for SpectatorTarget {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        Ok(Self(match VarInt::decode(r)?.0 {
            0 => None,
            n => Some(VarInt(n - 1)),
        }))
    }
}
