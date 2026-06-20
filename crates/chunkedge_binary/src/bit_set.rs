use std::fmt;
use std::io::Write;

use anyhow::ensure;

use crate::{Decode, Encode, VarInt};

// TODO: when better const exprs are available, compute BYTE_COUNT from
// BIT_COUNT.
/// A fixed-size bit set encoded as a byte array.
///
/// Minecraft's protocol calls this type `Fixed BitSet(n)`. Bits are addressed
/// from the least-significant bit of the first byte.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct FixedBitSet<const BIT_COUNT: usize, const BYTE_COUNT: usize>(pub [u8; BYTE_COUNT]);

impl<const BIT_COUNT: usize, const BYTE_COUNT: usize> FixedBitSet<BIT_COUNT, BYTE_COUNT> {
    /// Returns whether the bit at `idx` is set.
    ///
    /// Returns `false` if `idx` is outside the fixed bit range.
    pub fn bit(&self, idx: usize) -> bool {
        check_counts(BIT_COUNT, BYTE_COUNT);
        debug_assert!(
            idx < BIT_COUNT,
            "bit index of {idx} out of range for bitset with {BIT_COUNT} bits"
        );

        self.0
            .get(idx / 8)
            .is_some_and(|byte| (byte >> (idx % 8)) & 1 == 1)
    }

    /// Sets the bit at `idx`.
    ///
    /// Does nothing if `idx` is outside the fixed bit range.
    pub fn set(&mut self, idx: usize) {
        check_counts(BIT_COUNT, BYTE_COUNT);
        debug_assert!(
            idx < BIT_COUNT,
            "bit index of {idx} out of range for bitset with {BIT_COUNT} bits"
        );

        if idx < BIT_COUNT {
            let byte = &mut self.0[idx / 8];
            *byte |= 1 << (idx % 8);
        }
    }

    /// Clears the bit at `idx`.
    ///
    /// Does nothing if `idx` is outside the fixed bit range.
    pub fn clear(&mut self, idx: usize) {
        check_counts(BIT_COUNT, BYTE_COUNT);
        debug_assert!(
            idx < BIT_COUNT,
            "bit index of {idx} out of range for bitset with {BIT_COUNT} bits"
        );

        if idx < BIT_COUNT {
            let byte = &mut self.0[idx / 8];
            *byte &= !(1 << (idx % 8));
        }
    }
}

impl<const BIT_COUNT: usize, const BYTE_COUNT: usize> Encode
    for FixedBitSet<BIT_COUNT, BYTE_COUNT>
{
    fn encode(&self, w: impl Write) -> anyhow::Result<()> {
        check_counts(BIT_COUNT, BYTE_COUNT);
        self.0.encode(w)
    }
}

impl<const BIT_COUNT: usize, const BYTE_COUNT: usize> Decode<'_>
    for FixedBitSet<BIT_COUNT, BYTE_COUNT>
{
    fn decode(r: &mut &'_ [u8]) -> anyhow::Result<Self> {
        check_counts(BIT_COUNT, BYTE_COUNT);
        Ok(Self(Decode::decode(r)?))
    }
}

const fn check_counts(bits: usize, bytes: usize) {
    assert!(bits.div_ceil(8) == bytes)
}

impl<const BIT_COUNT: usize, const BYTE_COUNT: usize> fmt::Debug
    for FixedBitSet<BIT_COUNT, BYTE_COUNT>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<const BIT_COUNT: usize, const BYTE_COUNT: usize> fmt::Display
    for FixedBitSet<BIT_COUNT, BYTE_COUNT>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0b")?;

        for i in (0..BIT_COUNT).rev() {
            if self.bit(i) {
                write!(f, "1")?;
            } else {
                write!(f, "0")?;
            }
        }

        Ok(())
    }
}

/// 😔
macro_rules! impl_default {
    ($($N:literal)*) => {
        $(
            impl<const BIT_COUNT: usize> Default for FixedBitSet<BIT_COUNT, $N> {
                fn default() -> Self {
                    Self(Default::default())
                }
            }
        )*
    }
}

impl_default!(0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16);

/// A dynamically-sized bit set encoded as a prefixed array of 64-bit words.
///
/// Minecraft's protocol calls this type `BitSet`. Bits are addressed from the
/// least-significant bit of the first word.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct VariableBitSet(pub Vec<i64>);

impl VariableBitSet {
    /// Creates an empty bit set.
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns whether the bit at `idx` is set.
    pub fn bit(&self, idx: usize) -> bool {
        let word = idx / 64;
        let bit = idx % 64;

        self.0
            .get(word)
            .is_some_and(|word| ((*word as u64) >> bit) & 1 == 1)
    }

    /// Sets the bit at `idx`.
    pub fn set(&mut self, idx: usize) {
        let word = idx / 64;
        let bit = idx % 64;

        self.0.resize(self.0.len().max(word + 1), 0);
        self.0[word] = (self.0[word] as u64 | (1 << bit)) as i64;
    }

    /// Clears the bit at `idx`.
    ///
    /// Clearing a bit past the current end is a no-op. Clearing the highest set
    /// bit shrinks the packed representation to match Java's `BitSet`.
    pub fn clear(&mut self, idx: usize) {
        let word = idx / 64;
        let bit = idx % 64;

        if let Some(word_value) = self.0.get_mut(word) {
            *word_value = (*word_value as u64 & !(1 << bit)) as i64;

            while self.0.last() == Some(&0) {
                self.0.pop();
            }
        }
    }
}

impl Encode for VariableBitSet {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        ensure!(
            i32::try_from(self.0.len()).is_ok(),
            "length of bit set exceeds i32::MAX (got {})",
            self.0.len()
        );

        VarInt(self.0.len() as i32).encode(&mut w)?;
        i64::encode_slice(&self.0, w)
    }
}

impl Decode<'_> for VariableBitSet {
    fn decode(r: &mut &[u8]) -> anyhow::Result<Self> {
        Ok(Self(Vec::<i64>::decode(r)?))
    }
}

impl fmt::Debug for VariableBitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl fmt::Display for VariableBitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_empty() {
            return write!(f, "0b0");
        }

        write!(f, "0b")?;

        for i in (0..self.0.len() * 64).rev() {
            write!(f, "{}", u8::from(self.bit(i)))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_bit_set_ops() {
        let mut bits = FixedBitSet::<20, 3>::default();

        assert!(!bits.bit(5));
        bits.set(5);
        assert!(bits.bit(5));
        assert_eq!(bits.0, [0b00100000, 0, 0]);

        bits.clear(5);
        assert!(!bits.bit(5));
        assert_eq!(bits.0, [0, 0, 0]);
    }

    #[test]
    #[cfg_attr(debug_assertions, should_panic)]
    fn fixed_bit_set_out_of_range_is_ignored() {
        let mut bits = FixedBitSet::<20, 3>::default();

        assert!(!bits.bit(20));
        bits.set(20);
        bits.clear(20);
        assert_eq!(bits.0, [0, 0, 0]);
    }

    #[test]
    fn display_fixed_bit_set() {
        let mut bits = FixedBitSet::<20, 3>::default();
        bits.set(5);

        assert_eq!(format!("{bits}"), "0b00000000000000100000");
    }

    #[test]
    fn variable_bit_set_ops() {
        let mut bits = VariableBitSet::default();

        assert!(!bits.bit(70));
        bits.set(70);
        assert!(bits.bit(70));
        assert_eq!(bits.0, vec![0, 0b0100_0000]);

        bits.clear(70);
        assert!(!bits.bit(70));
        assert!(bits.0.is_empty());
    }
}
