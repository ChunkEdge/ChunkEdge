use std::io::Write;

use crate::{Decode, Encode, VarInt};

/// An optional nonnegative integer: zero means absent, otherwise the wire
/// value is one greater than the integer.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct OptionalVarInt(pub Option<i32>);

impl Encode for OptionalVarInt {
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        let value = match self.0 {
            None => 0,
            Some(n) => {
                anyhow::ensure!(n >= 0, "negative optional integer");
                n.checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("optional integer overflow"))?
            }
        };
        VarInt(value).encode(w)
    }
}

impl Decode<'_> for OptionalVarInt {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        let n = VarInt::decode(r)?.0;
        anyhow::ensure!(n >= 0, "negative optional integer");
        Ok(Self(if n == 0 { None } else { Some(n - 1) }))
    }
}
