use std::io::Write;

use chunkedge_binary::{Decode, Encode, VarInt};
use chunkedge_math::DVec3;

/// Absolute movement is either a single destination or a timed sequence.
#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub enum PositionPath {
    Linear(DVec3),
    Stepped(Vec<PositionStep>),
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct PositionStep {
    pub position: DVec3,
    pub tick_offset: VarInt,
}

/// Relative coordinates use units of 1/4096 of a block.
#[derive(Clone, PartialEq, Debug)]
pub enum PositionDelta {
    Linear([i16; 3]),
    Stepped(Vec<DeltaStep>),
}

impl From<[i16; 3]> for PositionDelta {
    fn from(value: [i16; 3]) -> Self {
        Self::Linear(value)
    }
}

#[derive(Clone, PartialEq, Debug, Encode, Decode)]
pub struct DeltaStep {
    pub tick_offset: VarInt,
    pub delta: [i16; 3],
}

impl PositionDelta {
    pub(crate) fn encode_with_ground(
        &self,
        on_ground: bool,
        mut w: impl Write,
    ) -> anyhow::Result<()> {
        let count = match self {
            Self::Linear(_) => 0,
            Self::Stepped(steps) => {
                anyhow::ensure!(!steps.is_empty(), "stepped movement must contain a step");
                steps.len()
            }
        };
        anyhow::ensure!(count <= (i32::MAX >> 1) as usize, "too many movement steps");
        VarInt(((count as i32) << 1) | i32::from(on_ground)).encode(&mut w)?;
        match self {
            Self::Linear(delta) => delta.encode(w),
            Self::Stepped(steps) => {
                for step in steps {
                    step.encode(&mut w)?;
                }
                Ok(())
            }
        }
    }

    pub(crate) fn decode_with_ground(r: &mut &[u8]) -> anyhow::Result<(Self, bool)> {
        let properties = VarInt::decode(r)?.0 as u32;
        let count = (properties >> 1) as usize;
        let delta = if count == 0 {
            Self::Linear(Decode::decode(r)?)
        } else {
            anyhow::ensure!(
                count <= r.len() / 7,
                "movement step count exceeds remaining bytes"
            );
            let mut steps = Vec::with_capacity(count);
            for _ in 0..count {
                steps.push(DeltaStep::decode(r)?);
            }
            Self::Stepped(steps)
        };
        Ok((delta, properties & 1 != 0))
    }
}
