use std::io::Write;

use crate::{Decode, Encode};

/// One of two wire values, prefixed by a boolean: `true` for [`Self::Left`],
/// `false` for [`Self::Right`]. Matches Minecraft's `ByteBufCodecs.either` and
/// `FriendlyByteBuf.readEither`/`writeEither` codecs.
/// See: <https://minecraft.wiki/w/Java_Edition_protocol/Packets#Type:Either>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Encode, R: Encode> Encode for Either<L, R> {
    fn encode(&self, mut w: impl Write) -> anyhow::Result<()> {
        match self {
            Self::Left(value) => {
                true.encode(&mut w)?;
                value.encode(w)
            }
            Self::Right(value) => {
                false.encode(&mut w)?;
                value.encode(w)
            }
        }
    }
}

impl<'a, L: Decode<'a>, R: Decode<'a>> Decode<'a> for Either<L, R> {
    fn decode(r: &mut &'a [u8]) -> anyhow::Result<Self> {
        if bool::decode(r)? {
            Ok(Self::Left(L::decode(r)?))
        } else {
            Ok(Self::Right(R::decode(r)?))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VarInt;

    #[test]
    fn boolean_tagged_fixtures() {
        let cases: [(Either<VarInt, &str>, &[u8]); 2] = [
            (Either::Left(VarInt(300)), &[1, 0xac, 2]),
            (Either::Right("hello"), b"\x00\x05hello"),
        ];
        for (value, expected) in cases {
            let mut encoded = Vec::new();
            value.encode(&mut encoded).unwrap();
            assert_eq!(encoded, expected);
            for end in 0..expected.len() {
                assert!(Either::<VarInt, &str>::decode(&mut &expected[..end]).is_err());
            }
            encoded.push(0xaa);
            let mut input = encoded.as_slice();
            assert_eq!(Either::<VarInt, &str>::decode(&mut input).unwrap(), value);
            assert_eq!(input, &[0xaa]);
        }
    }
}
