use chunkedge_binary::{Decode, Encode};

// The field itself has no Encode/Decode implementation.
#[derive(Debug, PartialEq)]
struct Borrowed<'a>(&'a str);

mod borrowed {
    use std::io::Write;

    use anyhow::ensure;

    use super::*;

    pub(super) fn encode<W: Write>(value: &Borrowed<'_>, mut w: W) -> anyhow::Result<()> {
        u8::try_from(value.0.len())?.encode(&mut w)?;
        Ok(w.write_all(value.0.as_bytes())?)
    }

    pub(super) fn decode<'a>(r: &mut &'a [u8]) -> anyhow::Result<Borrowed<'a>> {
        let len = usize::from(u8::decode(r)?);
        ensure!(r.len() >= len, "truncated borrowed field");
        let (value, rest) = r.split_at(len);
        let value = std::str::from_utf8(value)?;
        *r = rest;
        Ok(Borrowed(value))
    }
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Named<'a> {
    prefix: u8,
    #[packet(with = borrowed)]
    value: Borrowed<'a>,
    suffix: u16,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Tuple<'a>(u8, #[packet(with = borrowed)] Borrowed<'a>, u16);

#[derive(Debug, PartialEq, Encode, Decode)]
enum Variants<'a> {
    Named {
        #[packet(with = borrowed)]
        value: Borrowed<'a>,
        suffix: u8,
    },
    Tuple(#[packet(with = borrowed)] Borrowed<'a>, u8),
}
