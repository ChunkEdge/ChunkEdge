use std::io::Write;

use anyhow::{Context, ensure};
use chunkedge_binary::{Decode, Encode, VarInt};
use derive_more::{From, Into};

/// Vanilla's low-precision vector, in blocks (or blocks per tick for motion).
#[derive(Copy, Clone, Default, Debug, PartialEq, From, Into)]
pub struct LpVec3(pub [f64; 3]);

const DATA_BITS_MASK: u64 = 0x7fff;
const MAX_QUANTIZED_VALUE: f64 = 32766.0;
const SCALE_BITS_MASK: u64 = 0x3;
const CONTINUATION_FLAG: u64 = 0x4;
const ABS_MAX_VALUE: f64 = 1.7179869183E10;
const ABS_MIN_VALUE: f64 = 0.00003051944088384301;

fn pack(value: f64) -> u64 {
    ((value * 0.5 + 0.5) * MAX_QUANTIZED_VALUE).round() as u64
}

fn unpack(value: u64) -> f64 {
    (value & DATA_BITS_MASK).min(MAX_QUANTIZED_VALUE as u64) as f64 * 2.0 / MAX_QUANTIZED_VALUE
        - 1.0
}

/// Matches vanilla `net.minecraft.network.LpVec3`: each component, in blocks
/// per tick, is quantized to 15 bits relative to a shared scale factor.
impl Encode for LpVec3 {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        let [x, y, z] = self.0.map(|v| {
            if v.is_nan() {
                0.0
            } else {
                v.clamp(-ABS_MAX_VALUE, ABS_MAX_VALUE)
            }
        });

        let chessboard_length = x.abs().max(y.abs()).max(z.abs());

        if chessboard_length < ABS_MIN_VALUE {
            w.write_all(&[0]).context("failed to write velocity")?;
            return Ok(());
        }

        let scale = chessboard_length.ceil() as u64;
        let is_partial = (scale & SCALE_BITS_MASK) != scale;
        let markers = if is_partial {
            scale & SCALE_BITS_MASK | CONTINUATION_FLAG
        } else {
            scale
        };

        let buffer = markers
            | (pack(x / scale as f64) << 3)
            | (pack(y / scale as f64) << 18)
            | (pack(z / scale as f64) << 33);

        // Two raw bytes followed by a big-endian int
        // (`FriendlyByteBuf.writeInt`).
        w.write_all(&[buffer as u8, (buffer >> 8) as u8])
            .context("failed to write velocity")?;
        w.write_all(&((buffer >> 16) as u32).to_be_bytes())
            .context("failed to write velocity")?;

        if is_partial {
            VarInt((scale >> 2) as i32)
                .encode(&mut w)
                .context("failed to write velocity scale")?;
        }

        Ok(())
    }
}

impl Decode<'_> for LpVec3 {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        ensure!(!r.is_empty(), "unexpected end of input decoding velocity");
        let lowest = u64::from(r[0]);
        *r = &r[1..];

        if lowest == 0 {
            return Ok(Self([0.0; 3]));
        }

        ensure!(r.len() >= 5, "unexpected end of input decoding velocity");
        let middle = u64::from(r[0]);
        // Vanilla reads the upper word with `readUnsignedInt` (big-endian).
        let highest = u64::from(u32::from_be_bytes([r[1], r[2], r[3], r[4]]));
        *r = &r[5..];

        let buffer = highest << 16 | middle << 8 | lowest;

        let mut scale = lowest & SCALE_BITS_MASK;
        if lowest & CONTINUATION_FLAG == CONTINUATION_FLAG {
            scale |= u64::from(VarInt::decode(r)?.0 as u32) << 2;
        }

        Ok(Self(
            [buffer >> 3, buffer >> 18, buffer >> 33].map(|packed| unpack(packed) * scale as f64),
        ))
    }
}
