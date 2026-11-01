use chunkedge_binary::{Decode, Encode, VarInt};

use crate::{ByteAngle, Packet};

#[derive(Clone, Debug, Packet)]
pub struct MoveEntityPosRotS2c {
    pub entity_id: VarInt,
    pub delta: crate::entity_movement::PositionDelta,
    pub yaw: ByteAngle,
    pub pitch: ByteAngle,
    pub on_ground: bool,
}

impl Encode for MoveEntityPosRotS2c {
    fn encode(&self, mut w: impl std::io::Write) -> anyhow::Result<()> {
        self.entity_id.encode(&mut w)?;
        self.delta.encode_with_ground(self.on_ground, &mut w)?;
        self.yaw.encode(&mut w)?;
        self.pitch.encode(&mut w)?;
        Ok(())
    }
}

impl Decode<'_> for MoveEntityPosRotS2c {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        let entity_id = VarInt::decode(r)?;
        let (delta, on_ground) = crate::entity_movement::PositionDelta::decode_with_ground(r)?;
        Ok(Self {
            entity_id,
            delta,
            on_ground,
            yaw: Decode::decode(r)?,
            pitch: Decode::decode(r)?,
        })
    }
}
